// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Helper for system utilities like users and sessions

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{read, read_to_string};
use std::io;
use std::ops::ControlFlow;
use std::path::Path;
use std::str::from_utf8;

use glob::glob;
use pwd::Passwd;
use regex::Regex;

use crate::constants::{LOGIN_DEFS_PATHS, LOGIN_DEFS_UID_MAX, LOGIN_DEFS_UID_MIN, SESSION_DIRS};

/// XDG data directory variable name (parent directory for X11/Wayland sessions)
const XDG_DIR_ENV_VAR: &str = "XDG_DATA_DIRS";

// Convenient aliases for used maps
type UserMap = HashMap<String, String>;
type ShellMap = HashMap<String, Vec<String>>;
type SessionMap = HashMap<String, Vec<String>>;

/// Stores info of all regular users and sessions
pub struct SysUtil {
    /// Maps a user's full name to their system username
    users: UserMap,
    /// Maps a system username to their shell
    shells: ShellMap,
    /// Maps a session's full name to its command
    sessions: SessionMap,
}

impl SysUtil {
    pub fn new() -> io::Result<Self> {
        let path = (*LOGIN_DEFS_PATHS).iter().try_for_each(|path| {
            if let Ok(true) = AsRef::<Path>::as_ref(&path).try_exists() {
                ControlFlow::Break(path)
            } else {
                ControlFlow::Continue(())
            }
        });

        let normal_user = match path {
            ControlFlow::Break(path) => read_to_string(path)
                .map_err(|err| warn!("Failed to read '{path}': {err}"))
                .map(|text| NormalUser::parse_login_defs(&text))
                .unwrap_or_default(),
            ControlFlow::Continue(()) => {
                warn!("`login.defs` file not found in these paths: {LOGIN_DEFS_PATHS:?}",);

                NormalUser::default()
            }
        };

        debug!("{normal_user:?}");

        let (users, shells) = Self::init_users(normal_user)?;
        Ok(Self {
            users,
            shells,
            sessions: Self::init_sessions()?,
        })
    }

    /// Get the list of regular users.
    ///
    /// These are defined as a list of users with UID between `UID_MIN` and `UID_MAX`.
    fn init_users(normal_user: NormalUser) -> io::Result<(UserMap, ShellMap)> {
        let mut users = HashMap::new();
        let mut shells = HashMap::new();

        for entry in Passwd::iter().filter(|entry| normal_user.is_normal_user(entry.uid)) {
            // Use the actual system username if the "full name" is not available.
            let full_name = if let Some(gecos) = entry.gecos {
                if gecos.is_empty() {
                    debug!(
                        "Found user '{}' with UID '{}' and empty full name",
                        entry.name, entry.uid
                    );
                    entry.name.clone()
                } else {
                    // Only take first entry in gecos field.
                    let gecos_name_part: &str = gecos.split(',').next().unwrap_or(&gecos);
                    debug!(
                        "Found user '{}' with UID '{}' and full name: {gecos_name_part}",
                        entry.name, entry.uid
                    );
                    gecos_name_part.into()
                }
            } else {
                debug!(
                    "Found user '{}' with UID '{}' and missing full name",
                    entry.name, entry.uid
                );
                entry.name.clone()
            };
            users.insert(full_name, entry.name.clone());

            if let Some(cmd) = shlex::split(entry.shell.as_str()) {
                shells.insert(entry.name, cmd);
            } else {
                // Skip this user, since a missing command means that we can't use it.
                warn!(
                    "Couldn't split shell of username '{}' into arguments: {}",
                    entry.name, entry.shell
                );
            };
        }

        Ok((users, shells))
    }

    /// Get available X11 and Wayland sessions.
    ///
    /// These are defined as either X11 or Wayland session desktop files stored in specific
    /// directories.
    fn init_sessions() -> io::Result<SessionMap> {
        let mut found_session_names = HashSet::new();
        let mut sessions = HashMap::new();

        // Use the XDG spec if available, else use the one that's compiled.
        // The XDG env var can change after compilation in some distros like NixOS.
        let session_dirs = if let Ok(sess_parent_dirs) = env::var(XDG_DIR_ENV_VAR) {
            debug!("Found XDG env var {XDG_DIR_ENV_VAR}: {sess_parent_dirs}");
            match sess_parent_dirs
                .split(':')
                .map(|parent_dir| format!("{parent_dir}/xsessions:{parent_dir}/wayland-sessions"))
                .reduce(|a, b| a + ":" + &b)
            {
                None => SESSION_DIRS.to_string(),
                Some(dirs) => dirs,
            }
        } else {
            SESSION_DIRS.to_string()
        };

        for sess_dir in session_dirs.split(':') {
            let sess_parent_dir = if let Some(sess_parent_dir) = Path::new(sess_dir).parent() {
                sess_parent_dir
            } else {
                warn!("Session directory does not have a parent: {sess_dir}");
                continue;
            };
            debug!("Checking session directory: {sess_dir}");
            // Iterate over all '.desktop' files.
            for glob_path in glob(&format!("{sess_dir}/*.desktop"))
                .expect("Invalid glob pattern for session desktop files")
            {
                let path = match glob_path {
                    Ok(path) => path,
                    Err(err) => {
                        warn!("Error when globbing: {err}");
                        continue;
                    }
                };
                info!("Now scanning session file: {}", path.display());

                let contents = read(&path)?;
                let text = from_utf8(contents.as_slice()).unwrap_or_else(|err| {
                    panic!("Session file '{}' is not UTF-8: {}", path.display(), err)
                });

                let fname_and_type = match path.strip_prefix(sess_parent_dir) {
                    Ok(fname_and_type) => fname_and_type.to_owned(),
                    Err(err) => {
                        warn!("Error with file name: {err}");
                        continue;
                    }
                };

                if found_session_names.contains(&fname_and_type) {
                    debug!(
                        "{fname_and_type:?} was already found elsewhere, skipping {}",
                        path.display()
                    );
                    continue;
                };

                // The session launch command is specified as: Exec=command arg1 arg2...
                let cmd_regex =
                    Regex::new(r"Exec=(.*)").expect("Invalid regex for session command");
                // The session name is specified as: Name=My Session
                let name_regex = Regex::new(r"Name=(.*)").expect("Invalid regex for session name");

                // Hiding could be either as Hidden=true or NoDisplay=true
                let hidden_regex = Regex::new(r"Hidden=(.*)").expect("Invalid regex for hidden");
                let no_display_regex =
                    Regex::new(r"NoDisplay=(.*)").expect("Invalid regex for no display");

                let hidden: bool = if let Some(hidden_str) = hidden_regex
                    .captures(text)
                    .and_then(|capture| capture.get(1))
                {
                    hidden_str.as_str().parse().unwrap_or(false)
                } else {
                    false
                };

                let no_display: bool = if let Some(no_display_str) = no_display_regex
                    .captures(text)
                    .and_then(|capture| capture.get(1))
                {
                    no_display_str.as_str().parse().unwrap_or(false)
                } else {
                    false
                };

                if hidden | no_display {
                    found_session_names.insert(fname_and_type);
                    continue;
                };

                // Parse the desktop file to get the session command.
                let cmd = if let Some(cmd_str) =
                    cmd_regex.captures(text).and_then(|capture| capture.get(1))
                {
                    if let Some(cmd) = shlex::split(cmd_str.as_str()) {
                        cmd
                    } else {
                        warn!(
                            "Couldn't split command of '{}' into arguments: {}",
                            path.display(),
                            cmd_str.as_str()
                        );
                        // Skip the desktop file, since a missing command means that we can't
                        // use it.
                        continue;
                    }
                } else {
                    warn!("No command found for session: {}", path.display());
                    // Skip the desktop file, since a missing command means that we can't use it.
                    continue;
                };

                // Get the full name of this session.
                let name = if let Some(name) =
                    name_regex.captures(text).and_then(|capture| capture.get(1))
                {
                    debug!(
                        "Found name '{}' for session '{}' with command '{:?}'",
                        name.as_str(),
                        path.display(),
                        cmd
                    );
                    name.as_str()
                } else if let Some(stem) = path.file_stem() {
                    // Get the stem of the filename of this desktop file.
                    // This is used as backup, in case the file name doesn't exist.
                    if let Some(stem) = stem.to_str() {
                        debug!(
                            "Using file stem '{stem}', since no name was found for session: {}",
                            path.display()
                        );
                        stem
                    } else {
                        warn!("Non-UTF-8 file stem in session file: {}", path.display());
                        // No way to display this session name, so just skip it.
                        continue;
                    }
                } else {
                    warn!("No file stem found for session: {}", path.display());
                    // No file stem implies no file name, which shouldn't happen.
                    // Since there's no full name nor file stem, just skip this anomalous
                    // session.
                    continue;
                };
                found_session_names.insert(fname_and_type);
                sessions.insert(name.to_string(), cmd);
            }
        }

        Ok(sessions)
    }

    /// Get the mapping of a user's full name to their system username.
    ///
    /// If the full name is not available, their system username is used.
    pub fn get_users(&self) -> &UserMap {
        &self.users
    }

    /// Get the mapping of a system username to their shell.
    pub fn get_shells(&self) -> &ShellMap {
        &self.shells
    }

    /// Get the mapping of a session's full name to its command.
    ///
    /// If the full name is not available, the filename stem is used.
    pub fn get_sessions(&self) -> &SessionMap {
        &self.sessions
    }
}

/// A named tuple of min and max that stores UID limits for normal users.
///
/// Use [`Self::parse_login_defs`] to obtain the system configuration. If the file is missing or there are
/// parsing errors a fallback of [`Self::default`] should be used.
#[derive(Debug, PartialEq, Eq)]
struct NormalUser {
    min_uid: u64,
    max_uid: u64,
}

impl Default for NormalUser {
    fn default() -> Self {
        Self {
            min_uid: *LOGIN_DEFS_UID_MIN,
            max_uid: *LOGIN_DEFS_UID_MAX,
        }
    }
}

impl NormalUser {
    /// Parses the `login.defs` file content and looks for `UID_MIN` and `UID_MAX` definitions. If a definition is
    /// missing or causes parsing errors, the default values [`struct@LOGIN_DEFS_UID_MIN`] and
    /// [`struct@LOGIN_DEFS_UID_MAX`] are used.
    ///
    /// This parser is highly specific to parsing the 2 required values, thus it focuses on doing the least amout of
    /// compute required to extracting them.
    ///
    /// Errors are dropped because they are unlikely and their handling would result in the use of default values
    /// anyway.
    pub fn parse_login_defs(text: &str) -> Self {
        let mut min = None;
        let mut max = None;

        for line in text.lines().map(str::trim) {
            const KEY_LENGTH: usize = "UID_XXX".len();

            // At MSRV 1.80 you could use `split_at_checked`, this is just a way to not raise it.
            // This checks if the string is of sufficient length too.
            if !line.is_char_boundary(KEY_LENGTH) {
                continue;
            }
            let (key, val) = line.split_at(KEY_LENGTH);

            if !val.starts_with(char::is_whitespace) {
                continue;
            }

            match (key, min, max) {
                ("UID_MIN", None, _) => min = Self::parse_number(val),
                ("UID_MAX", _, None) => max = Self::parse_number(val),
                _ => continue,
            }

            if min.is_some() && max.is_some() {
                break;
            }
        }

        Self {
            min_uid: min.unwrap_or(*LOGIN_DEFS_UID_MIN),
            max_uid: max.unwrap_or(*LOGIN_DEFS_UID_MAX),
        }
    }

    /// Parses a number value in a `/etc/login.defs` entry. As per the manpage:
    ///
    /// - `0x` prefix: hex number
    /// - `0` prefix: octal number
    /// - starts with `1..9`: decimal number
    ///
    /// In case the string value is not parsable as a number the entry value is considered invalid and `None` is
    /// returned.
    fn parse_number(num: &str) -> Option<u64> {
        let num = num.trim();
        if num == "0" {
            return Some(0);
        }

        if let Some(octal) = num.strip_prefix('0') {
            if let Some(hex) = octal.strip_prefix('x') {
                return u64::from_str_radix(hex, 16).ok();
            }

            return u64::from_str_radix(octal, 8).ok();
        }

        num.parse().ok()
    }

    // Returns true for regular users, false for those outside the UID limit, eg. git or root.
    pub fn is_normal_user<T>(&self, uid: T) -> bool
    where
        T: Into<u64>,
    {
        (self.min_uid..=self.max_uid).contains(&uid.into())
    }
}

#[cfg(test)]
mod tests {
    #[allow(non_snake_case)]
    mod UidLimit {
        use super::super::*;

        #[test_case(
            &["UID_MIN 1", "UID_MAX 10"].join("\n")
            => NormalUser { min_uid: 1, max_uid: 10 };
            "both configured"
        )]
        #[test_case(
            &["UID_MAX 10", "UID_MIN 1"].join("\n")
            => NormalUser { min_uid: 1, max_uid: 10 };
            "reverse order"
        )]
        #[test_case(
            &["OTHER 20",
            "# Comment",
            "",
            "UID_MAX 10",
            "UID_MIN 1",
            "MORE_TEXT 40"].join("\n")
            => NormalUser { min_uid: 1, max_uid: 10 };
            "complex file"
        )]
        #[test_case(
            "UID_MAX10"
            => NormalUser::default();
            "no space"
        )]
        #[test_case(
            "SUB_UID_MAX 10"
            => NormalUser::default();
            "invalid field (with prefix)"
        )]
        #[test_case(
            "UID_MAX_BLAH 10"
            => NormalUser::default();
            "invalid field (with suffix)"
        )]
        fn parse_login_defs(text: &str) -> NormalUser {
            NormalUser::parse_login_defs(text)
        }

        #[test_case("" => None; "empty")]
        #[test_case("no" => None; "string")]
        #[test_case("0" => Some(0); "zero")]
        #[test_case("0x" => None; "0x isn't a hex number")]
        #[test_case("10" => Some(10); "decimal")]
        #[test_case("0777" => Some(0o777); "octal")]
        #[test_case("0xDeadBeef" => Some(0xdead_beef); "hex")]
        fn parse_number(num: &str) -> Option<u64> {
            NormalUser::parse_number(num)
        }
    }
}
