//! Include GTK UI resources when building
use gtk::gio;

fn main() {
    gio::compile_resources(
        "resources",
        "resources/resources.gresource.xml",
        "compiled.gresource",
    );
}
