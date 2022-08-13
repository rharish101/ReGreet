//! Stores constants that can be configured at compile time
use const_format::concatcp;

/// Get an environment variable during compile time, else return a default
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
pub const GREETER_NAME: &str = "egreet";

/// The shared data directory (usually `/usr` or `/usr/local`)
const INSTALL_DIR: &str = env_or!("INSTALL_DIR", "/usr");
/// The directory where to install this greeter's files
const GREETER_DIR: &str = concatcp!(INSTALL_DIR, "/share/", GREETER_NAME);
/// Path to the UI file
pub const UI_FILE_PATH: &str = concatcp!(GREETER_DIR, "/", GREETER_NAME, ".ui");

/// The directory where system-wide config files are located
const CONFIG_DIR: &str = env_or!("CONFIG_DIR", "/etc");

/// The greetd config directory
const GREETD_CONFIG_DIR: &str = env_or!("GREETD_CONFIG_DIR", concatcp!(CONFIG_DIR, "/greetd"));
/// Path to the config file
pub const CONFIG_PATH: &str = concatcp!(GREETD_CONFIG_DIR, "/", GREETER_NAME, ".toml");

/// The directory for system cache files
const CACHE_DIR: &str = env_or!("CACHE_DIR", "/var/cache");
/// Path to the cache file
pub const CACHE_PATH: &str = concatcp!(CACHE_DIR, "/", GREETER_NAME, "/cache.toml");

/// Path to the file that contains min/max UID of a regular user
pub const LOGIN_FILE: &str = concatcp!(CONFIG_DIR, "/login.defs");

/// Directories separated by `:`, containing desktop files for X11/Wayland sessions
pub const SESSION_DIRS: &str = env_or!(
    "SESSIONS_DIR",
    "/usr/share/xsessions:/usr/share/wayland-sessions"
);
