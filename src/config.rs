// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration for the greeter
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::common::load_toml;

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

/// The configuration struct
#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    background: Option<String>,
    #[serde(default, rename = "GTK")]
    gtk: Option<GtkSettings>,
}

impl Config {
    pub fn new(path: &Path) -> Self {
        load_toml(path)
    }

    pub fn get_background(&self) -> &Option<String> {
        &self.background
    }

    pub fn get_gtk_settings(&self) -> &Option<GtkSettings> {
        &self.gtk
    }
}
