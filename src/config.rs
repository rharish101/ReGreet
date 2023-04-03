// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration for the greeter

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::constants::{POWEROFF_CMD, REBOOT_CMD};
use crate::tomlutils::load_toml;

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

/// Struct for reboot/poweroff commands
#[derive(Deserialize, Serialize)]
pub struct SystemCommands {
    #[serde(default = "default_reboot_command")]
    pub reboot: Vec<String>,
    #[serde(default = "default_poweroff_command")]
    pub poweroff: Vec<String>,
}

impl Default for SystemCommands {
    fn default() -> Self {
        SystemCommands {
            reboot: default_reboot_command(),
            poweroff: default_poweroff_command(),
        }
    }
}

fn default_reboot_command() -> Vec<String> {
    shlex::split(REBOOT_CMD).expect("Unable to lex reboot command")
}

fn default_poweroff_command() -> Vec<String> {
    shlex::split(POWEROFF_CMD).expect("Unable to lex poweroff command")
}

/// The configuration struct
#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    background: Option<String>,
    #[serde(default)]
    pub background_fit: Option<BgFit>,
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
        &self.background
    }

    #[cfg(feature = "gtk4_8")]
    pub fn get_background_fit(&self) -> &Option<BgFit> {
        &self.background_fit
    }

    pub fn get_gtk_settings(&self) -> &Option<GtkSettings> {
        &self.gtk
    }

    pub fn get_sys_commands(&self) -> &SystemCommands {
        &self.commands
    }
}
