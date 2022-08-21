//! Subclass of the greeter GUI that holds the state
use std::cell::RefCell;

use glib::subclass::InitializingObject;
use gtk::{
    glib, prelude::*, subclass::prelude::*, ApplicationWindow, Button, ComboBoxText,
    CompositeTemplate, Entry, Label,
};

use crate::cache::Cache;
use crate::client::GreetdClient;
use crate::config::Config;
use crate::sysutil::SysUtil;

/// Part of the greeter GUI that holds the greeter's state
#[derive(CompositeTemplate)]
#[template(resource = "/apps/egreet/window.ui")]
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
    #[template_child]
    pub(super) password_entry: TemplateChild<Entry>,
    /// The label for the password widget
    #[template_child]
    pub(super) password_label: TemplateChild<Label>,
    /// The widget to display messages to the user
    #[template_child]
    pub(super) message_label: TemplateChild<Label>,
    /// The widget containing the usernames
    #[template_child]
    pub(super) usernames_box: TemplateChild<ComboBoxText>,
    /// The widget containing the sessions
    #[template_child]
    pub(super) sessions_box: TemplateChild<ComboBoxText>,
    /// The label for the sessions widget
    #[template_child]
    pub(super) sessions_label: TemplateChild<Label>,

    /// The button to enter the password and login
    #[template_child]
    pub(super) login_button: TemplateChild<Button>,
    /// The button to cancel password entry
    #[template_child]
    pub(super) cancel_button: TemplateChild<Button>,
    /// The button to reboot
    #[template_child]
    pub(super) reboot_button: TemplateChild<Button>,
    /// The button to power-off
    #[template_child]
    pub(super) poweroff_button: TemplateChild<Button>,
}

impl Default for Greeter {
    /// Initialize the greeter
    fn default() -> Self {
        // Get the utilities that we use
        let greetd_client =
            RefCell::new(GreetdClient::new().expect("Couldn't initialize greetd client"));
        let sys_util = SysUtil::new().expect("Couldn't read available users and sessions");
        let cache = RefCell::new(Cache::new());
        let config = Config::new();

        // Use the template defaults, since the UI builder will load them anyway
        let password_entry = TemplateChild::default();
        let password_label = TemplateChild::default();
        let message_label = TemplateChild::default();
        let usernames_box = TemplateChild::default();
        let sessions_box = TemplateChild::default();
        let sessions_label = TemplateChild::default();
        let login_button = TemplateChild::default();
        let cancel_button = TemplateChild::default();
        let reboot_button = TemplateChild::default();
        let poweroff_button = TemplateChild::default();

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
            sessions_label,
            login_button,
            cancel_button,
            reboot_button,
            poweroff_button,
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for Greeter {
    const NAME: &'static str = "Greeter";
    type Type = super::Greeter;
    type ParentType = ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
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
