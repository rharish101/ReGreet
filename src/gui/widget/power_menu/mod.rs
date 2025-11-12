// SPDX-FileCopyrightText: 2025 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A [serde-configurable][`PowerMenuConfig`] power menu.
//!
//! See submodules for information on how to configure each power menu backend.
//!
//! ## Implementing a custom power menu backend
//!
//! Users of ReGreet can use the [`custom`] backend to specify arbitrary commands you want to run. Although this is not
//! intended, nothing stops you from adding entries unrelated to power management.
//!
//! However, if you want to add support for another init system, for example, it is best to implement a custom backend.
//! Fortunately, this is easy to do.
//!
//! You will need to create a config for this backend. Most likely, only having a single `actions: Vec<Action>` field
//! will be sufficient. For forward compatibility, please do not skip creating a `struct {}` to hold the config (exactly
//! the `{}` struct format). Having this struct allows future maintainers to add or rename fields more easily without
//! having to deal with config structure migrations.
//!
//! Make sure to implement sensible defaults for the entire struct. In cases where a default value should be inferred
//! after the config is parsed, use an [`Option`] to detect that the user did not set the value and generate the default
//! in the [`PowerMenuInit`] implementation.
//!
//! Finally, implement a coversion from your custom config struct using the [`PowerMenuInit`] trait. The [`Action`] can
//! be used to infer a lot of information such as the [icon](`Action::icon`) and the [translated label](`Action::fl`).
//! The confirmation requirement logic defaults to checking if the action
//! [involves a poweroff](`Action::is_like_poweroff`).
//!
//! Lastly, add your config type as a variant to the [`PowerMenuConfig`].

use custom::CustomPowerMenuConfig;
use relm4::{
    adw::prelude::*,
    gtk::glib::markup_escape_text,
    prelude::{AsyncComponentParts, *},
};
use serde::Deserialize;
use systemd::SystemdPowerMenuConfig;
use tokio::process;
use unix::UnixPowerMenuConfig;

use crate::{demo, fl, gui::icons, i18n::lowercase_first_char};

mod custom;
mod systemd;
mod unix;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PowerMenuConfig {
    /// Systemd-aware widget
    Systemd(SystemdPowerMenuConfig),
    Unix(UnixPowerMenuConfig),
    Custom(CustomPowerMenuConfig),
}

impl Default for PowerMenuConfig {
    fn default() -> Self {
        Self::Systemd(Default::default())
    }
}

pub struct PowerMenu {
    state: MenuState,
    commands: Vec<Command>,
}

#[derive(Clone)]
pub struct Command {
    /// The label of the action.
    pub label: String,

    /// Command to be executed.
    pub command: Vec<String>,

    /// If `true`, a confirmation will be shown before the [`Self::command`] is executed.
    pub confirm: bool,

    /// The icon name to set for this action. A list of installed icons can be looked up using the `icon-library` app.
    ///
    /// If empty, no icon will be displayed.
    pub icon: String,
}

#[derive(PartialEq, Eq)]
pub enum MenuState {
    /// The default state with all buttons shown.
    Menu,

    /// The confimation widget for a specific action.
    Confirm(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMenuMsg {
    /// Sent when the user selected a power menu button. If the action [requires confirmation](`Command::confirm`)
    /// a confirmation screen is shown ([`MenuState::Confirm`] state).
    Request(usize),

    /// A confirmation of a [`PowerMenuMsg::Request`] was cancelled. Has no effect if no confirmation is requested.
    Cancel,

    /// Confirms an action that was requested in [`PowerMenuMsg::Request`]. Has no effect if no confirmation is requested.
    Confirm,
}

#[relm4::component(pub, async)]
impl AsyncComponent for PowerMenu {
    type Init = PowerMenuConfig;
    type Input = PowerMenuMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::MenuButton {
            set_icon_name: icons::POWER_MENU,
            set_tooltip: &fl!("power-menu-tooltip"),

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                gtk::Box {
                    #[name = "menu"]
                    #[transition(Crossfade)]
                    match model.state {
                        MenuState::Menu => &adw::PreferencesGroup {
                            set_title: &markup_escape_text(&fl!("power-menu-tooltip")),
                            set_description: Some(&format!("{}: {backend}", markup_escape_text(&fl!("power-menu-backend")))),

                            #[iterate]
                            add: &action_buttons(&model.commands, sender.clone()),
                        },

                        MenuState::Confirm(index) => &adw::PreferencesGroup {
                            #[watch]
                            set_title: &markup_escape_text(&fl!("power-menu-confirm-dialog-heading",
                                what = lowercase_first_char(&model.commands[index].label))
                            ),

                            set_separate_rows: true,

                            adw::ButtonRow {
                                set_title: &fl!("dialog-cancel"),

                                connect_activated => PowerMenuMsg::Cancel,
                            },

                            adw::ButtonRow {
                                #[watch]
                                set_title: &model.commands[index].label,
                                add_css_class: "destructive-action",

                                connect_activated => PowerMenuMsg::Confirm,
                            }
                        }
                    }
                }
            },
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let (backend, commands) = match init {
            Self::Init::Systemd(systemd_config) => {
                (systemd_config.backend(), systemd_config.commands())
            }

            Self::Init::Unix(unix_power_menu_config) => (
                unix_power_menu_config.backend(),
                unix_power_menu_config.commands(),
            ),

            Self::Init::Custom(custom_power_menu_config) => (
                custom_power_menu_config.backend(),
                custom_power_menu_config.commands(),
            ),
        };

        let model = Self {
            commands,
            state: MenuState::Menu,
        };

        let widgets = view_output!();

        widgets.menu.set_vhomogeneous(false);
        widgets.menu.set_hhomogeneous(false);

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _: &Self::Root,
    ) {
        use PowerMenuMsg as M;
        let index = match message {
            M::Request(index) => index,

            M::Confirm => {
                let MenuState::Confirm(what) = self.state else {
                    return;
                };
                what
            }

            M::Cancel => {
                self.state = MenuState::Menu;
                return;
            }
        };

        let Command {
            confirm,
            label,
            command,
            ..
        } = &self.commands[index];

        let require_confirm = self.state == MenuState::Menu && *confirm;

        if require_confirm {
            self.state = MenuState::Confirm(index);
            return;
        }

        if demo() {
            info!("Demo mode: not doing {label}");
            self.state = MenuState::Menu;

            return;
        }

        let command = process::Command::new(&command[0])
            .args(&command[1..])
            .status();

        let label = label.clone();
        sender.oneshot_command(async move {
            let Err(why) = command.await else { return };
            debug!("Failed to {label}: {why}");
        });
    }
}

fn action_buttons(
    commands: &[Command],
    sender: AsyncComponentSender<PowerMenu>,
) -> Vec<adw::ButtonRow> {
    commands.iter().enumerate().fold(
        Vec::with_capacity(commands.len()),
        |mut acc, (index, command)| {
            let button = adw::ButtonRow::new();
            button.set_title(&command.label);

            if !command.icon.is_empty() {
                button.set_start_icon_name(Some(&command.icon));
            }

            let sender = sender.clone();
            button.connect_activated(move |_| sender.input(PowerMenuMsg::Request(index)));

            acc.push(button);
            acc
        },
    )
}

impl Command {
    pub fn new(label: String, icon: String, confirm: bool, command: Vec<String>) -> Self {
        Self {
            label,
            command,
            confirm,
            icon,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    Poweroff,
    Halt,
    Reboot,
    RebootFirmware,
    Suspend,
    Hibernate,
    HybridSleep,
}

impl Action {
    /// Returns the icon name for this action
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Poweroff => crate::gui::icons::POWEROFF,
            Self::Halt => crate::gui::icons::POWEROFF,
            Self::Reboot => crate::gui::icons::REBOOT,
            Self::RebootFirmware => crate::gui::icons::REBOOT_FIRMWARE,
            Self::Suspend => crate::gui::icons::SUSPEND,
            Self::Hibernate => crate::gui::icons::HIBERNATE,
            Self::HybridSleep => crate::gui::icons::HIBERNATE,
        }
    }

    /// Returns the [`crate::fl!`] for the variant using this format: `fl!("power-menu-{kebab-case}")`
    pub fn fl(&self) -> String {
        use Action as A;
        match self {
            A::Poweroff => fl!("power-menu-poweroff"),
            A::Halt => fl!("power-menu-halt"),
            A::Reboot => fl!("power-menu-reboot"),
            A::RebootFirmware => fl!("power-menu-reboot-firmware"),
            A::Suspend => fl!("power-menu-suspend"),
            A::Hibernate => fl!("power-menu-hibernate"),
            A::HybridSleep => fl!("power-menu-hybrid-sleep"),
        }
    }

    /// Returns `true` if executing this action would power off the system (meaning reboot counts). Putting the system to
    /// sleep does not count as it can be easily undone by waking up the system.
    pub const fn is_like_poweroff(&self) -> bool {
        matches!(
            self,
            Self::Poweroff | Self::Halt | Self::Reboot | Self::RebootFirmware
        )
    }
}

/// A convenience trait that converts any power menu config into a set of information that the generic UI can display.
trait PowerMenuInit {
    fn backend(&self) -> String;
    fn commands(self) -> Vec<Command>;
}
