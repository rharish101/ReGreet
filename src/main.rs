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
use gtk::{gio, prelude::*, Application};
use gui::Greeter;
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};

use crate::constants::{APP_ID, LOG_PATH};

const MAX_LOG_FILES: usize = 3;
const MAX_LOG_SIZE: usize = 1024 * 1024;
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Warn;

fn main() {
    // Setup logging
    init_logging();

    // Register and include resources
    gio::resources_register_include!("compiled.gresource").expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to the "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

/// Initialize logging with file rotation
fn init_logging() {
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

    // Get the log level from the environment
    let log_level = match env::var("RUST_LOG").map(|x| x.to_lowercase()).as_deref() {
        Ok("off") => LevelFilter::Off,
        Ok("error") => LevelFilter::Error,
        Ok("warn") => LevelFilter::Warn,
        Ok("info") => LevelFilter::Info,
        Ok("debug") => LevelFilter::Debug,
        Ok("trace") => LevelFilter::Trace,
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
