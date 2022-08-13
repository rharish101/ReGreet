/// Subclass of the greeter GUI that holds the state
use std::cell::RefCell;

use gtk4::{
    glib, subclass::prelude::*, ApplicationWindow, Builder, Button, ComboBoxText, Entry, Label,
    Window,
};

use crate::cache::Cache;
use crate::client::GreetdClient;
use crate::config::Config;
use crate::constants::UI_FILE_PATH;
use crate::sysutil::SysUtil;

/// Part of the greeter GUI that holds the greeter's state
pub struct Greeter {
    /// Client to communicate with greetd
    // RefCell is needed since we need to borrow the client as mutable
    pub(super) greetd_client: RefCell<GreetdClient>,
    /// System utility to get available users and sessions
    pub(super) sys_util: SysUtil,
    /// The cache that persists between logins
    // RefCell is needed since we need to borrow the cache as mutable
    pub(super) cache: RefCell<Cache>,
    /// The config for this greeter
    pub(super) config: Config,

    /// The widget where the user enters the password
    pub(super) password_entry: Entry,
    /// The label for the password widget
    pub(super) password_label: Label,
    /// The widget to display messages to the user
    pub(super) message_label: Label,
    /// The widget containing the usernames
    pub(super) usernames_box: ComboBoxText,
    /// The widget containing the sessions
    pub(super) sessions_box: ComboBoxText,

    /// The default window
    pub(super) login_window: Window,
    /// The button to enter the password and login
    pub(super) login_button: Button,
    /// The button to reboot
    pub(super) reboot_button: Button,
    /// The button to power-off
    pub(super) poweroff_button: Button,
}

impl Default for Greeter {
    /// Initialize the greeter
    fn default() -> Self {
        // Get the utilities that we use
        let greetd_client = RefCell::new(GreetdClient::new().unwrap());
        let sys_util = SysUtil::new().unwrap();
        let cache = RefCell::new(Cache::new());
        let config = Config::new();
        let builder = Builder::from_file(UI_FILE_PATH);

        // Get the GUI elements that we need references to
        let password_entry: Entry = builder
            .object("password_entry")
            .expect("Invalid GTK UI file: missing password entry");
        let password_label = builder
            .object("password_label")
            .expect("Invalid GTK UI file: missing password label");
        let message_label: Label = builder
            .object("message_label")
            .expect("Invalid GTK UI file: missing message label");
        let usernames_box: ComboBoxText = builder
            .object("usernames_cb")
            .expect("Invalid GTK UI file: missing username combo box");
        let sessions_box: ComboBoxText = builder
            .object("sessions_cb")
            .expect("Invalid GTK UI file: missing session combo box");
        let login_window: Window = builder.object("login_window").expect("Invalid GTK UI file");
        let login_button: Button = builder.object("login_button").expect("Invalid GTK UI file");
        let reboot_button: Button = builder
            .object("reboot_button")
            .expect("Invalid GTK UI file");
        let poweroff_button: Button = builder
            .object("poweroff_button")
            .expect("Invalid GTK UI file");

        Self {
            greetd_client,
            sys_util,
            cache,
            config,
            password_entry,
            password_label,
            message_label,
            usernames_box,
            sessions_box,
            login_window,
            login_button,
            reboot_button,
            poweroff_button,
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for Greeter {
    const NAME: &'static str = "GreeterGUI";
    type Type = super::Greeter;
    type ParentType = ApplicationWindow;
}

impl ObjectImpl for Greeter {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        // Setup all GUI elements
        obj.setup();
    }
}

impl WidgetImpl for Greeter {}
impl WindowImpl for Greeter {}
impl ApplicationWindowImpl for Greeter {}
