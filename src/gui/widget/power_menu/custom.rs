// SPDX-FileCopyrightText: 2025 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A fully customizable power menu widget.

use super::{Action, PowerMenuInit};

#[derive(Deserialize, Clone)]

/// A fully-customizable power menu. You can run arbitrary commands and set labels for them. See [`Command`] for a list
/// of all properties that can be configured
pub struct CustomPowerMenuConfig {
    backend: String,
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
    Label(String),
}

impl PowerMenuInit for CustomPowerMenuConfig {
    fn backend(&self) -> String {
        self.backend.clone()
    }
    fn commands(self) -> Vec<super::Command> {
        let len = self.commands.len();
        self.commands.into_iter().fold(
            Vec::with_capacity(len),
            |mut acc,
             Command {
                 title,
                 command,
                 confirm,
                 icon,
             }| {
                let icon = icon.unwrap_or(match title {
                    Title::Action(action) => action.icon().to_owned(),
                    Title::Label(_) => String::default(),
                });

                let label = match title {
                    Title::Action(action) => action.fl(),
                    Title::Label(label) => label,
                };

                acc.push(super::Command {
                    label,
                    command,
                    confirm: confirm.unwrap_or_default(),
                    icon,
                });
                acc
            },
        )
    }
}
