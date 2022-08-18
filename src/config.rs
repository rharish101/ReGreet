//! Configuration for the greeter
use serde::{Deserialize, Serialize};

use crate::common::load_toml;
use crate::constants::CONFIG_PATH;

/// Default session
const DEFAULT_SESSION: &str = "Sway";
const DEFAULT_GTK_PREFER_DARK_THEME: bool = true;
const DEFAULT_GTK_CURSOR_THEME: &str = "Adwaita";
const DEFAULT_GTK_THEME: &str = "Adwaita";

/// Struct holding all supported GTK settings
#[derive(Deserialize, Serialize)]
pub struct GTKSettings {
    pub application_prefer_dark_theme: bool,
    pub cursor_theme_name: Option<String>,
    pub font_name: Option<String>,
    pub icon_theme_name: Option<String>,
    pub theme_name: Option<String>,
}

/// The configuration struct
#[derive(Deserialize, Serialize)]
pub struct Config {
    default_session: String,
    gtk: Option<GTKSettings>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_session: DEFAULT_SESSION.to_string(),
            gtk: Some(GTKSettings {
                application_prefer_dark_theme: DEFAULT_GTK_PREFER_DARK_THEME,
                cursor_theme_name: Some(String::from(DEFAULT_GTK_CURSOR_THEME)),
                font_name: None,
                icon_theme_name: None,
                theme_name: Some(String::from(DEFAULT_GTK_THEME)),
            }),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        load_toml(CONFIG_PATH)
    }

    pub fn get_default_session(&self) -> &str {
        self.default_session.as_str()
    }

    pub fn get_gtk_settings(&self) -> &Option<GTKSettings> {
        &self.gtk
    }
}
