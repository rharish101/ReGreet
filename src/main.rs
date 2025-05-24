// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod cache;
mod client;
mod config;
mod constants;
mod gui;
mod i18n;
mod sysutil;
mod tomlutils;

use std::fs::{create_dir_all, OpenOptions};
use std::io::{Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use clap::{Parser, ValueEnum};
use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
use i18n_embed::DesktopLanguageRequester;
use tracing::subscriber::set_global_default;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::{
    filter::LevelFilter, fmt::layer, fmt::time::OffsetTime, layer::SubscriberExt,
};

use crate::constants::{APP_ID, CONFIG_PATH, CSS_PATH, LOG_PATH};
use crate::gui::{Greeter, GreeterInit};

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate const_format;
#[macro_use(Deserialize)]
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate test_case;

const MAX_LOG_FILES: usize = 3;
const MAX_LOG_SIZE: usize = 1024 * 1024;

static DEMO: OnceLock<bool> = OnceLock::new();

/// Get the demo mode status
fn demo() -> bool {
    *DEMO.get().unwrap_or(&false)
}

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
    /// The path to the log file
    #[arg(short = 'l', long, value_name = "PATH", default_value = LOG_PATH)]
    logs: PathBuf,

    /// The verbosity level of the logs
    #[arg(short = 'L', long, value_name = "LEVEL", default_value = "info")]
    log_level: LogLevel,

    /// Output all logs to stdout
    #[arg(short, long)]
    verbose: bool,

    /// The path to the config file
    #[arg(short, long, value_name = "PATH", default_value = CONFIG_PATH)]
    config: PathBuf,

    /// The path to the custom CSS stylesheet
    #[arg(short, long, value_name = "PATH", default_value = CSS_PATH)]
    style: PathBuf,

    /// Run in demo mode
    #[arg(long)]
    demo: bool,
}

fn main() {
    i18n::init(&DesktopLanguageRequester::requested_languages());

    let args = Args::parse();
    DEMO.get_or_init(|| args.demo);

    // Keep the guard alive till the end of the function, since logging depends on this.
    let _guard = init_logging(&args.logs, &args.log_level, args.verbose);

    let app = relm4::RelmApp::new(APP_ID);
    app.with_args(vec![]).run_async::<Greeter>(GreeterInit {
        config_path: args.config,
        css_path: args.style,
        demo: args.demo,
    });
}

/// Initialize the log file with file rotation.
fn setup_log_file(log_path: &Path) -> IoResult<FileRotate<AppendCount>> {
    if !log_path.exists() {
        if let Some(log_dir) = log_path.parent() {
            create_dir_all(log_dir)?;
        };
    };

    // Manually write to the log file, since `FileRotate` will silently fail if the log file can't
    // be written to.
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    file.write_all(&[])?;

    Ok(FileRotate::new(
        log_path,
        AppendCount::new(MAX_LOG_FILES),
        ContentLimit::Bytes(MAX_LOG_SIZE),
        Compression::OnRotate(0),
        None,
    ))
}

/// Initialize logging with file rotation.
fn init_logging(log_path: &Path, log_level: &LogLevel, stdout: bool) -> Vec<WorkerGuard> {
    // Parse the log level string.
    let filter = match log_level {
        LogLevel::Off => LevelFilter::OFF,
        LogLevel::Error => LevelFilter::ERROR,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Trace => LevelFilter::TRACE,
    };

    // Load the timer before spawning threads, otherwise getting the local time offset will fail.
    let timer = OffsetTime::local_rfc_3339().expect("Couldn't get local time offset");

    // Set up the logger.
    let builder = tracing_subscriber::fmt()
        .with_max_level(filter)
        // The timer could be reused later.
        .with_timer(timer.clone());

    // Log in a separate non-blocking thread, then return the guard (otherise the non-blocking
    // writer will immediately stop).
    let mut guards = Vec::new();
    match setup_log_file(log_path) {
        Ok(file) => {
            let (file, guard) = non_blocking(file);
            guards.push(guard);
            let builder = builder
                .with_writer(file)
                // Disable colouring through ANSI escape sequences in log files.
                .with_ansi(false);

            if stdout {
                let (stdout, guard) = non_blocking(std::io::stdout());
                guards.push(guard);
                set_global_default(
                    builder
                        .finish()
                        .with(layer().with_writer(stdout).with_timer(timer)),
                )
                .unwrap();
            } else {
                builder.init();
            };
        }
        Err(file_err) => {
            let (file, guard) = non_blocking(std::io::stdout());
            guards.push(guard);
            builder.with_writer(file).init();
            tracing::error!("Couldn't create log file '{LOG_PATH}': {file_err}");
        }
    };

    // Log all panics in the log file as well as stderr.
    std::panic::set_hook(Box::new(|panic| {
        tracing::error!("{panic}");
        eprintln!("{panic}");
    }));

    guards
}
