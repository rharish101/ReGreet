//! The main GUI for the greeter
mod imp;

use std::process::Command;

use greetd_ipc::{ErrorType as GreetdErrorType, Response};
use gtk::{gio, glib, prelude::*, subclass::prelude::*, Application, Button};
use log::{debug, error, info, warn};

use crate::client::AuthStatus;
use crate::common::capitalize;

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
        // Setup the welcome message
        self.imp().message_label.set_text("Welcome Back!");

        self.setup_callbacks();
        // NOTE: This starts login for the last user
        self.setup_users_sessions();

        // If the selected user requires a password, (i.e the password entry is visible), focus the
        // password entry. Otherwise, focus the user selection box.
        if self.imp().password_entry.get_sensitive() {
            self.imp().password_entry.grab_focus()
        } else {
            self.imp().usernames_box.grab_focus()
        };

        // Make the window fullscreen
        self.fullscreen();
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
        self.imp().password_entry.connect_activate(
            glib::clone!(@weak self as gui => move |_| gui.login_click_handler()),
        );
        self.imp()
            .login_button
            .connect_clicked(glib::clone!(@weak self as gui => move |_| gui.login_click_handler()));

        // Set the default behaviour of pressing the Return key to act like the login button
        self.set_default_widget(Some(&self.imp().login_button.get()));
    }

    /// Populate the user and session combo boxes with entries
    fn setup_users_sessions(&self) {
        // The user that is shown during initial login
        let mut initial_user = None;

        // Populate the usernames combo box
        for username in self.imp().sys_util.get_users().keys() {
            if initial_user.is_none() {
                initial_user = Some(username.clone());
            }
            self.imp().usernames_box.append(Some(username), username);
        }

        // Populate the sessions combo box
        for session in self.imp().sys_util.get_sessions().keys() {
            self.imp().sessions_box.append(Some(session), session)
        }

        // If the last user is known, show their login initially
        if let Some(last_user) = self.imp().cache.borrow().get_last_user() {
            initial_user = Some(last_user.to_string());
        } else if let Some(user) = &initial_user {
            info!("Using first found user '{}' as initial user", user);
        }

        // NOTE: This should call `self.user_change_handler`
        if !self
            .imp()
            .usernames_box
            .set_active_id(initial_user.as_deref())
        {
            if let Some(user) = initial_user {
                warn!("Couldn't find user '{}' to set as the initial user", user);
            }
        }
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

    /// Event handler for selecting a different username in the ComboBox
    ///
    /// This creates a greetd session, i.e. starts a login attempt for the current user.
    fn user_change_handler(&self) {
        // Handle the case when the user is changed in the middle of authenticating another user
        if let AuthStatus::InProgress = self.imp().greetd_client.borrow().get_auth_status() {
            if let Err(err) = self.imp().greetd_client.borrow_mut().cancel_session() {
                warn!("Couldn't cancel greetd session: {}", err);
            };
        };

        // Get the current username
        let username = if let Some(username) = self.get_current_username() {
            username
        } else {
            // No username found (which shouldn't happend), so we can't start the session
            error!("No username selected");
            return;
        };

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
                // User doesn't need a password
                self.set_password_visibility(false);
            }
            Response::AuthMessage { auth_message, .. } => {
                // Reset the password field, because a password is needed
                self.set_password_visibility(true);
                self.imp().password_entry.set_text("");

                if !auth_message.to_lowercase().contains("password") {
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

        // Get the last session used by this user, and auto-select it in the session combo box
        if let Some(last_session) = self.imp().cache.borrow_mut().get_last_session(&username) {
            if !self.imp().sessions_box.set_active_id(Some(last_session)) {
                warn!(
                    "Last session '{}' for user '{}' missing",
                    last_session, username
                );
            }
        } else {
            // Last session not found, so use the default session
            let default_session = self.imp().config.get_default_session();
            if !self.imp().sessions_box.set_active_id(Some(default_session)) {
                warn!("Default session '{}' missing", default_session);
            }
        };
    }

    /// Event handler for clicking the "Login" button
    ///
    /// This submits the entered password for logging in.
    fn login_click_handler(&self) {
        // Check if a password is needed. If not, then directly start the session.
        match self.imp().greetd_client.borrow().get_auth_status() {
            AuthStatus::Done => {
                // No password is needed, so directly start session
                info!("No password needed for current user");
                self.start_session();
                return;
            }
            AuthStatus::InProgress => (),
            AuthStatus::NotStarted => {
                // Somehow, the session wasn't started, so do it here
                warn!("Session not created for the current user before login; creating it now");
                self.user_change_handler();
            }
        };

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
        // Get the currently selected user
        let user = if let Some(user) = self.imp().usernames_box.active_text() {
            user.to_string()
        } else {
            // No username selected, so return None
            return None;
        };
        debug!("Retrieved current user: {}", user);

        // Get the actual username of the currently selected user
        let username = if let Some(username) = self.imp().sys_util.get_users().get(&user) {
            username.to_string()
        } else {
            error!("Unknown user '{}' selected", user);
            user
        };

        debug!("Found username '{}' for current user", username);
        Some(username)
    }

    /// Show or hide the password entry field
    fn set_password_visibility(&self, visible: bool) {
        // Make the password field enabled and visible
        self.imp().password_entry.set_sensitive(visible);
        self.imp().password_entry.set_visible(visible);

        // Make the password label enabled and visible
        self.imp().password_label.set_sensitive(visible);
        self.imp().password_label.set_visible(visible);
    }

    /// Start the session for the selected user
    fn start_session(&self) {
        // Get the currently selected session
        let session = if let Some(session) = self.imp().sessions_box.active_text() {
            session.to_string()
        } else {
            // No session selected
            let default_session = self.imp().config.get_default_session().to_string();
            info!(
                "No session selected; using default session: {}",
                default_session
            );
            default_session
        };
        debug!("Retrieved current session: {}", session);

        // Get the session command
        let cmd = if let Some(cmd) = self.imp().sys_util.get_sessions().get(&session) {
            cmd
        } else {
            // Shouldn't happen, unless there are no sessions available
            let error_msg = format!("Session '{}' not found", session);
            self.display_error(
                &capitalize(&error_msg),
                &format!("Session '{}' not found", session),
            );
            return;
        };

        // Start the session
        let response = self
            .imp()
            .greetd_client
            .borrow_mut()
            .start_session(cmd.clone())
            .unwrap_or_else(|err| panic!("Failed to start session: {}", err));

        match response {
            Response::Success => {
                info!("Session successfully started");
                if let Some(username) = self.get_current_username() {
                    self.imp().cache.borrow_mut().set_last_user(&username);
                    self.imp()
                        .cache
                        .borrow_mut()
                        .set_last_session(&username, &session);
                    debug!("Updated cache with current user: {}", username);
                }

                debug!("Saving cache to disk");
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

    /// Show the message from greetd to the user
    fn display_error(&self, display_text: &str, log_text: &str) {
        self.imp().message_label.set_text(display_text);
        error!("{}", log_text);
    }
}
