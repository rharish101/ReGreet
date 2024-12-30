// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stores constants that can be configured at compile time

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
const CACHE_DIR: &str = env_or!("STATE_DIR", concatcp!("/var/lib/", GREETER_NAME));
/// Path to the cache file
pub const CACHE_PATH: &str = concatcp!(CACHE_DIR, "/state.toml");

/// The directory for system log files
const LOG_DIR: &str = env_or!("LOG_DIR", concatcp!("/var/log/", GREETER_NAME));
/// Path to the log file
pub const LOG_PATH: &str = concatcp!(LOG_DIR, "/log");

/// Default command for rebooting
pub const REBOOT_CMD: &str = env_or!("REBOOT_CMD", "reboot");
/// Default command for shutting down
pub const POWEROFF_CMD: &str = env_or!("POWEROFF_CMD", "poweroff");

/// Default greeting message
pub const GREETING_MSG: &str = "Welcome back!";

/// `:`-separated search path for `login.defs` file.
///
/// By default this file is at `/etc/login.defs`, however some distros (e.g. Tumbleweed) move it to other locations.
///
/// See: <https://github.com/rharish101/ReGreet/issues/89>
pub const LOGIN_DEFS_PATHS: &[&str] = {
    const ENV: &str = env_or!("LOGIN_DEFS_PATHS", "/etc/login.defs:/usr/etc/login.defs");
    &str_split!(ENV, ':')
};

lazy_static! {
    /// Override the default `UID_MIN` in `login.defs`. If the string cannot be parsed at runtime, the value is `1_000`.
    ///
    /// This is not meant as a configuration facility. Only override this value if it's a different default in the
    /// `passwd` suite.
    pub static ref LOGIN_DEFS_UID_MIN: u64 = {
        const DEFAULT: u64 = 1_000;
        const ENV: &str = env_or!("LOGIN_DEFS_UID_MIN", formatcp!("{DEFAULT}"));

        ENV.parse()
            .map_err(|e| error!("Failed to parse LOGIN_DEFS_UID_MIN='{ENV}': {e}. This is a compile time mistake!"))
            .unwrap_or(DEFAULT)
    };

    /// Override the default `UID_MAX` in `login.defs`. If the string cannot be parsed at runtime, the value is
    /// `60_000`.
    ///
    /// This is not meant as a configuration facility. Only override this value if it's a different default in the
    /// `passwd` suite.
    pub static ref LOGIN_DEFS_UID_MAX: u64 = {
        const DEFAULT: u64 = 60_000;
        const ENV: &str = env_or!("LOGIN_DEFS_UID_MAX", formatcp!("{DEFAULT}"));

        ENV.parse()
            .map_err(|e| error!("Failed to parse LOGIN_DEFS_UID_MAX='{ENV}': {e}. This is a compile time mistake!"))
            .unwrap_or(DEFAULT)
    };
}

/// Directories separated by `:`, containing desktop files for X11/Wayland sessions
pub const SESSION_DIRS: &str = env_or!(
    "SESSION_DIRS",
    "/usr/share/xsessions:/usr/share/wayland-sessions"
);

/// Command prefix for X11 sessions to start the X server
pub const X11_CMD_PREFIX: &str = env_or!("X11_CMD_PREFIX", "startx /usr/bin/env");
