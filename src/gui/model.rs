// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// SPDX-FileCopyrightText: 2021 Maximilian Moser <maximilian.moser@tuwien.ac.at>
//
// SPDX-License-Identifier: MIT

//! The main logic for the greeter

use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use greetd_ipc::{AuthMessageType, ErrorType, Response};
use relm4::{
    gtk::{
        gdk::{Display, Monitor},
        prelude::*,
    },
    AsyncComponentSender,
};
use tokio::{sync::Mutex, time::sleep};
use tracing::{debug, error, info, instrument, warn};

use crate::cache::Cache;
use crate::client::{AuthStatus, GreetdClient};
use crate::config::Config;
use crate::sysutil::SysUtil;

use super::messages::{CommandMsg, UserSessInfo};

const ERROR_MSG_CLEAR_DELAY: u64 = 5;

#[derive(PartialEq)]
pub(super) enum InputMode {
    None,
    Secret,
    Visible,
}

// Fields only set by the model, that are meant to be read only by the widgets
#[tracker::track]
pub(super) struct Updates {
    /// Message to be shown to the user
    pub(super) message: String,
    /// Error message to be shown to the user below the prompt
    pub(super) error: Option<String>,
    /// Text in the password field
    pub(super) input: String,
    /// Whether the username is being entered manually
    pub(super) manual_user_mode: bool,
    /// Whether the session is being entered manually
    pub(super) manual_sess_mode: bool,
    /// Input prompt sent by greetd for text input
    pub(super) input_prompt: String,
    /// Whether the user is currently entering a secret, something visible or nothing
    pub(super) input_mode: InputMode,
    /// ID of the active session
    pub(super) active_session_id: Option<String>,
    /// Time that is displayed
    pub(super) time: String,
    /// Monitor where the window is displayed
    pub(super) monitor: Option<Monitor>,
}

impl Updates {
    pub(super) fn is_input(&self) -> bool {
        self.input_mode != InputMode::None
    }
}

/// Capitalize the first letter of the string.
fn capitalize(string: &str) -> String {
    string[0..1].to_uppercase() + &string[1..]
}

/// Greeter model that holds its state
pub struct Greeter {
    /// Client to communicate with greetd
    pub(super) greetd_client: Arc<Mutex<GreetdClient>>,
    /// System utility to get available users and sessions
    pub(super) sys_util: SysUtil,
    /// The cache that persists between logins
    pub(super) cache: Cache,
    /// The config for this greeter
    pub(super) config: Config,
    /// Session info set after pressing login
    pub(super) sess_info: Option<UserSessInfo>,
    /// The updates from the model that are read by the view
    pub(super) updates: Updates,
}

impl Greeter {
    pub(super) async fn new(config_path: &Path) -> Self {
        let config = Config::new(config_path);

        let updates = Updates {
            message: config.get_default_message(),
            error: None,
            input: String::new(),
            manual_user_mode: false,
            manual_sess_mode: false,
            input_mode: InputMode::None,
            input_prompt: String::new(),
            active_session_id: None,
            tracker: 0,
            time: "".to_string(),
            monitor: None,
        };
        let greetd_client = Arc::new(Mutex::new(
            GreetdClient::new()
                .await
                .expect("Couldn't initialize greetd client"),
        ));
        Self {
            greetd_client,
            sys_util: SysUtil::new().expect("Couldn't read available users and sessions"),
            cache: Cache::new(),
            sess_info: None,
            config,
            updates,
        }
    }

    /// Make the greeter full screen over the first monitor.
    #[instrument(skip(self, sender))]
    pub(super) fn choose_monitor(
        &mut self,
        display_name: &str,
        sender: &AsyncComponentSender<Self>,
    ) {
        let display = match Display::open(display_name) {
            Some(display) => display,
            None => {
                error!("Couldn't get display with name: {display_name}");
                return;
            }
        };

        let mut chosen_monitor = None;
        for monitor in display
            .monitors()
            .into_iter()
            .filter_map(|item| {
                item.ok()
                    .and_then(|object| object.downcast::<Monitor>().ok())
            })
            .filter(Monitor::is_valid)
        {
            debug!("Found monitor: {monitor}");
            let sender = sender.clone();
            monitor.connect_invalidate(move |monitor| {
                let display_name = monitor.display().name();
                sender.oneshot_command(async move { CommandMsg::MonitorRemoved(display_name) })
            });
            if chosen_monitor.is_none() {
                // Choose the first monitor.
                chosen_monitor = Some(monitor);
            }
        }

        self.updates.set_monitor(chosen_monitor);
    }

    /// Run a command and log any errors in a background thread.
    fn run_cmd(command: &[String], sender: &AsyncComponentSender<Self>) {
        let mut process = Command::new(&command[0]);
        process.args(command[1..].iter());
        // Run the command and check its output in a separate thread, so as to not block the GUI.
        sender.spawn_command(move |_| match process.output() {
            Ok(output) => {
                if !output.status.success() {
                    if let Ok(err) = std::str::from_utf8(&output.stderr) {
                        error!("Failed to launch command: {err}")
                    } else {
                        error!("Failed to launch command: {:?}", output.stderr)
                    }
                }
            }
            Err(err) => error!("Failed to launch command: {err}"),
        });
    }

    /// Event handler for clicking the "Reboot" button
    ///
    /// This reboots the PC.
    #[instrument(skip_all)]
    pub(super) fn reboot_click_handler(&self, sender: &AsyncComponentSender<Self>) {
        info!("Rebooting");
        Self::run_cmd(&self.config.get_sys_commands().reboot, sender);
    }

    /// Event handler for clicking the "Power-Off" button
    ///
    /// This shuts down the PC.
    #[instrument(skip_all)]
    pub(super) fn poweroff_click_handler(&self, sender: &AsyncComponentSender<Self>) {
        info!("Shutting down");
        Self::run_cmd(&self.config.get_sys_commands().poweroff, sender);
    }

    /// Event handler for clicking the "Cancel" button
    ///
    /// This cancels the created session and goes back to the user/session chooser.
    #[instrument(skip_all)]
    pub(super) async fn cancel_click_handler(&mut self) {
        if let Err(err) = self.greetd_client.lock().await.cancel_session().await {
            warn!("Couldn't cancel greetd session: {err}");
        };
        self.updates.set_input(String::new());
        self.updates.set_input_mode(InputMode::None);
        self.updates.set_message(self.config.get_default_message())
    }

    /// Create a greetd session, i.e. start a login attempt for the current user.
    async fn create_session(&mut self, sender: &AsyncComponentSender<Self>) {
        let username = if let Some(username) = self.get_current_username() {
            username
        } else {
            // No username found (which shouldn't happen), so we can't create the session.
            return;
        };

        // Before trying to create a session, check if the session command (if manually entered) is
        // valid.
        if self.updates.manual_sess_mode {
            let info = self.sess_info.as_ref().expect("No session info set yet");
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
            .lock()
            .await
            .create_session(&username)
            .await
            .unwrap_or_else(|err| {
                panic!("Failed to create session for username '{username}': {err}",)
            });

        self.handle_greetd_response(sender, response).await;
    }

    /// This function handles a greetd response as follows:
    /// - if the response indicates authentication success, start the session
    /// - if the response is an authentication message:
    ///     - for info and error messages (no input request), display/log the text and send an empty authentication response to greetd.
    ///       This allows for immediate greetd updates when using authentication procedures that don't use text input.
    ///       Also reset input mode to `None`
    ///     - for input requests (visible/secret), set the input mode accordingly and return
    /// - if the response is an error, display it and return
    ///
    /// This way of handling responses allows for composite authentication procedures, e.g.:
    /// 1. Fingerprint
    /// 2. Password
    pub(super) async fn handle_greetd_response(
        &mut self,
        sender: &AsyncComponentSender<Self>,
        response: Response,
    ) {
        match response {
            Response::Success => {
                // Authentication was successful and the session may be started.
                // This may happen on the first request, in which case logging in
                // as the given user requires no authentication.
                info!("Successfully logged in; starting session");
                self.start_session(sender).await;
                return;
            }
            Response::AuthMessage {
                auth_message,
                auth_message_type,
            } => {
                match auth_message_type {
                    AuthMessageType::Secret => {
                        // Greetd has requested input that should be hidden
                        // e.g.: a password
                        info!("greetd asks for a secret auth input: {auth_message}");
                        self.updates.set_input_mode(InputMode::Secret);
                        self.updates.set_input(String::new());
                        self.updates
                            .set_input_prompt(auth_message.trim_end().to_string());
                        return;
                    }
                    AuthMessageType::Visible => {
                        // Greetd has requested input that need not be hidden
                        info!("greetd asks for a visible auth input: {auth_message}");
                        self.updates.set_input_mode(InputMode::Visible);
                        self.updates.set_input(String::new());
                        self.updates
                            .set_input_prompt(auth_message.trim_end().to_string());
                        return;
                    }
                    AuthMessageType::Info => {
                        // Greetd has sent an info message that should be displayed
                        // e.g.: asking for a fingerprint
                        info!("greetd sent an info: {auth_message}");
                        self.updates.set_input_mode(InputMode::None);
                        self.updates.set_message(auth_message);
                    }
                    AuthMessageType::Error => {
                        // Greetd has sent an error message that should be displayed and logged
                        self.updates.set_input_mode(InputMode::None);
                        // Reset outdated info message, if any
                        self.updates.set_message(self.config.get_default_message());
                        self.display_error(
                            sender,
                            &capitalize(&auth_message),
                            &format!("Authentication message error from greetd: {auth_message}"),
                        );
                    }
                }
            }
            Response::Error {
                description,
                error_type,
            } => {
                // some general response error. This can be an authentication failure or a general error
                self.display_error(
                    sender,
                    &format!("Login failed: {}", capitalize(&description)),
                    &format!("Error from greetd: {description}"),
                );

                // In case this is an authentication error (e.g. wrong password), the session should be cancelled.
                if let ErrorType::AuthError = error_type {
                    self.cancel_click_handler().await
                }
                return;
            }
        }

        debug!("Sending empty auth response to greetd");
        let client = Arc::clone(&self.greetd_client);
        sender.oneshot_command(async move {
            debug!("Sending empty auth response to greetd");
            let response = client
                .lock()
                .await
                .send_auth_response(None)
                .await
                .unwrap_or_else(|err| panic!("Failed to respond to greetd: {err}"));
            CommandMsg::HandleGreetdResponse(response)
        });
    }

    /// Event handler for selecting a different username in the `ComboBoxText`
    ///
    /// This changes the session in the combo box according to the last used session of the current user.
    #[instrument(skip_all)]
    pub(super) fn user_change_handler(&mut self) {
        let username = if let Some(username) = self.get_current_username() {
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
    pub(super) async fn login_click_handler(
        &mut self,
        sender: &AsyncComponentSender<Self>,
        input: String,
    ) {
        // Check if a password is needed. If not, then directly start the session.
        let auth_status = self.greetd_client.lock().await.get_auth_status().clone();
        match auth_status {
            AuthStatus::Done => {
                // No password is needed, but the session should've been already started by
                // `create_session`.
                warn!("No password needed for current user, but session not already started");
                self.start_session(sender).await;
            }
            AuthStatus::InProgress => {
                self.send_input(sender, input).await;
            }
            AuthStatus::NotStarted => {
                self.create_session(sender).await;
            }
        };
    }

    /// Send the entered input for logging in.
    async fn send_input(&mut self, sender: &AsyncComponentSender<Self>, input: String) {
        // Reset the password field, for convenience when the user has to re-enter a password.
        self.updates.set_input(String::new());

        // Send the password, as authentication for the current user.
        let resp = self
            .greetd_client
            .lock()
            .await
            .send_auth_response(Some(input))
            .await
            .unwrap_or_else(|err| panic!("Failed to send input: {err}"));

        self.handle_greetd_response(sender, resp).await;
    }

    /// Get the currently selected username.
    fn get_current_username(&self) -> Option<String> {
        let info = self.sess_info.as_ref().expect("No session info set yet");
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
        sender: &AsyncComponentSender<Self>,
    ) -> (Option<String>, Option<Vec<String>>) {
        let info = self.sess_info.as_ref().expect("No session info set yet");
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
            let username = if let Some(username) = self.get_current_username() {
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
    async fn start_session(&mut self, sender: &AsyncComponentSender<Self>) {
        // Get the session command.
        let (session, cmd) = if let (session, Some(cmd)) = self.get_current_session_cmd(sender) {
            (session, cmd)
        } else {
            // Error handling should be inside `get_current_session_cmd`, so simply return.
            return;
        };

        // Generate env string that will be passed to greetd when starting the session
        let env = self.config.get_env();
        let mut environment = Vec::with_capacity(env.len());
        for (k, v) in env {
            environment.push(format!("{}={}", k, v));
        }

        if let Some(username) = self.get_current_username() {
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

        // Start the session.
        let response = self
            .greetd_client
            .lock()
            .await
            .start_session(cmd, environment)
            .await
            .unwrap_or_else(|err| panic!("Failed to start session: {err}"));

        match response {
            Response::Success => {
                info!("Session successfully started");
                std::process::exit(0);
            }

            Response::AuthMessage { .. } => unimplemented!(),

            Response::Error { description, .. } => {
                self.cancel_click_handler().await;
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
        sender: &AsyncComponentSender<Self>,
        display_text: &str,
        log_text: &str,
    ) {
        self.updates.set_error(Some(display_text.to_string()));
        error!("{log_text}");

        sender.oneshot_command(async move {
            sleep(Duration::from_secs(ERROR_MSG_CLEAR_DELAY)).await;
            CommandMsg::ClearErr
        });
    }
}

impl Drop for Greeter {
    fn drop(&mut self) {
        // Cancel any created session, just to be safe.
        let client = Arc::clone(&self.greetd_client);
        tokio::spawn(async move {
            client
                .lock()
                .await
                .cancel_session()
                .await
                .expect("Couldn't cancel session on exit.")
        });
    }
}
