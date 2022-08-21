mod cache;
mod client;
mod common;
mod config;
mod constants;
mod gui;
mod sysutil;

use std::env;
use std::fs::create_dir_all;
use std::path::Path;

use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
use gtk::{
    gio,
    glib::{Char, OptionArg, OptionFlags, VariantDict},
    prelude::*,
    Application,
};
use gui::Greeter;
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};

use crate::constants::{APP_ID, LOG_PATH};

const MAX_LOG_FILES: usize = 3;
const MAX_LOG_SIZE: usize = 1024 * 1024;

const LOG_LEVEL_CLI_ARG: &str = "log-level";
// NOTE: A change in this should also change the argument description
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Warn;

fn main() {
    // Register and include resources
    gio::resources_register_include!("compiled.gresource").expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Specify CLI args and the handler
    app.add_main_option(
        LOG_LEVEL_CLI_ARG,
        Char::from(b'l'),
        OptionFlags::NONE,
        OptionArg::String,
        "The verbosity level of the logs [allowed: off, error, warn, info, debug, trace] [default: warn]",
        Some("LEVEL"),
    );
    app.connect_handle_local_options(handle_cli_args);

    // Connect to the "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

/// Handle the CLI arguments
fn handle_cli_args(_: &Application, args: &VariantDict) -> i32 {
    let log_level = if let Ok(value) = args.lookup::<String>(LOG_LEVEL_CLI_ARG) {
        if let Some(value) = value {
            value
        } else {
            String::new()
        }
    } else {
        // Invalid argument type, so return a positive number indicating that the app should crash
        return 1;
    };
    init_logging(&log_level);

    // This denotes that the app should continue to function
    -1
}

/// Initialize logging with file rotation
fn init_logging(log_level: &str) {
    let log_path = Path::new(LOG_PATH);
    if !log_path.exists() {
        // Create the log directory
        if let Some(log_dir) = log_path.parent() {
            create_dir_all(log_dir).expect("Couldn't create missing log directory");
        };
    }

    // Setup the log file rotation
    let log = FileRotate::new(
        log_path,
        AppendCount::new(MAX_LOG_FILES),
        ContentLimit::Bytes(MAX_LOG_SIZE),
        Compression::OnRotate(0),
        None,
    );

    // Parse the log level string
    let log_level = match log_level {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => DEFAULT_LOG_LEVEL,
    };

    // Setup the logger
    let mut config_builder = ConfigBuilder::new();
    // Ignore failure in getting the local time zone
    let _ = config_builder
        .set_time_format_rfc3339()
        .set_time_offset_to_local();
    WriteLogger::init(log_level, config_builder.build(), log).expect("Couldn't setup logging");
}

/// Create a new greeter window and show it
fn build_ui(app: &Application) {
    let window = Greeter::new(app);
    window.present();
}
