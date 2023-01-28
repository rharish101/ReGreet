// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Setup for using the greeter as a Relm4 component
use chrono::Local;
use std::time::Duration;
use tracing::{debug, info, warn};

use gtk::prelude::*;
use relm4::{gtk, Component, ComponentParts, ComponentSender, Sender, ShutdownReceiver};

use super::messages::{CommandMsg, InputMsg, UserSessInfo};
use super::model::{Greeter, Updates, DEFAULT_MSG};
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
fn setup_datetime_display(sender: ComponentSender<Greeter>) {
    // Set a timer in a separate thread that signals the main thread to update the time, so as
    // not to block the GUI.
    sender.command(|sender: Sender<CommandMsg>, receiver: ShutdownReceiver| {
        receiver
            .register(async move {
                // Run it infinitely, since the clock always needs to stay updated.
                loop {
                    if sender.send(CommandMsg::UpdateTime).is_err() {
                        warn!("Couldn't update datetime");
                    };
                    std::thread::sleep(Duration::from_millis(DATETIME_UPDATE_DELAY));
                }
            })
            .drop_on_shutdown()
    });
}

#[relm4::component(pub)]
impl Component for Greeter {
    type Input = InputMsg;
    type Output = ();
    type Init = ();
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
                message_label {
                    #[track(model.updates.changed(Updates::message()))]
                    set_label: &model.updates.message
                },
                #[template_child]
                session_label {
                    #[track(model.updates.changed(Updates::password_mode()))]
                    set_sensitive: !model.updates.password_mode,
                    #[track(model.updates.changed(Updates::password_mode()))]
                    set_visible: !model.updates.password_mode,
                },
                #[template_child]
                usernames_box {
                    #[track(model.updates.changed(Updates::password_mode()))]
                    set_sensitive: !model.updates.password_mode,
                    connect_changed[sender, sessions_box] => move |this| sender.input(
                        Self::Input::UserChanged(
                            UserSessInfo::extract(this, &sessions_box)
                        )
                    ),
                },
                #[template_child]
                sessions_box {
                    #[track(model.updates.changed(Updates::password_mode()))]
                    set_sensitive: !model.updates.password_mode,
                    #[track(model.updates.changed(Updates::password_mode()))]
                    set_visible: !model.updates.password_mode,
                    #[track(model.updates.changed(Updates::active_session_id()))]
                    set_active_id: model.updates.active_session_id.as_deref(),
                },
                #[template_child]
                password_label {
                    #[track(model.updates.changed(Updates::password_mode()))]
                    set_sensitive: model.updates.password_mode,
                    #[track(model.updates.changed(Updates::password_mode()))]
                    set_visible: model.updates.password_mode,
                },
                #[template_child]
                password_entry {
                    #[track(model.updates.changed(Updates::password_mode()))]
                    set_sensitive: model.updates.password_mode,
                    #[track(model.updates.changed(Updates::password_mode()))]
                    set_visible: model.updates.password_mode,
                    #[track(
                        model.updates.changed(Updates::password_mode())
                        && model.updates.password_mode
                    )]
                    grab_focus: (),
                    #[track(model.updates.changed(Updates::password()))]
                    set_text: &model.updates.password,
                    connect_activate[sender, usernames_box, sessions_box] => move |this| {
                        sender.input(InputMsg::Login {
                            password: this.text().to_string(),
                            info: UserSessInfo::extract(&usernames_box, &sessions_box),
                        })
                    }
                },
                #[template_child]
                cancel_button {
                    #[track(model.updates.changed(Updates::password_mode()))]
                    set_sensitive: model.updates.password_mode,
                    #[track(model.updates.changed(Updates::password_mode()))]
                    set_visible: model.updates.password_mode,
                    connect_clicked => Self::Input::Cancel,
                },
                #[template_child]
                login_button {
                    #[track(
                        model.updates.changed(Updates::password_mode())
                        && !model.updates.password_mode
                    )]
                    grab_focus: (),
                    connect_clicked[
                        sender, password_entry, usernames_box, sessions_box
                    ] => move |_| {
                        sender.input(InputMsg::Login {
                            password: password_entry.text().to_string(),
                            info: UserSessInfo::extract(&usernames_box, &sessions_box),
                        })
                    }
                },
                #[template_child]
                datetime_label { set_label: &Local::now().format(DATETIME_FMT).to_string() },
                #[template_child]
                reboot_button { connect_clicked => InputMsg::Reboot },
                #[template_child]
                poweroff_button { connect_clicked => InputMsg::PowerOff },
            }
        }
    }

    /// Initialize the greeter.
    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self::new();
        let widgets = view_output!();

        // Cancel any previous session, just in case someone started one.
        if let Err(err) = model.greetd_client.cancel_session() {
            warn!("Couldn't cancel greetd session: {err}");
        };

        setup_settings(&model, root);
        setup_users_sessions(&model, &widgets);
        setup_datetime_display(sender);

        // Set the default behaviour of pressing the Return key to act like the login button.
        root.set_default_widget(Some(&widgets.ui.login_button));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        // Reset the tracker for update changes.
        self.updates.reset();

        match msg {
            Self::Input::Login { password, info } => {
                self.login_click_handler(&sender, password, &info)
            }
            Self::Input::Cancel => self.cancel_click_handler(),
            Self::Input::UserChanged(info) => self.user_change_handler(&info),
            Self::Input::Reboot => Self::reboot_click_handler(&sender),
            Self::Input::PowerOff => Self::poweroff_click_handler(&sender),
        }
    }

    /// Perform the requested changes when a background task sends a message.
    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            Self::CommandOutput::UpdateTime => widgets
                .ui
                .datetime_label
                .set_label(&Local::now().format(DATETIME_FMT).to_string()),
            Self::CommandOutput::ClearErr => widgets.ui.message_label.set_label(DEFAULT_MSG),
            Self::CommandOutput::Noop => (),
        };
    }
}
