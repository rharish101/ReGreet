// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Setup for using the greeter as a Relm4 component

use std::path::PathBuf;

use relm4::{
    component::{AsyncComponent, AsyncComponentParts},
    gtk::prelude::*,
    prelude::*,
    AsyncComponentSender,
};
use tracing::{debug, info, warn};

#[cfg(feature = "gtk4_8")]
use crate::config::BgFit;

use super::messages::{CommandMsg, InputMsg, UserSessInfo};
use super::model::{Greeter, InputMode, Updates};
use super::templates::Ui;

/// Load GTK settings from the greeter config.
fn setup_settings(model: &Greeter, root: &adw::ApplicationWindow) {
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

/// The info required to initialize the greeter
pub struct GreeterInit {
    pub config_path: PathBuf,
    pub css_path: PathBuf,
    pub demo: bool,
}

#[relm4::component(pub, async)]
impl AsyncComponent for Greeter {
    type Input = InputMsg;
    type Output = ();
    type Init = GreeterInit;
    type CommandOutput = CommandMsg;

    view! {
        // The `view!` macro needs a proper widget, not a template, as the root.
        #[name = "window"]
        adw::ApplicationWindow {
            set_visible: true,

            // Name the UI widget, otherwise the inner children cannot be accessed by name.
            #[name = "ui"]
            #[template]
            Ui {
                #[template_child]
                background { set_filename: model.config.get_background() },

                #[template_child]
                clock_frame {
                    model.clock.widget(),
                },

                #[template_child]
                panel_end {
                    append = model.power_menu.widget() {
                        set_halign: gtk::Align::End,
                        set_hexpand: false,
                    },
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
                        sender,
                        username_entry = ui.username_entry.clone(),
                        sessions_box = ui.sessions_box.clone(),
                        session_entry = ui.session_entry.clone(),
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
                        && model.updates.input_mode == InputMode::Secret
                    )]
                    grab_focus: (),
                    #[track(model.updates.changed(Updates::input()))]
                    set_text: &model.updates.input,
                    connect_activate[
                        sender,
                        usernames_box = ui.usernames_box.clone(),
                        username_entry = ui.username_entry.clone(),
                        sessions_box = ui.sessions_box.clone(),
                        session_entry = ui.session_entry.clone(),
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
                        && model.updates.input_mode == InputMode::Visible
                    )]
                    grab_focus: (),
                    #[track(model.updates.changed(Updates::input()))]
                    set_text: &model.updates.input,
                    connect_activate[
                        sender,
                        usernames_box = ui.usernames_box.clone(),
                        username_entry = ui.username_entry.clone(),
                        sessions_box = ui.sessions_box.clone(),
                        session_entry = ui.session_entry.clone(),
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
                        secret_entry = ui.secret_entry.clone(),
                        visible_entry = ui.visible_entry.clone(),
                        usernames_box = ui.usernames_box.clone(),
                        username_entry = ui.username_entry.clone(),
                        sessions_box = ui.sessions_box.clone(),
                        session_entry = ui.session_entry.clone(),
                    ] => move |_| {
                        sender.input(Self::Input::Login {
                            input: if secret_entry.is_visible() {
                                // This should correspond to `InputMode::Secret`.
                                secret_entry.text().to_string()
                            } else if EntryExt::is_visible(&visible_entry) {
                                // This should correspond to `InputMode::Visible`.
                                visible_entry.text().to_string()
                            } else {
                                // This should correspond to `InputMode::None`.
                                String::new()
                            },
                            info: UserSessInfo::extract(
                                &usernames_box, &username_entry, &sessions_box, &session_entry
                            ),
                        })
                    }
                },
                #[template_child]
                error_info {
                    #[track(model.updates.changed(Updates::error()))]
                    set_revealed: model.updates.error.is_some(),
                },
                #[template_child]
                error_label {
                    #[track(model.updates.changed(Updates::error()))]
                    set_label: model.updates.error.as_ref().unwrap_or(&"".to_string()),
                },
            }
        }
    }

    fn post_view() {
        if model.updates.changed(Updates::monitor()) {
            if let Some(monitor) = &model.updates.monitor {
                widgets.window.fullscreen_on_monitor(monitor);
                // For some reason, the GTK settings are reset when changing monitors, so re-apply them.
                setup_settings(self, &widgets.window);
            }
        }
    }

    /// Initialize the greeter.
    async fn init(
        input: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self::new(&input.config_path, input.demo).await;
        let widgets = view_output!();

        // Make the info bar permanently visible, since it was made invisible during init. The
        // actual visuals are controlled by `InfoBar::set_revealed`.
        widgets.ui.error_info.set_visible(true);

        // cfg directives don't work inside Relm4 view! macro.
        #[cfg(feature = "gtk4_8")]
        widgets
            .ui
            .background
            .set_content_fit(match model.config.get_background_fit() {
                BgFit::Fill => gtk4::ContentFit::Fill,
                BgFit::Contain => gtk4::ContentFit::Contain,
                BgFit::Cover => gtk4::ContentFit::Cover,
                BgFit::ScaleDown => gtk4::ContentFit::ScaleDown,
            });

        // Cancel any previous session, just in case someone started one.
        if let Err(err) = model.greetd_client.lock().await.cancel_session().await {
            warn!("Couldn't cancel greetd session: {err}");
        };

        model.choose_monitor(widgets.ui.display().name().as_str(), &sender);
        if let Some(monitor) = &model.updates.monitor {
            // The window needs to be manually fullscreened, since the monitor is `None` at widget
            // init.
            root.fullscreen_on_monitor(monitor);
        } else {
            // Couldn't choose a monitor, so let the compositor choose it for us.
            root.fullscreen();
        }

        // For some reason, the GTK settings are reset when changing monitors, so apply them after
        // full-screening.
        setup_settings(&model, &root);
        setup_users_sessions(&model, &widgets);

        if input.css_path.exists() {
            debug!("Loading custom CSS from file: {}", input.css_path.display());
            let provider = gtk::CssProvider::new();
            provider.load_from_path(input.css_path);
            gtk::style_context_add_provider_for_display(
                &widgets.ui.display(),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        };

        // Set the default behaviour of pressing the Return key to act like the login button.
        root.set_default_widget(Some(&widgets.ui.login_button));

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        debug!("Got input message: {msg:?}");

        // Reset the tracker for update changes.
        self.updates.reset();

        match msg {
            Self::Input::Login { input, info } => {
                self.sess_info = Some(info);
                self.login_click_handler(&sender, input).await
            }
            Self::Input::Cancel => self.cancel_click_handler().await,
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
        }
    }

    /// Perform the requested changes when a background task sends a message.
    async fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        debug!("Got command message: {msg:?}");

        // Reset the tracker for update changes.
        self.updates.reset();

        match msg {
            Self::CommandOutput::ClearErr => self.updates.set_error(None),
            Self::CommandOutput::HandleGreetdResponse(response) => {
                self.handle_greetd_response(&sender, response).await
            }
            Self::CommandOutput::MonitorRemoved(display_name) => {
                self.choose_monitor(display_name.as_str(), &sender)
            }
        };
    }
}
