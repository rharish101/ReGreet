//! The main GUI for the greeter
mod imp;

use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use chrono::Local;
use greetd_ipc::{ErrorType as GreetdErrorType, Response};
use gtk::{
    gio, glib,
    glib::{MainContext, PRIORITY_DEFAULT, PRIORITY_LOW},
    prelude::*,
    subclass::prelude::*,
    Application, Button,
};
use log::{debug, error, info, warn};

use crate::client::AuthStatus;
use crate::common::capitalize;

const DEFAULT_MSG: &str = "Welcome back!";
const ERROR_MSG_CLEAR_DELAY: u64 = 5;

const DATETIME_FMT: &str = "%a %R";
const DATETIME_UPDATE_DELAY: u64 = 500;

// Inherit from GtkApplicationWindow: https://docs.gtk.org/gtk4/class.ApplicationWindow.html
glib::wrapper! {
    /// Part of the greeter GUI that defines the behaviour
    pub struct Greeter(ObjectSubclass<imp::Greeter>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Greeter {
    /// Create a new window for the greeter
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create the greeter window")
    }

    /// Setup all GUI elements
    pub fn setup(&self) {
        // Cancel any previous session, just in case someone started one
        if let Err(err) = self.imp().greetd_client.borrow_mut().cancel_session() {
            warn!("Couldn't cancel greetd session: {}", err);
        };

        // Setup the welcome message
        self.imp().message_label.set_text(DEFAULT_MSG);

        self.setup_background();
        self.setup_settings();
        self.setup_callbacks();
        self.setup_users_sessions();
        self.setup_datetime_display();

        // Make the window fullscreen
        self.fullscreen();
    }

    /// Setup the image background
    fn setup_background(&self) {
        if let Some(bg_path) = self.imp().config.get_background() {
            if Path::new(bg_path).exists() {
                debug!("Setting background: {}", bg_path);
                self.imp().background.set_filename(Some(bg_path));
            } else {
                warn!("Couldn't find requested background: {}", bg_path);
            }
        } else {
            debug!("No background configured");
        }
    }

    /// Load GTK settings from the greeter config
    fn setup_settings(&self) {
        let settings = self.settings();
        let config = if let Some(config) = self.imp().config.get_gtk_settings() {
            config
        } else {
            return;
        };

        debug!(
            "Setting dark theme: {}",
            config.application_prefer_dark_theme
        );
        settings.set_gtk_application_prefer_dark_theme(config.application_prefer_dark_theme);

        if let Some(cursor_theme) = &config.cursor_theme_name {
            debug!("Setting cursor theme: {}", cursor_theme);
        };
        settings.set_gtk_cursor_theme_name(config.cursor_theme_name.as_deref());

        if let Some(font) = &config.font_name {
            debug!("Setting font: {}", font);
        };
        settings.set_gtk_font_name(config.font_name.as_deref());

        if let Some(icon_theme) = &config.icon_theme_name {
            debug!("Setting icon theme: {}", icon_theme);
        };
        settings.set_gtk_icon_theme_name(config.icon_theme_name.as_deref());

        if let Some(theme) = &config.theme_name {
            debug!("Setting theme: {}", theme);
        };
        settings.set_gtk_theme_name(config.theme_name.as_deref());
    }

    /// Register handlers for GUI elements
    fn setup_callbacks(&self) {
        self.imp()
            .reboot_button
            .connect_clicked(Self::reboot_click_handler);
        self.imp()
            .poweroff_button
            .connect_clicked(Self::poweroff_click_handler);
        self.imp()
            .usernames_box
            .connect_changed(glib::clone!(@weak self as gui => move |_| gui.user_change_handler()));
        self.imp()
            .login_button
            .connect_clicked(glib::clone!(@weak self as gui => move |_| gui.login_click_handler()));
        self.imp().password_entry.connect_activate(
            glib::clone!(@weak self as gui => move |_| gui.login_click_handler()),
        );
        self.imp().cancel_button.connect_clicked(
            glib::clone!(@weak self as gui => move |_| gui.cancel_click_handler()),
        );

        // Set the default behaviour of pressing the Return key to act like the login button
        self.set_default_widget(Some(&self.imp().login_button.get()));
    }

    /// Populate the user and session combo boxes with entries
    fn setup_users_sessions(&self) {
        // The user that is shown during initial login
        let mut initial_username = None;

        // Populate the usernames combo box
        for (user, username) in self.imp().sys_util.get_users().iter() {
            debug!("Found user: {}", user);
            if initial_username.is_none() {
                initial_username = Some(username.clone());
            }
            self.imp().usernames_box.append(Some(username), user);
        }

        // Populate the sessions combo box
        for session in self.imp().sys_util.get_sessions().keys() {
            debug!("Found session: {}", session);
            self.imp().sessions_box.append(Some(session), session);
        }

        // If the last user is known, show their login initially
        if let Some(last_user) = self.imp().cache.borrow().get_last_user() {
            initial_username = Some(last_user.to_string());
        } else if let Some(user) = &initial_username {
            info!("Using first found user '{}' as initial user", user);
        }

        // Set the user shown initially at login
        if !self
            .imp()
            .usernames_box
            .set_active_id(initial_username.as_deref())
        {
            if let Some(user) = initial_username {
                warn!("Couldn't find user '{}' to set as the initial user", user);
            }
        }
    }

    /// Setup auto updation for the datetime label
    fn setup_datetime_display(&self) {
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

        // Set a timer in a separate thread that signals the main thread to reset the displayed
        // message, so as to not block the GUI
        thread::spawn(move || loop {
            if sender.send(()).is_err() {
                warn!("Couldn't update datetime");
            };
            thread::sleep(Duration::from_millis(DATETIME_UPDATE_DELAY));
        });

        // Resets the displayed message after getting the signal to do so
        receiver.attach(
            None,
            glib::clone!(@weak self as gui => @default-return Continue(true), move |_| {
                let datetime_str = &format!("{}", Local::now().format(DATETIME_FMT));
                gui.imp().datetime_label.set_text(datetime_str); Continue(true)
            }),
        );
    }

    /// Event handler for clicking the "Reboot" button
    ///
    /// This reboots the PC.
    fn reboot_click_handler(_: &Button) {
        info!("Rebooting");
        if let Err(err) = Command::new("systemctl").arg("reboot").spawn() {
            error!("Failed to reboot: {}", err);
        }
    }

    /// Event handler for clicking the "Power-Off" button
    ///
    /// This shuts down the PC.
    fn poweroff_click_handler(_: &Button) {
        info!("Shutting down");
        if let Err(err) = Command::new("systemctl").arg("poweroff").spawn() {
            error!("Failed to poweroff: {}", err);
        }
    }

    /// Event handler for clicking the "Cancel" button
    ///
    /// This cancels the created session and goes back to the user/session chooser.
    fn cancel_click_handler(&self) {
        // Cancel the current session
        if let Err(err) = self.imp().greetd_client.borrow_mut().cancel_session() {
            warn!("Couldn't cancel greetd session: {}", err);
        };

        // Clear the password entry field
        self.imp().password_entry.set_text("");

        // Move out of the password mode
        self.set_password_mode(false);
    }

    /// Create a greetd session, i.e. starts a login attempt for the current user
    fn create_session(&self) {
        // Get the current username
        let username = if let Some(username) = self.get_current_username() {
            username
        } else {
            // No username found (which shouldn't happen), so we can't create the session
            return;
        };

        // Before trying to create a session, check if the session command (if manually entered) is
        // valid
        if self.imp().sessions_box.active_id().is_none() {
            if let Some(cmd) = self.imp().sessions_box.active_text() {
                if shlex::split(cmd.as_str()).is_none() {
                    // This must be an invalid command
                    self.display_error(
                        "Invalid session command",
                        &format!("Invalid session command: {}", cmd),
                    );
                    return;
                };
                debug!("Manually entered session command is parsable");
            };
        };

        info!("Creating session for user: {}", username);

        // Create a session for the current user
        let response = self
            .imp()
            .greetd_client
            .borrow_mut()
            .create_session(&username)
            .unwrap_or_else(|err| {
                panic!(
                    "Failed to create session for username '{}': {}",
                    username, err
                )
            });

        match response {
            Response::Success => {
                // No password is needed, so directly start session
                info!("No password needed for current user");
                self.start_session();
            }
            Response::AuthMessage { auth_message, .. } => {
                if auth_message.to_lowercase().contains("password") {
                    // Show the password field, because a password is needed
                    self.set_password_mode(true);
                    self.imp().password_entry.set_text("");
                } else {
                    // greetd has requested something other than the password, so just display it
                    // to the user and let them figure it out
                    self.display_error(
                        &capitalize(&auth_message),
                        &format!(
                            "Expected password request, but greetd requested: {}",
                            auth_message
                        ),
                    );
                };
            }
            Response::Error { description, .. } => {
                self.display_error(
                    &capitalize(&description),
                    &format!("Message from greetd: {}", description),
                );
            }
        };
    }

    /// Event handler for selecting a different username in the `ComboBoxText`
    ///
    /// This changes the session in the combo box according to the last used session of the current user.
    fn user_change_handler(&self) {
        // Get the current username
        let username = if let Some(username) = self.get_current_username() {
            username
        } else {
            // No username found (which shouldn't happen), so we can't change the session
            return;
        };

        if let Some(last_session) = self.imp().cache.borrow_mut().get_last_session(&username) {
            // Set the last session used by this user in the session combo box
            if !self.imp().sessions_box.set_active_id(Some(last_session)) {
                warn!(
                    "Last session '{}' for user '{}' missing",
                    last_session, username
                );
            }
        } else {
            // Last session not found, so skip changing the session
            info!("Last session for user '{}' missing", username);
        };
    }

    /// Event handler for clicking the "Login" button
    ///
    /// This does one of the following, depending of the state of authentication:
    ///     - Begins a login attempt for the given user
    ///     - Submits the entered password for logging in and starts the session
    fn login_click_handler(&self) {
        // Check if a password is needed. If not, then directly start the session.
        let auth_status = self.imp().greetd_client.borrow().get_auth_status().clone();
        match auth_status {
            AuthStatus::Done => {
                // No password is needed, but the session should've been already started by
                // `create_session`
                warn!("No password needed for current user, but session not already started");
                self.start_session();
            }
            AuthStatus::InProgress => {
                self.send_password();
            }
            AuthStatus::NotStarted => {
                self.create_session();
            }
        };
    }

    /// Send the entered password for logging in
    fn send_password(&self) {
        // Get the entered password
        let password = self.imp().password_entry.text().to_string();
        // Reset the password field, for convenience when the user has to re-enter a password
        self.imp().password_entry.set_text("");

        // Send the password, as authentication for the current user
        let resp = self
            .imp()
            .greetd_client
            .borrow_mut()
            .send_password(Some(password))
            .unwrap_or_else(|err| panic!("Failed to send password: {}", err));

        match resp {
            Response::Success => {
                info!("Successfully logged in; starting session");
                self.start_session();
            }
            // The client should raise an `unimplemented!`, so ignore it
            Response::AuthMessage { .. } => (),
            Response::Error {
                error_type,
                description,
            } => {
                match error_type {
                    GreetdErrorType::AuthError => {
                        // Most likely entered the wrong password
                        self.display_error("Login failed", "Login failed");
                    }
                    GreetdErrorType::Error => {
                        self.display_error(
                            &capitalize(&description),
                            &format!("Message from greetd: {}", description),
                        );
                    }
                }
            }
        };
    }

    /// Get the currently selected username
    fn get_current_username(&self) -> Option<String> {
        // Get the currently selected user's ID, which should be their username
        if let Some(username) = self.imp().usernames_box.active_id() {
            debug!("Retrieved username '{}' from options", username);
            Some(username.to_string())
        } else if let Some(username) = self.imp().usernames_box.active_text() {
            // In case of manual entry, the ID should be missing
            debug!("Retrieved username '{}' through manual entry", username);
            Some(username.to_string())
        } else {
            // This shouldn't happen, since we have an entry within the usernames box
            error!("No username entered");
            None
        }
    }

    /// Enter or exit the password mode
    fn set_password_mode(&self, enter: bool) {
        // Show the password entry and label
        self.imp().password_entry.set_sensitive(enter);
        self.imp().password_entry.set_visible(enter);
        self.imp().password_label.set_sensitive(enter);
        self.imp().password_label.set_visible(enter);

        // Show the cancel button
        self.imp().cancel_button.set_sensitive(enter);
        self.imp().cancel_button.set_visible(enter);

        // Hide the session chooser and label
        self.imp().sessions_box.set_sensitive(!enter);
        self.imp().sessions_box.set_visible(!enter);
        self.imp().sessions_label.set_sensitive(!enter);
        self.imp().sessions_label.set_visible(!enter);

        // Make the user chooser unchangeable
        self.imp().usernames_box.set_sensitive(!enter);

        // Focus on the most convenient element
        if enter {
            self.imp().password_entry.grab_focus();
        } else {
            self.imp().usernames_box.grab_focus();
        }
    }

    /// Get the currently selected session name (if available) and command
    fn get_current_session_cmd(&self) -> (Option<String>, Option<Vec<String>>) {
        // Get the currently selected session
        if let Some(session) = self.imp().sessions_box.active_id() {
            debug!("Retrieved current session: {}", session);
            if let Some(cmd) = self.imp().sys_util.get_sessions().get(session.as_str()) {
                (Some(session.to_string()), Some(cmd.clone()))
            } else {
                // Shouldn't happen, unless there are no sessions available
                let error_msg = format!("Session '{}' not found", session);
                self.display_error(&error_msg, &error_msg);
                (None, None)
            }
        } else if let Some(manual_cmd) = self.imp().sessions_box.active_text() {
            // In case of manual entry, the ID should be missing
            debug!(
                "Retrieved session command '{}' through manual entry",
                manual_cmd
            );
            if let Some(cmd) = shlex::split(manual_cmd.as_str()) {
                (None, Some(cmd))
            } else {
                // This must be an invalid command
                self.display_error(
                    "Invalid session command",
                    &format!("Invalid session command: {}", manual_cmd),
                );
                (None, None)
            }
        } else {
            // This shouldn't happen, since we have an entry within the sessions box
            let username = if let Some(username) = self.get_current_username() {
                username
            } else {
                // This shouldn't happen, because a session should've been created with a username
                unimplemented!("Trying to create session without a username");
            };
            warn!(
                "No entry found; using default login shell of user: {}",
                username
            );
            if let Some(cmd) = self.imp().sys_util.get_shells().get(username.as_str()) {
                (None, Some(cmd.clone()))
            } else {
                // No login shell exists
                let error_msg = "No session or login shell found";
                self.display_error(error_msg, error_msg);
                (None, None)
            }
        }
    }

    /// Start the session for the selected user
    fn start_session(&self) {
        // Get the session command
        let (session, cmd) = if let (session, Some(cmd)) = self.get_current_session_cmd() {
            (session, cmd)
        } else {
            // Error handling should be inside `get_current_session_cmd`, so simply return
            return;
        };

        // Start the session
        let response = self
            .imp()
            .greetd_client
            .borrow_mut()
            .start_session(cmd)
            .unwrap_or_else(|err| panic!("Failed to start session: {}", err));

        match response {
            Response::Success => {
                info!("Session successfully started");
                if let Some(username) = self.get_current_username() {
                    self.imp().cache.borrow_mut().set_last_user(&username);
                    if let Some(session) = session {
                        self.imp()
                            .cache
                            .borrow_mut()
                            .set_last_session(&username, &session);
                    }
                    debug!("Updated cache with current user: {}", username);
                }

                info!("Saving cache to disk");
                if let Err(err) = self.imp().cache.borrow_mut().save() {
                    error!("Error saving cache to disk: {}", err);
                }

                self.imp().message_label.set_text("Logging in...");
            }

            // The client should raise an `unimplemented!`, so ignore it
            Response::AuthMessage { .. } => (),

            Response::Error { description, .. } => {
                self.display_error(
                    "Failed to start session",
                    &format!("Failed to start session; error: {}", description),
                );
            }
        }
    }

    /// Show an error message to the user
    fn display_error(&self, display_text: &str, log_text: &str) {
        self.imp().message_label.set_text(display_text);
        error!("{}", log_text);

        let (sender, receiver) = MainContext::channel(PRIORITY_LOW);

        // Set a timer in a separate thread that signals the main thread to reset the displayed
        // message, so as to not block the GUI
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(ERROR_MSG_CLEAR_DELAY));
            if sender.send(()).is_err() {
                warn!("Couldn't reset error message");
            };
        });

        // Resets the displayed message after getting the signal to do so
        receiver.attach(
            None,
            glib::clone!(@weak self as gui => @default-return Continue(false), move |_| {
                gui.imp().message_label.set_text(DEFAULT_MSG); Continue(false)
            }),
        );
    }
}
