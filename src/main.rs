mod cache;
mod client;
mod common;
mod config;
mod constants;
mod gui;
mod lru;
mod sysutil;

use gtk::{gio, prelude::*, Application};
use gui::Greeter;

use crate::constants::APP_ID;

fn main() {
    // Setup the logger
    pretty_env_logger::init();

    // Register and include resources
    gio::resources_register_include!("compiled.gresource").expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to the "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

/// Create a new greeter window and show it
fn build_ui(app: &Application) {
    let window = Greeter::new(app);
    window.present();
}
