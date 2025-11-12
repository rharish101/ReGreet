// SPDX-FileCopyrightText: 2025 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Systemd power menu backend.
//!
//! See supported actions as variants of [`Action`].

use std::collections::HashSet;

use crate::fl;

use super::{Action, Command, PowerMenuInit};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SystemdPowerMenuConfig {
    /// The list of actions to show. Order is preserved.
    ///
    /// The first unique occurance is used, with duplicates discarded.
    /// E.g. `["poweroff", "reboot", "poweroff"]` Results in this order of widgets: Poweroff, Reboot.
    ///
    /// The button labels, icons and `systemctl` commands are automatically selected based on the action you specify.
    #[serde(default = "default_actions")]
    pub actions: Vec<Action>,
}

impl Default for SystemdPowerMenuConfig {
    fn default() -> Self {
        Self {
            actions: default_actions(),
        }
    }
}

/// Sensible default set of actions that most users would find sufficient.
fn default_actions() -> Vec<Action> {
    [Action::Poweroff, Action::Reboot, Action::Suspend].into()
}

impl PowerMenuInit for SystemdPowerMenuConfig {
    fn backend(&self) -> String {
        fl!("power-menu-backend-systemd")
    }

    fn commands(self) -> Vec<Command> {
        let mut mentioned = HashSet::new();

        let len = self.actions.len();

        self.actions
            .into_iter()
            .fold(Vec::with_capacity(len), |mut acc, action| {
                if !mentioned.insert(action) {
                    return acc;
                }

                let command = match action {
                    Action::Poweroff => vec!["systemctl".to_string(), "poweroff".to_string()],
                    Action::Halt => vec!["systemctl".to_string(), "halt".to_string()],
                    Action::Reboot => vec!["systemctl".to_string(), "reboot".to_string()],
                    Action::RebootFirmware => vec![
                        "systemctl".to_string(),
                        "reboot".to_string(),
                        "--firmware-setup".to_string(),
                    ],
                    Action::Suspend => vec!["systemctl".to_string(), "suspend".to_string()],
                    Action::Hibernate => vec!["systemctl".to_string(), "hibernate".to_string()],
                    Action::HybridSleep => {
                        vec!["systemctl".to_string(), "hybrid-sleep".to_string()]
                    }
                };

                acc.push(Command::new(
                    action.fl(),
                    action.icon().to_owned(),
                    action.is_like_poweroff(),
                    command,
                ));
                acc
            })
    }
}
