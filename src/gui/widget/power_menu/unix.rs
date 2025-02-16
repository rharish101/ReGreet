// SPDX-FileCopyrightText: 2025 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A power menu widget that uses the generic `man 8 shutdown` linux command.
//!
//! See supported actions as variants of [`Action`].

use std::collections::HashSet;

use relm4::{adw::glib::markup_escape_text, adw::prelude::*, prelude::*};
use tokio::process::Command;

use crate::{
    demo, fl,
    gui::{icons, GAP},
    i18n::lowercase_first_char,
};

#[derive(Deserialize, Clone)]
pub struct UnixPowerMenuConfig {
    #[serde(default = "Action::default_set")]
    actions: Vec<Action>,
}

#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    Poweroff,
    Reboot,
    Halt,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnixPowerMenu {
    Menu,
    Confirm(Action),
}

#[derive(Debug)]
pub enum UnixPowerMenuMsg {
    Request(Action),
    Confirm,
    Cancel,
}

#[relm4::component(pub, async)]
impl AsyncComponent for UnixPowerMenu {
    type Init = UnixPowerMenuConfig;
    type Input = UnixPowerMenuMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box {
            #[name = "menu"]
            #[transition(Crossfade)]
            match model {
                Self::Menu => &gtk::Box::new(gtk::Orientation::Vertical, GAP) {
                    gtk::Label {
                        set_markup: &format!("<big><b>{}</b> (Unix)</big>", fl!("power-menu-tooltip"))
                    },

                    #[iterate]
                    append: &action_buttons(actions, sender.clone()),
                },

                Self::Confirm(action) => &gtk::Box::new(gtk::Orientation::Vertical, GAP) {
                    gtk::Label {
                        #[watch]
                        set_markup: &format!("<big><b>{}</b></big>",
                            markup_escape_text(&fl!("power-menu-confirm-dialog-heading",
                                what = lowercase_first_char(&action.fl()))
                            )
                        ),
                    },

                    gtk::Box {
                        set_spacing: GAP,

                        gtk::Button::with_label(&fl!("dialog-cancel")) {
                            connect_clicked => UnixPowerMenuMsg::Cancel,
                        },

                        gtk::Button {
                            add_css_class: "destructive-action",

                            #[watch]
                            set_label: &action.fl(),

                            connect_clicked => UnixPowerMenuMsg::Confirm,
                        }
                    }
                },
            }
        }
    }

    async fn init(
        UnixPowerMenuConfig { actions }: Self::Init,
        root: Self::Root,
        sender: relm4::AsyncComponentSender<Self>,
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
        use UnixPowerMenuMsg as M;
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

        if *self == Self::Menu {
            *self = Self::Confirm(action);
            return;
        }

        if demo() {
            info!("Demo mode: not doing {action:?}");
            *self = Self::Menu;

            return;
        }

        let shutdown = Command::new("shutdown")
            .args(action.shutdown_args())
            .status();

        sender.oneshot_command(async move {
            let Err(why) = shutdown.await else { return };
            debug!("Failed to {action:?}: {why}");
        });
    }
}

fn action_buttons(
    activated: Vec<Action>,
    sender: AsyncComponentSender<UnixPowerMenu>,
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

            let container = gtk::Box::new(gtk::Orientation::Horizontal, GAP);
            container.append(&icon);
            container.append(&label);

            let button = gtk::Button::new();
            button.set_child(Some(&container));

            let sender = sender.clone();
            button.connect_clicked(move |_| sender.input(UnixPowerMenuMsg::Request(action)));

            acc.push(button);
            acc
        })
}

impl Action {
    /// Sensible default set of actions that most users would find sufficient.
    pub fn default_set() -> Vec<Self> {
        [Self::Poweroff, Self::Reboot].into()
    }

    pub const fn shutdown_args(&self) -> &'static [&'static str] {
        match self {
            Self::Poweroff => &["--poweroff", "now"],
            Self::Reboot => &["--reboot", "now"],
            Self::Halt => &["--halt", "now"],
        }
    }

    /// Returns the icon name for this action
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Poweroff => icons::POWEROFF,
            Self::Reboot => icons::REBOOT,
            Self::Halt => icons::POWEROFF,
        }
    }

    /// Returns the [`crate::fl!`] for the variant using this format: `fl!("power-menu-{kebab-case}")`
    pub fn fl(&self) -> String {
        match self {
            Self::Poweroff => fl!("power-menu-poweroff"),
            Self::Reboot => fl!("power-menu-reboot"),
            Self::Halt => fl!("power-menu-halt"),
        }
    }
}
