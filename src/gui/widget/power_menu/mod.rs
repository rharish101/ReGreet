// SPDX-FileCopyrightText: 2025 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A [serde-configurable][`PowerMenuConfig`] power menu.

use custom::CustomPowerMenuConfig;
use relm4::{
    gtk::{glib::markup_escape_text, prelude::*},
    prelude::{AsyncComponentParts, *},
};
use serde::Deserialize;
use systemd::SystemdPowerMenuConfig;
use tokio::process;
use unix::UnixPowerMenuConfig;

use crate::{
    demo, fl,
    gui::{icons, GAP},
    i18n::lowercase_first_char,
};

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
    /// Sent when the user selected a power menu button. If the action requires confirmation (involves a poweroff of the
    /// system) a confirmation screen is shown ([`PowerMenu::Confirm`] state).
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
                        MenuState::Menu => &gtk::Box::new(gtk::Orientation::Vertical, GAP) {
                            gtk::Label {
                                set_markup: &format!(
                                    "<big><b>{}</b></big>\n<small>{}: {backend}</small>",
                                    markup_escape_text(&fl!("power-menu-tooltip")),
                                    markup_escape_text(&fl!("power-menu-backend")),
                                ),
                            },

                            #[iterate]
                            append: &action_buttons(&model.commands, sender.clone()),
                        },

                        MenuState::Confirm(index) => &gtk::Box::new(gtk::Orientation::Vertical, GAP) {
                            gtk::Label {
                                #[watch]
                                set_markup: &format!("<big><b>{}</b></big>",
                                    markup_escape_text(&fl!("power-menu-confirm-dialog-heading",
                                        what = lowercase_first_char(&model.commands[index].label))
                                    )
                                ),
                            },

                            gtk::Box {
                                set_spacing: GAP,

                                gtk::Button::with_label(&fl!("dialog-cancel")) {
                                    connect_clicked => PowerMenuMsg::Cancel,
                                },

                                gtk::Button {
                                    add_css_class: "destructive-action",

                                    #[watch]
                                    set_label: &model.commands[index].label,

                                    connect_clicked => PowerMenuMsg::Confirm,
                                }
                            }
                        },
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
) -> Vec<gtk::Button> {
    commands.iter().enumerate().fold(
        Vec::with_capacity(commands.len()),
        |mut acc, (index, command)| {
            let button = gtk::Button::new();

            if command.icon.is_empty() {
                button.set_label(&command.label);
            } else {
                let icon = gtk::Image::new();
                icon.set_icon_name(Some(&command.icon));

                let label = gtk::Label::new(Some(&command.label));

                let container = gtk::Box::new(gtk::Orientation::Horizontal, GAP);
                container.append(&icon);
                container.append(&label);

                button.set_child(Some(&container));
            }

            let sender = sender.clone();
            button.connect_clicked(move |_| sender.input(PowerMenuMsg::Request(index)));

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

    pub const fn is_like_poweroff(&self) -> bool {
        matches!(
            self,
            Self::Poweroff | Self::Halt | Self::Reboot | Self::RebootFirmware
        )
    }
}

trait PowerMenuInit {
    fn backend(&self) -> String;
    fn commands(self) -> Vec<Command>;
}
