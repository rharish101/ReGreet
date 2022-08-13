mod cache;
mod client;
mod common;
mod config;
mod constants;
mod gui;
mod lru;
mod sysutil;

use gtk4::{prelude::*, Application};
use gui::Greeter;

fn main() {
    // Setup the logger
    pretty_env_logger::init();

    // Create a new application
    let app = Application::builder()
        .application_id("egreet")
        .build();

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
