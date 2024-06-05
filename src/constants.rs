// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stores constants that can be configured at compile time

use const_format::concatcp;

/// Get an environment variable during compile time, else return a default.
macro_rules! env_or {
    ($name:expr, $default:expr) => {
        // This is needed because `Option.unwrap_or` is not a const fn:
        // https://github.com/rust-lang/rust/issues/91930
        if let Some(value) = option_env!($name) {
            value
        } else {
            $default
        }
    };
}

/// The name for this greeter
const GREETER_NAME: &str = "regreet";
/// The app ID for this GTK app
pub const APP_ID: &str = concatcp!("apps.", GREETER_NAME);

/// The greetd config directory
const GREETD_CONFIG_DIR: &str = env_or!("GREETD_CONFIG_DIR", "/etc/greetd");
/// Path to the config file
pub const CONFIG_PATH: &str = concatcp!(GREETD_CONFIG_DIR, "/", GREETER_NAME, ".toml");
/// Path to the config file
pub const CSS_PATH: &str = concatcp!(GREETD_CONFIG_DIR, "/", GREETER_NAME, ".css");

/// The directory for system cache files
const CACHE_DIR: &str = env_or!("CACHE_DIR", concatcp!("/var/cache/", GREETER_NAME));
/// Path to the cache file
pub const CACHE_PATH: &str = concatcp!(CACHE_DIR, "/cache.toml");

/// The directory for system log files
const LOG_DIR: &str = env_or!("LOG_DIR", concatcp!("/var/log/", GREETER_NAME));
/// Path to the cache file
pub const LOG_PATH: &str = concatcp!(LOG_DIR, "/log");

/// Default command for rebooting
pub const REBOOT_CMD: &str = env_or!("REBOOT_CMD", "reboot");
/// Default command for shutting down
pub const POWEROFF_CMD: &str = env_or!("POWEROFF_CMD", "poweroff");

/// Default greeting message
pub const GREETING_MSG: &str = "Welcome back!";

/// Directories separated by `:`, containing desktop files for X11/Wayland sessions
pub const SESSION_DIRS: &str = env_or!(
    "SESSION_DIRS",
    "/usr/share/xsessions:/usr/share/wayland-sessions"
);
