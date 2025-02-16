// SPDX-FileCopyrightText: 2025 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A [serde-configurable][`PowerMenuConfig`] power menu.

use relm4::prelude::*;
use serde::Deserialize;
use systemd::{SystemdPowerMenu, SystemdPowerMenuConfig};
use unix::{UnixPowerMenu, UnixPowerMenuConfig};

use crate::{fl, gui::icons};

mod systemd;
mod unix;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PowerMenuConfig {
    /// Systemd-aware widget
    Systemd(SystemdPowerMenuConfig),
    Unix(UnixPowerMenuConfig),
}

impl Default for PowerMenuConfig {
    fn default() -> Self {
        Self::Systemd(Default::default())
    }
}

pub enum PowerMenu {
    Systemd(AsyncController<SystemdPowerMenu>),
    Unix(AsyncController<UnixPowerMenu>),
}

#[relm4::component(pub)]
impl Component for PowerMenu {
    type Init = PowerMenuConfig;
    type Input = ();
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::MenuButton {
            set_icon_name: icons::POWER_MENU,
            set_tooltip: &fl!("power-menu-tooltip"),

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                model.widget() { },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = match init {
            Self::Init::Systemd(systemd_config) => {
                Self::Systemd(SystemdPowerMenu::builder().launch(systemd_config).detach())
            }

            Self::Init::Unix(unix_power_menu_config) => Self::Unix(
                UnixPowerMenu::builder()
                    .launch(unix_power_menu_config)
                    .detach(),
            ),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

impl PowerMenu {
    fn widget(&self) -> &gtk::Box {
        match self {
            Self::Systemd(controller) => controller.widget(),
            Self::Unix(controller) => controller.widget(),
        }
    }
}
