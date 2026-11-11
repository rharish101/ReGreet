// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod cache;
mod cli;
mod client;
mod config;
mod constants;
mod gui;
mod sysutil;
mod tomlutils;

use std::fs::{OpenOptions, create_dir_all};
use std::io::{Result as IoResult, Write};
use std::path::Path;

use clap::{Arg, ArgAction, Args, Command, FromArgMatches};
use file_rotate::{ContentLimit, FileRotate, compression::Compression, suffix::AppendCount};
use gettextrs::{LocaleCategory, bindtextdomain, gettext, setlocale, textdomain};
use tracing::subscriber::set_global_default;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::{
    filter::LevelFilter, fmt::layer, fmt::time::OffsetTime, layer::SubscriberExt,
};

use crate::cli::{CliArgs, LogLevel};
use crate::constants::{APP_ID, GREETER_NAME, LOCALE_DIR, LOG_PATH};
use crate::gui::{Greeter, GreeterInit};

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate const_format;

const MAX_LOG_FILES: usize = 3;
const MAX_LOG_SIZE: usize = 1024 * 1024;

fn main() {
    unsafe {
        setlocale(LocaleCategory::LcAll, "");
        bindtextdomain(GREETER_NAME, LOCALE_DIR).unwrap();
        textdomain(GREETER_NAME).unwrap();
    }

    let cli = Command::new(env!("CARGO_PKG_NAME"))
        .disable_help_flag(true)
        .disable_version_flag(true)
        .version(env!("CARGO_PKG_VERSION"))
        .override_usage(format!(
            "{} [{}]",
            env!("CARGO_PKG_NAME"),
            gettext("OPTIONS"),
        ))
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::Help)
                .display_order(999)
                .help(gettext("Print help")),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::Version)
                .display_order(1000)
                .help(gettext("Print version")),
        )
        // We don't use the description in Cargo.toml, to simplify translation
        .about(gettext("Clean and customizable greeter for greetd"))
        .help_template(format!(
            "{{about-section}}\n{} {{usage}}\n{}\n{{options}}",
            gettext("Usage:"),
            gettext("Options:")
        ));

    let cliargs = CliArgs::augment_args(cli);
    let matches = cliargs.get_matches();
    let args = CliArgs::from_arg_matches(&matches).unwrap();

    // Keep the guard alive till the end of the function, since logging depends on this.
    let _guard = init_logging(&args.logs, &args.log_level, args.verbose);

    // We will make the app invisible on startup, then make it visible after style loaded and initialized.
    let app = relm4::RelmApp::new(APP_ID).visible_on_activate(false);
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
