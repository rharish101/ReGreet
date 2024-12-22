// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration for the greeter

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::constants::{GREETING_MSG, POWEROFF_CMD, REBOOT_CMD, X11_CMD_PREFIX};
use crate::tomlutils::load_toml;

#[derive(Deserialize, Serialize)]
pub struct AppearanceSettings {
    #[serde(default = "default_greeting_msg")]
    pub greeting_msg: String,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        AppearanceSettings {
            greeting_msg: default_greeting_msg(),
        }
    }
}

/// Struct holding all supported GTK settings
#[derive(Default, Deserialize, Serialize)]
pub struct GtkSettings {
    #[serde(default)]
    pub application_prefer_dark_theme: bool,
    #[serde(default)]
    pub cursor_theme_name: Option<String>,
    #[serde(default)]
    pub font_name: Option<String>,
    #[serde(default)]
    pub icon_theme_name: Option<String>,
    #[serde(default)]
    pub theme_name: Option<String>,
}

/// Analogue to `gtk4::ContentFit`
#[derive(Default, Deserialize, Serialize)]
pub enum BgFit {
    Fill,
    #[default]
    Contain,
    Cover,
    ScaleDown,
}

/// Struct for info about the background image
#[derive(Default, Deserialize, Serialize)]
struct Background {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    fit: BgFit,
}

/// Struct for various system commands
#[derive(Deserialize, Serialize)]
pub struct SystemCommands {
    #[serde(default = "default_reboot_command")]
    pub reboot: Vec<String>,
    #[serde(default = "default_poweroff_command")]
    pub poweroff: Vec<String>,
    #[serde(default = "default_x11_command_prefix")]
    pub x11_prefix: Vec<String>,
}

impl Default for SystemCommands {
    fn default() -> Self {
        SystemCommands {
            reboot: default_reboot_command(),
            poweroff: default_poweroff_command(),
            x11_prefix: default_x11_command_prefix(),
        }
    }
}

fn default_reboot_command() -> Vec<String> {
    shlex::split(REBOOT_CMD).expect("Unable to lex reboot command")
}

fn default_poweroff_command() -> Vec<String> {
    shlex::split(POWEROFF_CMD).expect("Unable to lex poweroff command")
}

fn default_x11_command_prefix() -> Vec<String> {
    shlex::split(X11_CMD_PREFIX).expect("Unable to lex X11 command prefix")
}

fn default_greeting_msg() -> String {
    GREETING_MSG.to_string()
}

/// The configuration struct
#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    appearance: AppearanceSettings,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    background: Background,
    #[serde(default, rename = "GTK")]
    gtk: Option<GtkSettings>,
    #[serde(default)]
    commands: SystemCommands,
}

impl Config {
    pub fn new(path: &Path) -> Self {
        load_toml(path)
    }

    pub fn get_env(&self) -> &HashMap<String, String> {
        &self.env
    }

    pub fn get_background(&self) -> &Option<String> {
        &self.background.path
    }

    #[cfg(feature = "gtk4_8")]
    pub fn get_background_fit(&self) -> &BgFit {
        &self.background.fit
    }

    pub fn get_gtk_settings(&self) -> &Option<GtkSettings> {
        &self.gtk
    }

    pub fn get_sys_commands(&self) -> &SystemCommands {
        &self.commands
    }

    pub fn get_default_message(&self) -> String {
        self.appearance.greeting_msg.clone()
    }
}
