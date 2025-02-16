// SPDX-FileCopyrightText: 2025 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A systemd-aware power menu widget.

use std::collections::HashSet;

use relm4::{
    gtk::{glib::markup_escape_text, prelude::*},
    prelude::{AsyncComponentParts, *},
    AsyncComponentSender,
};
use tokio::process::Command;

use crate::{demo, fl, i18n::lowercase_first_char};

macro_rules! actions {
    ($($variant:ident = $systemctl_args:expr; $icon:ident),+ $(,)?) => {
        #[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[serde(rename_all = "snake_case")]
        pub enum Action {
            $($variant),+
        }

        impl Action {
            /// Every variant as a [`Vec`].
            pub fn all() -> Vec<Action> {
                // It is easier to use a macro than adding a complex dependency.
               [$(Action::$variant),+].into()
            }

            pub fn systemctl_args(&self) -> &'static[&'static str] {
                match self {
                    $(Self::$variant => &$systemctl_args[..]),+
                }
            }

            pub fn icon(&self) -> &'static str {
                match self {
                    $(Self::$variant => crate::gui::icons::$icon),+
                }
            }
        }

    };
}

actions! {
    Poweroff = ["poweroff"]; POWEROFF,
    Reboot = ["reboot"]; REBOOT,
    RebootFirmware = ["reboot", "--firmware-setup"]; REBOOT_FIRMWARE,
    Suspend = ["suspend"]; SUSPEND,
    Hibernate = ["hibernate"]; HIBERNATE,
    HybridSleep = ["hybrid-sleep"]; HIBERNATE, // TODO: Is there an icon for it?
}

impl Action {
    /// Returns the [`crate::fl!`] for the variant using this format: `fl!("power-menu-{snake-case}")`
    fn fl(&self) -> String {
        use Action as A;
        match self {
            A::Poweroff => fl!("power-menu-poweroff"),
            A::Reboot => fl!("power-menu-reboot"),
            A::RebootFirmware => fl!("power-menu-reboot-firmware"),
            A::Suspend => fl!("power-menu-suspend"),
            A::Hibernate => fl!("power-menu-hibernate"),
            A::HybridSleep => fl!("power-menu-hybrid-sleep"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SystemdPowerMenuConfig {
    /// The list of actions to show. Order is preserved. The first unique occurance is used, with duplicates discarded.
    /// E.g. `["poweroff", "reboot", "poweroff"]` Results in this order of widgets: Poweroff, Reboot.
    ///
    /// See [`Action`] for all available actions and what they do.
    #[serde(default = "Action::all")]
    pub actions: Vec<Action>,
}

impl Default for SystemdPowerMenuConfig {
    fn default() -> Self {
        Self {
            actions: Action::all(),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum SystemdPowerMenu {
    /// The default state with all buttons shown.
    Menu,

    /// The confimation widget for a specific action.
    Confirm(Action),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemdPowerMenuMsg {
    /// Sent when the user selected a power menu button. If the action requires confirmation (involves a poweroff of the
    /// system) a confirmation screen is shown ([`SystemdPowerMenu::Confirm`] state).
    Request(Action),

    /// A confirmation of a [`Self::Request`] was cancelled. Has no effect if no confirmation is requested.
    Cancel,

    /// Confirms an action that was requested in [`Self::Request`]. Has no effect if no confirmation is requested.
    Confirm,
}

#[relm4::component(pub, async)]
impl AsyncComponent for SystemdPowerMenu {
    type Init = SystemdPowerMenuConfig;
    type Input = SystemdPowerMenuMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box {
            #[name = "menu"]
            #[transition(Crossfade)]
            match model {
                Self::Menu => &gtk::Box::new(gtk::Orientation::Vertical, 15) {
                    gtk::Label {
                        set_markup: &format!("<big><b>{}</b> (Systemd)</big>", fl!("power-menu-tooltip")),
                    },

                    #[iterate]
                    append: &action_buttons(actions, sender.clone()),
                },

                Self::Confirm(action) => &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 15,

                    gtk::Label {
                        #[watch]
                        set_markup: &format!("<big><b>{}</b></big>",
                            markup_escape_text(&fl!("power-menu-confirm-dialog-heading",
                                what = lowercase_first_char(&action.fl()))
                            )
                        ),
                    },

                    gtk::Box {
                        set_spacing: 15,

                        gtk::Button {
                            set_label: &fl!("dialog-cancel"),

                            connect_clicked => SystemdPowerMenuMsg::Cancel,
                        },

                        gtk::Button {
                            add_css_class: "destructive-action",

                            #[watch]
                            set_label: &action.fl(),

                            connect_clicked => SystemdPowerMenuMsg::Confirm,
                        }
                    }
                },
            }
        }
    }

    async fn init(
        SystemdPowerMenuConfig { actions }: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self::Menu;
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
        use SystemdPowerMenuMsg as M;
        let action = match &message {
            M::Request(action) => *action,

            M::Confirm => {
                let Self::Confirm(what) = self else { return };
                *what
            }

            M::Cancel => {
                *self = Self::Menu;
                return;
            }
        };

        use Action as A;
        let require_confirm = *self == Self::Menu
            && matches!(
                &message,
                M::Request(A::Poweroff | A::Reboot | A::RebootFirmware)
            );

        if require_confirm {
            *self = Self::Confirm(action);
            return;
        }

        if demo() {
            info!("Demo mode: not doing {action:?}");
            *self = Self::Menu;

            return;
        }

        let systemctl = Command::new("systemctl")
            .args(action.systemctl_args())
            .status();

        sender.oneshot_command(async move {
            let Err(why) = systemctl.await else { return };
            debug!("Failed to {action:?}: {why}");
        });
    }
}

fn action_buttons(
    activated: Vec<Action>,
    sender: AsyncComponentSender<SystemdPowerMenu>,
) -> Vec<gtk::Button> {
    let len = activated.len();
    let mut mentioned = HashSet::new();

    activated
        .into_iter()
        .fold(Vec::with_capacity(len), |mut acc, action| {
            if !mentioned.insert(action) {
                return acc;
            }

            let label = gtk::Label::new(Some(&action.fl()));
            let icon = gtk::Image::new();
            icon.set_icon_name(Some(action.icon()));

            let container = gtk::Box::new(gtk::Orientation::Horizontal, 15);
            container.append(&icon);
            container.append(&label);

            let button = gtk::Button::new();
            button.set_child(Some(&container));

            let sender = sender.clone();
            button.connect_clicked(move |_| sender.input(SystemdPowerMenuMsg::Request(action)));

            acc.push(button);
            acc
        })
}
