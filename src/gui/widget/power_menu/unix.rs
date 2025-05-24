// SPDX-FileCopyrightText: 2025 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A power menu widget that uses the generic `man 8 shutdown` linux command.
//!
//! See supported actions as variants of [`Action`].

use std::collections::HashSet;

use crate::fl;

use super::{Action, Command, PowerMenuInit};

#[derive(Deserialize, Clone)]
pub struct UnixPowerMenuConfig {
    /// The list of actions to show. Order is preserved. The first unique occurance is used, with duplicates discarded.
    /// E.g. `["poweroff", "reboot", "poweroff"]` Results in this order of widgets: Poweroff, Reboot.
    ///
    /// Please note that only `poweroff`, `reboot`, and `halt` are supported by the unix `shutdown` command.
    #[serde(default = "default_actions")]
    actions: Vec<Action>,
}

pub fn default_actions() -> Vec<Action> {
    [Action::Poweroff, Action::Reboot].into()
}

impl PowerMenuInit for UnixPowerMenuConfig {
    fn backend(&self) -> String {
        fl!("power-menu-backend-unix")
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
                    Action::Poweroff => {
                        vec![
                            "shutdown".to_string(),
                            "--poweroff".to_string(),
                            "now".to_string(),
                        ]
                    }
                    Action::Reboot => vec![
                        "shutdown".to_string(),
                        "--reboot".to_string(),
                        "now".to_string(),
                    ],
                    Action::Halt => vec![
                        "shutdown".to_string(),
                        "--halt".to_string(),
                        "now".to_string(),
                    ],
                    _ => return acc,
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
