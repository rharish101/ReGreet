// SPDX-FileCopyrightText: 2025 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A fully customizable power menu widget.

use adw::glib::markup_escape_text;
use relm4::{adw::prelude::*, prelude::*};
use tokio::process;

use crate::{
    demo, fl,
    gui::{widget::power_menu::header_label, GAP},
    i18n::lowercase_first_char,
};

#[derive(Deserialize, Clone)]

pub struct CustomPowerMenuConfig {
    commands: Vec<Command>,
}

#[derive(Deserialize, Clone)]
pub struct Command {
    /// The title of the action.
    #[serde(flatten)]
    title: Title,

    /// Command to be executed.
    #[serde(alias = "cmd")]
    command: Vec<String>,

    /// If `true`, a confirmation will be shown before the [`Self::command`] is executed.
    ///
    /// If [`Some`], use the value. Otherwise, try to derive the value from the [`Title::Action`], falling back to
    /// `false`. Actions that involve a poweroff are inferred to require confirmation.
    confirm: Option<bool>,

    /// The icon name to set for this action. A list of installed icons can be looked up using the `icon-library` app.
    icon: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Title {
    /// One of the `i18n`-supported actions
    Action(Action),

    /// Arbitrary name string
    Name(String),
}

#[derive(Deserialize, Clone, Copy)]
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

pub struct CustomPowerMenu {
    state: MenuState,
    commands: Vec<Command>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MenuState {
    Menu,
    Confirm(usize),
}

#[derive(Debug)]
pub enum CustomPowerMenuMsg {
    Request(usize),
    Confirm,
    Cancel,
}

#[relm4::component(pub, async)]
impl AsyncComponent for CustomPowerMenu {
    type Init = CustomPowerMenuConfig;
    type Input = CustomPowerMenuMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box {
            #[name="menu"]
            #[transition(Crossfade)]
            match model.state {
                MenuState::Menu => gtk::Box::new(gtk::Orientation::Vertical, GAP) {
                    header_label("Custom") {},

                    #[iterate]
                    append: &action_buttons(&model.commands, sender.clone()),
                },

                MenuState::Confirm(index) =>&gtk::Box::new(gtk::Orientation::Vertical, GAP) {
                    gtk::Label {
                        #[watch]
                        set_markup: &format!("<big><b>{}</b></big>",
                            markup_escape_text(&fl!("power-menu-confirm-dialog-heading",
                                what = lowercase_first_char(&model.commands[index].fl()))
                            )
                        ),
                    },

                    gtk::Box {
                        set_spacing:GAP,

                        gtk::Button::with_label(&fl!("dialog-cancel")) {
                            connect_clicked => CustomPowerMenuMsg::Cancel,
                        },

                        gtk::Button {
                            add_css_class: "destructive-action",

                            #[watch]
                            set_label: &model.commands[index].fl(),

                            connect_clicked => CustomPowerMenuMsg::Confirm,
                        }
                    }
                }
            }
        }
    }

    async fn init(
        CustomPowerMenuConfig { commands }: Self::Init,
        root: Self::Root,
        sender: relm4::AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            commands,
            state: MenuState::Menu,
        };
        let widgets = view_output!();

        widgets.menu.set_hhomogeneous(false);
        widgets.menu.set_vhomogeneous(false);

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _: &Self::Root,
    ) {
        use CustomPowerMenuMsg as M;
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

        let command = &self.commands[index];

        use Action as A;
        let require_confirm = self.state == MenuState::Menu
            && (command.confirm == Some(true)
                || matches!(
                    command.title,
                    Title::Action(A::Poweroff | A::Reboot | A::RebootFirmware)
                ));

        if require_confirm {
            self.state = MenuState::Confirm(index);
            return;
        }

        let fl = command.fl();

        if demo() {
            info!("Demo mode: not doing {fl}");
            self.state = MenuState::Menu;

            return;
        }

        let command = process::Command::new(&command.command[0])
            .args(&command.command[1..])
            .status();

        sender.oneshot_command(async move {
            let Err(why) = command.await else { return };
            debug!("Failed to {fl}: {why}");
        });
    }
}

fn action_buttons(
    commands: &[Command],
    sender: AsyncComponentSender<CustomPowerMenu>,
) -> Vec<gtk::Button> {
    let len = commands.len();

    commands
        .iter()
        .enumerate()
        .fold(Vec::with_capacity(len), |mut acc, (index, command)| {
            let button = gtk::Button::new();

            if let Some(icon_name) = command.icon() {
                let icon = gtk::Image::new();
                icon.set_icon_name(Some(icon_name));

                let label = gtk::Label::new(Some(&command.fl()));

                let container = gtk::Box::new(gtk::Orientation::Horizontal, GAP);
                container.append(&icon);
                container.append(&label);

                button.set_child(Some(&container));
            } else {
                button.set_label(&command.fl());
            }

            let sender = sender.clone();
            button.connect_clicked(move |_| sender.input(CustomPowerMenuMsg::Request(index)));

            acc.push(button);
            acc
        })
}

impl Command {
    fn fl(&self) -> String {
        let action = match self.title {
            Title::Action(action) => action,

            Title::Name(ref name) => return name.clone(),
        };

        use Action as A;
        match action {
            A::Poweroff => fl!("power-menu-poweroff"),
            A::Halt => fl!("power-menu-halt"),
            A::Reboot => fl!("power-menu-reboot"),
            A::RebootFirmware => fl!("power-menu-reboot-firmware"),
            A::Suspend => fl!("power-menu-suspend"),
            A::Hibernate => fl!("power-menu-hibernate"),
            A::HybridSleep => fl!("power-menu-hybrid-sleep"),
        }
    }

    fn icon(&self) -> Option<&str> {
        let None = self.icon else {
            return self.icon.as_deref();
        };

        let Title::Action(action) = self.title else {
            return None;
        };

        use Action as A;
        Some(match action {
            A::Poweroff => crate::gui::icons::POWEROFF,
            A::Halt => crate::gui::icons::POWEROFF,
            A::Reboot => crate::gui::icons::REBOOT,
            A::RebootFirmware => crate::gui::icons::REBOOT_FIRMWARE,
            A::Suspend => crate::gui::icons::SUSPEND,
            A::Hibernate => crate::gui::icons::HIBERNATE,
            A::HybridSleep => crate::gui::icons::HIBERNATE,
        })
    }
}
