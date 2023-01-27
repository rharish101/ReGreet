mod cache;
mod client;
mod common;
mod config;
mod constants;
mod gui;
mod sysutil;

use std::fs::create_dir_all;
use std::path::Path;

use clap::{Parser, ValueEnum};
use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
use gui::Greeter;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::{filter::LevelFilter, fmt::time::OffsetTime};

use crate::constants::{APP_ID, LOG_PATH};

const MAX_LOG_FILES: usize = 3;
const MAX_LOG_SIZE: usize = 1024 * 1024;

#[derive(Clone, Debug, ValueEnum)]
enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The verbosity level of the logs
    #[arg(short, long, value_name = "LEVEL", default_value = "warn")]
    log_level: LogLevel,
}

fn main() {
    let args = Args::parse();
    // Keep the guard alive till the end of the function, since logging depends on this.
    let _guard = init_logging(args.log_level);

    let app = relm4::RelmApp::new(APP_ID);
    app.run::<Greeter>(());
}

/// Initialize logging with file rotation.
fn init_logging(log_level: LogLevel) -> WorkerGuard {
    let log_path = Path::new(LOG_PATH);
    if !log_path.exists() {
        // Create the log directory.
        if let Some(log_dir) = log_path.parent() {
            create_dir_all(log_dir).expect("Couldn't create missing log directory");
        };
    }

    // Load the timer before spawning threads, otherwise getting the local time offset will fail.
    let timer = OffsetTime::local_rfc_3339().expect("Couldn't get local time offset");

    // Set up the log file rotation in a separate non-blocking thread.
    let (log_file, guard) = non_blocking(FileRotate::new(
        log_path,
        AppendCount::new(MAX_LOG_FILES),
        ContentLimit::Bytes(MAX_LOG_SIZE),
        Compression::OnRotate(0),
        None,
    ));

    // Parse the log level string.
    let filter = match log_level {
        LogLevel::Off => LevelFilter::OFF,
        LogLevel::Error => LevelFilter::ERROR,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Trace => LevelFilter::TRACE,
    };

    // Set up the logger.
    tracing_subscriber::fmt()
        .with_writer(log_file)
        .with_max_level(filter)
        .with_ansi(false)
        .with_timer(timer)
        .init();

    // Return this guard, otherwise the non-blocking writer will immediately stop.
    guard
}
