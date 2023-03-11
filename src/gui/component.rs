// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Setup for using the greeter as a Relm4 component

use std::path::PathBuf;
use std::time::Duration;

use chrono::Local;
use tracing::{debug, info, warn};

use gtk::prelude::*;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use std::thread::sleep;

#[cfg(feature = "gtk4_8")]
use crate::config::BgFit;

use super::messages::{CommandMsg, InputMsg, UserSessInfo};
use super::model::{Greeter, InputMode, Updates};
use super::templates::Ui;

const DATETIME_FMT: &str = "%a %R";
const DATETIME_UPDATE_DELAY: u64 = 500;

/// Load GTK settings from the greeter config.
fn setup_settings(model: &Greeter, root: &gtk::ApplicationWindow) {
    let settings = root.settings();
    let config = if let Some(config) = model.config.get_gtk_settings() {
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
        debug!("Setting cursor theme: {cursor_theme}");
        settings.set_gtk_cursor_theme_name(config.cursor_theme_name.as_deref());
    };

    if let Some(font) = &config.font_name {
        debug!("Setting font: {font}");
        settings.set_gtk_font_name(config.font_name.as_deref());
    };

    if let Some(icon_theme) = &config.icon_theme_name {
        debug!("Setting icon theme: {icon_theme}");
        settings.set_gtk_icon_theme_name(config.icon_theme_name.as_deref());
    };

    if let Some(theme) = &config.theme_name {
        debug!("Setting theme: {theme}");
        settings.set_gtk_theme_name(config.theme_name.as_deref());
    };
}

/// Populate the user and session combo boxes with entries.
fn setup_users_sessions(model: &Greeter, widgets: &GreeterWidgets) {
    // The user that is shown during initial login
    let mut initial_username = None;

    // Populate the usernames combo box.
    for (user, username) in model.sys_util.get_users().iter() {
        debug!("Found user: {user}");
        if initial_username.is_none() {
            initial_username = Some(username.clone());
        }
        widgets.ui.usernames_box.append(Some(username), user);
    }

    // Populate the sessions combo box.
    for session in model.sys_util.get_sessions().keys() {
        debug!("Found session: {session}");
        widgets.ui.sessions_box.append(Some(session), session);
    }

    // If the last user is known, show their login initially.
    if let Some(last_user) = model.cache.get_last_user() {
        initial_username = Some(last_user.to_string());
    } else if let Some(user) = &initial_username {
        info!("Using first found user '{user}' as initial user");
    }

    // Set the user shown initially at login.
    if !widgets
        .ui
        .usernames_box
        .set_active_id(initial_username.as_deref())
    {
        if let Some(user) = initial_username {
            warn!("Couldn't find user '{user}' to set as the initial user");
        }
    }
}

/// Set up auto updation for the datetime label.
fn setup_datetime_display(sender: &ComponentSender<Greeter>) {
    // Set a timer in a separate thread that signals the main thread to update the time, so as to
    // not block the GUI.
    sender.spawn_command(|sender| {
        // Run it infinitely, since the clock always needs to stay updated.
        loop {
            if sender.send(CommandMsg::UpdateTime).is_err() {
                warn!("Couldn't update datetime");
            };
            sleep(Duration::from_millis(DATETIME_UPDATE_DELAY));
        }
    });
}

/// The info required to initialize the greeter
pub struct GreeterInit {
    pub config_path: PathBuf,
    pub css_path: PathBuf,
}

#[relm4::component(pub)]
impl Component for Greeter {
    type Input = InputMsg;
    type Output = ();
    type Init = GreeterInit;
    type CommandOutput = CommandMsg;

    view! {
        // The `view!` macro needs a proper widget, not a template, as the root.
        gtk::ApplicationWindow {
            set_fullscreened: true,
            set_visible: true,

            // Name the UI widget, otherwise the inner children cannot be accessed by name.
            #[name = "ui"]
            #[template]
            Ui {
                #[template_child]
                background { set_filename: model.config.get_background().clone() },
                #[template_child]
                datetime_label {
                    #[track(model.updates.changed(Updates::time()))]
                    set_label: &model.updates.time
                },

                #[template_child]
                message_label {
                    #[track(model.updates.changed(Updates::message()))]
                    set_label: &model.updates.message,
                },
                #[template_child]
                session_label {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: !model.updates.is_input(),
                },
                #[template_child]
                usernames_box {
                    #[track(
                        model.updates.changed(Updates::manual_user_mode())
                        || model.updates.changed(Updates::input_mode())
                    )]
                    set_sensitive: !model.updates.manual_user_mode && !model.updates.is_input(),
                    #[track(model.updates.changed(Updates::manual_user_mode()))]
                    set_visible: !model.updates.manual_user_mode,
                    connect_changed[
                        sender, username_entry, sessions_box, session_entry
                    ] => move |this| sender.input(
                        Self::Input::UserChanged(
                            UserSessInfo::extract(this, &username_entry, &sessions_box, &session_entry)
                        )
                    ),
                },
                #[template_child]
                username_entry {
                    #[track(
                        model.updates.changed(Updates::manual_user_mode())
                        || model.updates.changed(Updates::input_mode())
                    )]
                    set_sensitive: model.updates.manual_user_mode && !model.updates.is_input(),
                    #[track(model.updates.changed(Updates::manual_user_mode()))]
                    set_visible: model.updates.manual_user_mode,
                },
                #[template_child]
                sessions_box {
                    #[track(
                        model.updates.changed(Updates::manual_sess_mode())
                        || model.updates.changed(Updates::input_mode())
                    )]
                    set_visible: !model.updates.manual_sess_mode && !model.updates.is_input(),
                    #[track(model.updates.changed(Updates::active_session_id()))]
                    set_active_id: model.updates.active_session_id.as_deref(),
                },
                #[template_child]
                session_entry {
                    #[track(
                        model.updates.changed(Updates::manual_sess_mode())
                        || model.updates.changed(Updates::input_mode())
                    )]
                    set_visible: model.updates.manual_sess_mode && !model.updates.is_input(),
                },
                #[template_child]
                input_label {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: model.updates.is_input(),
                    #[track(model.updates.changed(Updates::input_prompt()))]
                    set_label: &model.updates.input_prompt,
                },
                #[template_child]
                secret_entry {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: model.updates.input_mode == InputMode::Secret,
                    #[track(
                        model.updates.changed(Updates::input_mode())
                        && model.updates.is_input()
                    )]
                    grab_focus: (),
                    #[track(model.updates.changed(Updates::input()))]
                    set_text: &model.updates.input,
                    connect_activate[
                        sender, usernames_box, username_entry, sessions_box, session_entry
                    ] => move |this| {
                        sender.input(Self::Input::Login {
                            input: this.text().to_string(),
                            info: UserSessInfo::extract(
                                &usernames_box, &username_entry, &sessions_box, &session_entry
                            ),
                        })
                    }
                },
                #[template_child]
                visible_entry {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: model.updates.input_mode == InputMode::Visible,
                    #[track(
                        model.updates.changed(Updates::input_mode())
                        && model.updates.is_input()
                    )]
                    grab_focus: (),
                    #[track(model.updates.changed(Updates::input()))]
                    set_text: &model.updates.input,
                    connect_activate[
                        sender, usernames_box, username_entry, sessions_box, session_entry
                    ] => move |this| {
                        sender.input(Self::Input::Login {
                            input: this.text().to_string(),
                            info: UserSessInfo::extract(
                                &usernames_box, &username_entry, &sessions_box, &session_entry
                            ),
                        })
                    }
                },
                #[template_child]
                user_toggle {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_sensitive: !model.updates.is_input(),
                    connect_clicked => Self::Input::ToggleManualUser,
                },
                #[template_child]
                sess_toggle {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: !model.updates.is_input(),
                    connect_clicked => Self::Input::ToggleManualSess,
                },
                #[template_child]
                cancel_button {
                    #[track(model.updates.changed(Updates::input_mode()))]
                    set_visible: model.updates.is_input(),
                    connect_clicked => Self::Input::Cancel,
                },
                #[template_child]
                login_button {
                    #[track(
                        model.updates.changed(Updates::input_mode())
                        && !model.updates.is_input()
                    )]
                    grab_focus: (),
                    connect_clicked[
                        sender,
                        secret_entry,
                        visible_entry,
                        usernames_box,
                        username_entry,
                        sessions_box,
                        session_entry,
                    ] => move |_| {
                        sender.input(Self::Input::Login {
                            input: match model.updates.input_mode {
                                InputMode::Secret => secret_entry.text().to_string(),
                                InputMode::Visible => visible_entry.text().to_string(),
                                InputMode::None => String::new(),
                            },
                            info: UserSessInfo::extract(
                                &usernames_box, &username_entry, &sessions_box, &session_entry
                            ),
                        })
                    }
                },
                #[template_child]
                reboot_button { connect_clicked => Self::Input::Reboot },
                #[template_child]
                poweroff_button { connect_clicked => Self::Input::PowerOff },
            }
        }
    }

    /// Initialize the greeter.
    fn init(
        input: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self::new(&input.config_path);
        let widgets = view_output!();

        // cfg directives don't work inside Relm4 view! macro.
        #[cfg(feature = "gtk4_8")]
        if let Some(fit) = model.config.get_background_fit() {
            widgets.ui.background.set_content_fit(match fit {
                BgFit::Fill => gtk4::ContentFit::Fill,
                BgFit::Contain => gtk4::ContentFit::Contain,
                BgFit::Cover => gtk4::ContentFit::Cover,
                BgFit::ScaleDown => gtk4::ContentFit::ScaleDown,
            });
        };

        // Cancel any previous session, just in case someone started one.
        if let Err(err) = model.greetd_client.lock().unwrap().cancel_session() {
            warn!("Couldn't cancel greetd session: {err}");
        };

        setup_settings(&model, root);
        setup_users_sessions(&model, &widgets);
        setup_datetime_display(&sender);
        if input.css_path.exists() {
            debug!("Loading custom CSS from file: {}", input.css_path.display());
            relm4::set_global_css_from_file(input.css_path);
        };

        // Set the default behaviour of pressing the Return key to act like the login button.
        root.set_default_widget(Some(&widgets.ui.login_button));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        // Reset the tracker for update changes.
        self.updates.reset();

        match msg {
            Self::Input::Login { input, info } => {
                self.sess_info = Some(info);
                self.login_click_handler(&sender, input);
            }
            Self::Input::Cancel => self.cancel_click_handler(),
            Self::Input::UserChanged(info) => {
                self.sess_info = Some(info);
                self.user_change_handler();
            }
            Self::Input::ToggleManualUser => self
                .updates
                .set_manual_user_mode(!self.updates.manual_user_mode),
            Self::Input::ToggleManualSess => self
                .updates
                .set_manual_sess_mode(!self.updates.manual_sess_mode),
            Self::Input::Reboot => Self::reboot_click_handler(&sender),
            Self::Input::PowerOff => Self::poweroff_click_handler(&sender),
        }
    }

    /// Perform the requested changes when a background task sends a message.
    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            Self::CommandOutput::UpdateTime => self
                .updates
                .set_time(Local::now().format(DATETIME_FMT).to_string()),
            Self::CommandOutput::ClearErr => self.updates.set_error(None), // TODO see if this works at all
            Self::CommandOutput::HandleGreetdResponse(response) => {
                self.handle_greetd_response(&sender, response)
            }
        };
    }
}
