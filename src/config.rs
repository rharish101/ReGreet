//! Configuration for the greeter
use serde::{Deserialize, Serialize};

use crate::common::load_toml;
use crate::constants::CONFIG_PATH;

/// Default session
const DEFAULT_SESSION: &str = "sway";

// TODO: Add GTK settings
/// The configuration struct
#[derive(Deserialize, Serialize)]
pub struct Config {
    default_session: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_session: DEFAULT_SESSION.to_string(),
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
}
