//! Configuration for the greeter
use serde::{Deserialize, Serialize};

use crate::common::load_toml;
use crate::constants::CONFIG_PATH;

/// Default session
const DEFAULT_GTK_PREFER_DARK_THEME: bool = true;
const DEFAULT_GTK_CURSOR_THEME: &str = "Adwaita";
const DEFAULT_GTK_THEME: &str = "Adwaita";

/// Struct holding all supported GTK settings
#[derive(Deserialize, Serialize)]
pub struct GTKSettings {
    #[serde(default = "get_default_gtk_prefer_dark_theme")]
    pub application_prefer_dark_theme: bool,
    #[serde(default = "get_default_gtk_cursor_theme")]
    pub cursor_theme_name: Option<String>,
    #[serde(default)]
    pub font_name: Option<String>,
    #[serde(default)]
    pub icon_theme_name: Option<String>,
    #[serde(default = "get_default_gtk_theme")]
    pub theme_name: Option<String>,
}

impl Default for GTKSettings {
    fn default() -> Self {
        Self {
            application_prefer_dark_theme: get_default_gtk_prefer_dark_theme(),
            cursor_theme_name: get_default_gtk_cursor_theme(),
            font_name: None,
            icon_theme_name: None,
            theme_name: get_default_gtk_theme(),
        }
    }
}

/// The configuration struct
#[derive(Deserialize, Serialize)]
pub struct Config {
    #[serde(default, rename = "GTK")]
    gtk: Option<GTKSettings>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gtk: Some(GTKSettings::default()),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        load_toml(CONFIG_PATH)
    }

    pub fn get_gtk_settings(&self) -> &Option<GTKSettings> {
        &self.gtk
    }
}

// Functions used for defaults when deserializing

fn get_default_gtk_prefer_dark_theme() -> bool {
    DEFAULT_GTK_PREFER_DARK_THEME
}

fn get_default_gtk_cursor_theme() -> Option<String> {
    Some(String::from(DEFAULT_GTK_CURSOR_THEME))
}

fn get_default_gtk_theme() -> Option<String> {
    Some(String::from(DEFAULT_GTK_THEME))
}
