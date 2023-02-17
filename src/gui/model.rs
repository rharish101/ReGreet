// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// SPDX-FileCopyrightText: 2021 Maximilian Moser <maximilian.moser@tuwien.ac.at>
//
// SPDX-License-Identifier: MIT

//! The main logic for the greeter

use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use greetd_ipc::{ErrorType as GreetdErrorType, Response};
use relm4::ComponentSender;
use tracing::{debug, error, info, instrument, warn};

use crate::cache::Cache;
use crate::client::{AuthStatus, GreetdClient};
use crate::common::capitalize;
use crate::config::Config;
use crate::sysutil::SysUtil;

use super::messages::{CommandMsg, UserSessInfo};

pub(super) const DEFAULT_MSG: &str = "Welcome back!";
const ERROR_MSG_CLEAR_DELAY: u64 = 5;

// Fields only set by the model, that are meant to be read only by the widgets
#[tracker::track]
pub(super) struct Updates {
    /// Message to be shown to the user
    pub(super) message: String,
    /// Text in the password field
    pub(super) password: String,
    /// Whether the username is being entered manually
    pub(super) manual_user_mode: bool,
    /// Whether the session is being entered manually
    pub(super) manual_sess_mode: bool,
    /// Whether the password is being entered
    pub(super) password_mode: bool,
    /// ID of the active session
    pub(super) active_session_id: Option<String>,
}

/// Greeter model that holds its state
pub struct Greeter {
    /// Client to communicate with greetd
    pub(super) greetd_client: GreetdClient,
    /// System utility to get available users and sessions
    pub(super) sys_util: SysUtil,
    /// The cache that persists between logins
    pub(super) cache: Cache,
    /// The config for this greeter
    pub(super) config: Config,
    /// The updates from the model that are read by the view
    pub(super) updates: Updates,
}

impl Greeter {
    pub(super) fn new(config_path: &Path) -> Self {
        let updates = Updates {
            message: DEFAULT_MSG.to_string(),
            password: String::new(),
            manual_user_mode: false,
            manual_sess_mode: false,
            password_mode: false,
            active_session_id: None,
            tracker: 0,
        };
        Self {
            greetd_client: GreetdClient::new().expect("Couldn't initialize greetd client"),
            sys_util: SysUtil::new().expect("Couldn't read available users and sessions"),
            cache: Cache::new(),
            config: Config::new(config_path),
            updates,
        }
    }

    /// Run a systemctl command and log any errors in a background thread.
    fn systemctl_cmd(command: String, sender: &ComponentSender<Self>) {
        // Run the command and check its output in a separate thread, so as to not block the GUI.
        sender.spawn_command(
            move |_| match Command::new("systemctl").arg(&command).output() {
                Ok(output) => {
                    if !output.status.success() {
                        if let Ok(err) = std::str::from_utf8(&output.stderr) {
                            error!("Failed to {command}: {err}")
                        } else {
                            error!("Failed to {command}: {:?}", output.stderr)
                        }
                    }
                }
                Err(err) => error!("Failed to launch {command}: {err}"),
            },
        );
    }

    /// Event handler for clicking the "Reboot" button
    ///
    /// This reboots the PC.
    #[instrument(skip_all)]
    pub(super) fn reboot_click_handler(sender: &ComponentSender<Self>) {
        info!("Rebooting");
        Self::systemctl_cmd("reboot".to_string(), sender);
    }

    /// Event handler for clicking the "Power-Off" button
    ///
    /// This shuts down the PC.
    #[instrument(skip_all)]
    pub(super) fn poweroff_click_handler(sender: &ComponentSender<Self>) {
        info!("Shutting down");
        Self::systemctl_cmd("poweroff".to_string(), sender);
    }

    /// Event handler for clicking the "Cancel" button
    ///
    /// This cancels the created session and goes back to the user/session chooser.
    #[instrument(skip_all)]
    pub(super) fn cancel_click_handler(&mut self) {
        if let Err(err) = self.greetd_client.cancel_session() {
            warn!("Couldn't cancel greetd session: {err}");
        };
        self.updates.set_password(String::new());
        self.updates.set_password_mode(false);
    }

    /// Create a greetd session, i.e. start a login attempt for the current user.
    fn create_session(&mut self, sender: &ComponentSender<Self>, info: &UserSessInfo) {
        let username = if let Some(username) = self.get_current_username(info) {
            username
        } else {
            // No username found (which shouldn't happen), so we can't create the session.
            return;
        };

        // Before trying to create a session, check if the session command (if manually entered) is
        // valid.
        if self.updates.manual_sess_mode {
            if shlex::split(info.sess_text.as_str()).is_none() {
                // This must be an invalid command.
                self.display_error(
                    sender,
                    "Invalid session command",
                    &format!("Invalid session command: {}", info.sess_text),
                );
                return;
            };
            debug!("Manually entered session command is parsable");
        };

        info!("Creating session for user: {username}");

        // Create a session for the current user.
        let response = self
            .greetd_client
            .create_session(&username)
            .unwrap_or_else(|err| {
                panic!("Failed to create session for username '{username}': {err}",)
            });

        match response {
            Response::Success => {
                // No password is needed, so directly start the session.
                info!("No password needed for current user");
                self.start_session(sender, info);
            }
            Response::AuthMessage { auth_message, .. } => {
                if auth_message.to_lowercase().contains("password") {
                    // Show the password field, because a password is needed.
                    self.updates.set_password_mode(true);
                    self.updates.set_password(String::new());
                } else {
                    // greetd has requested something other than the password, so just display it
                    // to the user and let them figure it out.
                    self.display_error(
                        sender,
                        &capitalize(&auth_message),
                        &format!(
                            "Expected password request, but greetd requested: {auth_message}",
                        ),
                    );
                };
            }
            Response::Error { description, .. } => {
                self.display_error(
                    sender,
                    &capitalize(&description),
                    &format!("Message from greetd: {description}"),
                );
            }
        };
    }

    /// Event handler for selecting a different username in the `ComboBoxText`
    ///
    /// This changes the session in the combo box according to the last used session of the current user.
    #[instrument(skip_all)]
    pub(super) fn user_change_handler(&mut self, info: &UserSessInfo) {
        let username = if let Some(username) = self.get_current_username(info) {
            username
        } else {
            // No username found (which shouldn't happen), so we can't change the session.
            return;
        };

        if let Some(last_session) = self.cache.get_last_session(&username) {
            // Set the last session used by this user in the session combo box.
            self.updates
                .set_active_session_id(Some(last_session.to_string()));
        } else {
            // Last session not found, so skip changing the session.
            info!("Last session for user '{username}' missing");
        };
    }

    /// Event handler for clicking the "Login" button
    ///
    /// This does one of the following, depending of the state of authentication:
    ///     - Begins a login attempt for the given user
    ///     - Submits the entered password for logging in and starts the session
    #[instrument(skip_all)]
    pub(super) fn login_click_handler(
        &mut self,
        sender: &ComponentSender<Self>,
        password: String,
        info: &UserSessInfo,
    ) {
        // Check if a password is needed. If not, then directly start the session.
        let auth_status = self.greetd_client.get_auth_status().clone();
        match auth_status {
            AuthStatus::Done => {
                // No password is needed, but the session should've been already started by
                // `create_session`.
                warn!("No password needed for current user, but session not already started");
                self.start_session(sender, info);
            }
            AuthStatus::InProgress => {
                self.send_password(sender, password, info);
            }
            AuthStatus::NotStarted => {
                self.create_session(sender, info);
            }
        };
    }

    /// Send the entered password for logging in.
    fn send_password(
        &mut self,
        sender: &ComponentSender<Self>,
        password: String,
        info: &UserSessInfo,
    ) {
        // Reset the password field, for convenience when the user has to re-enter a password.
        self.updates.set_password(String::new());

        // Send the password, as authentication for the current user.
        let resp = self
            .greetd_client
            .send_password(Some(password))
            .unwrap_or_else(|err| panic!("Failed to send password: {err}"));

        match resp {
            Response::Success => {
                info!("Successfully logged in; starting session");
                self.start_session(sender, info);
            }
            // The client should raise an `unimplemented!`, so ignore it.
            Response::AuthMessage { .. } => (),
            Response::Error {
                error_type: GreetdErrorType::AuthError,
                ..
            } => {
                // The user most likely entered the wrong password.
                self.display_error(sender, "Login failed", "Login failed");
            }
            Response::Error {
                error_type: GreetdErrorType::Error,
                description,
            } => {
                self.display_error(
                    sender,
                    &capitalize(&description),
                    &format!("Message from greetd: {description}"),
                );
            }
        };
    }

    /// Get the currently selected username.
    fn get_current_username(&self, info: &UserSessInfo) -> Option<String> {
        if self.updates.manual_user_mode {
            debug!(
                "Retrieved username '{}' through manual entry",
                info.user_text
            );
            Some(info.user_text.to_string())
        } else if let Some(username) = &info.user_id {
            // Get the currently selected user's ID, which should be their username.
            debug!("Retrieved username '{username}' from options");
            Some(username.to_string())
        } else {
            error!("No username entered");
            None
        }
    }

    /// Get the currently selected session name (if available) and command.
    fn get_current_session_cmd(
        &mut self,
        sender: &ComponentSender<Self>,
        info: &UserSessInfo,
    ) -> (Option<String>, Option<Vec<String>>) {
        if self.updates.manual_sess_mode {
            debug!(
                "Retrieved session command '{}' through manual entry",
                info.sess_text
            );
            if let Some(cmd) = shlex::split(info.sess_text.as_str()) {
                (None, Some(cmd))
            } else {
                // This must be an invalid command.
                self.display_error(
                    sender,
                    "Invalid session command",
                    &format!("Invalid session command: {}", info.sess_text),
                );
                (None, None)
            }
        } else if let Some(session) = &info.sess_id {
            // Get the currently selected session.
            debug!("Retrieved current session: {session}");
            if let Some(cmd) = self.sys_util.get_sessions().get(session.as_str()) {
                (Some(session.to_string()), Some(cmd.clone()))
            } else {
                // Shouldn't happen, unless there are no sessions available.
                let error_msg = format!("Session '{session}' not found");
                self.display_error(sender, &error_msg, &error_msg);
                (None, None)
            }
        } else {
            let username = if let Some(username) = self.get_current_username(info) {
                username
            } else {
                // This shouldn't happen, because a session should've been created with a username.
                unimplemented!("Trying to create session without a username");
            };
            warn!("No entry found; using default login shell of user: {username}",);
            if let Some(cmd) = self.sys_util.get_shells().get(username.as_str()) {
                (None, Some(cmd.clone()))
            } else {
                // No login shell exists.
                let error_msg = "No session or login shell found";
                self.display_error(sender, error_msg, error_msg);
                (None, None)
            }
        }
    }

    /// Start the session for the selected user.
    fn start_session(&mut self, sender: &ComponentSender<Self>, info: &UserSessInfo) {
        // Get the session command.
        let (session, cmd) =
            if let (session, Some(cmd)) = self.get_current_session_cmd(sender, info) {
                (session, cmd)
            } else {
                // Error handling should be inside `get_current_session_cmd`, so simply return.
                return;
            };

        // Start the session.
        let response = self
            .greetd_client
            .start_session(cmd)
            .unwrap_or_else(|err| panic!("Failed to start session: {err}"));

        match response {
            Response::Success => {
                info!("Session successfully started");
                if let Some(username) = self.get_current_username(info) {
                    self.cache.set_last_user(&username);
                    if let Some(session) = session {
                        self.cache.set_last_session(&username, &session);
                    }
                    debug!("Updated cache with current user: {username}");
                }

                info!("Saving cache to disk");
                if let Err(err) = self.cache.save() {
                    error!("Error saving cache to disk: {err}");
                }

                self.updates.set_message("Logging in...".to_string());
            }

            // The client should raise an `unimplemented!`, so ignore it.
            Response::AuthMessage { .. } => (),

            Response::Error { description, .. } => {
                self.display_error(
                    sender,
                    "Failed to start session",
                    &format!("Failed to start session; error: {description}"),
                );
            }
        }
    }

    /// Show an error message to the user.
    fn display_error(
        &mut self,
        sender: &ComponentSender<Self>,
        display_text: &str,
        log_text: &str,
    ) {
        self.updates.set_message(display_text.to_string());
        error!("{log_text}");

        // Set a timer in a separate thread that signals the main thread to reset the displayed
        // message, so as to not block the GUI.
        sender.spawn_oneshot_command(|| {
            sleep(Duration::from_secs(ERROR_MSG_CLEAR_DELAY));
            CommandMsg::ClearErr
        });
    }
}
