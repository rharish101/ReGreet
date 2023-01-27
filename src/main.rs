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
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};

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
    init_logging(args.log_level);

    let app = relm4::RelmApp::new(APP_ID);
    app.run::<Greeter>(());
}

/// Initialize logging with file rotation.
fn init_logging(log_level: LogLevel) {
    let log_path = Path::new(LOG_PATH);
    if !log_path.exists() {
        // Create the log directory.
        if let Some(log_dir) = log_path.parent() {
            create_dir_all(log_dir).expect("Couldn't create missing log directory");
        };
    }

    // Set up the log file rotation.
    let log = FileRotate::new(
        log_path,
        AppendCount::new(MAX_LOG_FILES),
        ContentLimit::Bytes(MAX_LOG_SIZE),
        Compression::OnRotate(0),
        None,
    );

    // Parse the log level string.
    let filter = match log_level {
        LogLevel::Off => LevelFilter::Off,
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Trace => LevelFilter::Trace,
    };

    // Set up the logger.
    let mut config_builder = ConfigBuilder::new();
    // Ignore failure in getting the local time zone.
    let _ = config_builder
        .set_time_format_rfc3339()
        .set_time_offset_to_local();
    WriteLogger::init(filter, config_builder.build(), log).expect("Couldn't setup logging");
}
