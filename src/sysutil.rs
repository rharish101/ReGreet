// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Helper for system utilities like users and sessions

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs::read;
use std::io::Result as IOResult;
use std::path::Path;
use std::str::from_utf8;

use glob::glob;
use pwd::Passwd;
use regex::Regex;
use shlex::Shlex;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::constants::{WAYLAND_SESSION_DIRS, XSESSION_DIRS};

/// Path to the file that contains min/max UID of a regular user
pub const LOGIN_FILE: &str = "/etc/login.defs";
/// Default minimum UID for `useradd` (a/c to my system)
const DEFAULT_UID_MIN: u32 = 1000;
/// Default maximum UID for `useradd` (a/c to my system)
const DEFAULT_UID_MAX: u32 = 60000;
/// XDG data directory variable name (parent directory for X11/Wayland sessions)
const XDG_DIR_ENV_VAR: &str = "XDG_DATA_DIRS";

#[derive(Clone, Copy)]
pub enum SessionType {
    X11,
    Wayland,
    Unknown,
}

#[derive(Clone)]
pub struct SessionInfo {
    pub command: Vec<String>,
    pub sess_type: SessionType,
}

// Convenient aliases for used maps
type UserMap = HashMap<String, String>;
type ShellMap = HashMap<String, Vec<String>>;
type SessionMap = HashMap<String, SessionInfo>;

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
    pub fn new(config: &Config) -> IOResult<Self> {
        let (users, shells) = Self::init_users()?;
        let x11_cmd_prefix = config.get_sys_commands().x11_prefix.clone();
        let mut sessions =
            Self::init_sessions(XSESSION_DIRS, "xsessions", x11_cmd_prefix, SessionType::X11)?;
        sessions.extend(Self::init_sessions(
            WAYLAND_SESSION_DIRS,
            "wayland-sessions",
            Vec::new(),
            SessionType::Wayland,
        )?);
        Ok(Self {
            users,
            shells,
            sessions,
        })
    }

    /// Get the min and max UID for the current system.
    fn get_uid_limits() -> IOResult<(u32, u32)> {
        let contents = read(LOGIN_FILE)?;
        let text = from_utf8(contents.as_slice())
            .unwrap_or_else(|err| panic!("Login file '{LOGIN_FILE}' is not UTF-8: {err}"));

        // UID_MIN/MAX are limits to a UID for a regular user i.e. a user created with `useradd`.
        // Thus, to find regular users, we filter the list of users with these UID limits.
        let min_uid_regex = Regex::new(r"\nUID_MIN\s+([0-9]+)").expect("Invalid regex for UID_MIN");
        let max_uid_regex = Regex::new(r"\nUID_MAX\s+([0-9]+)").expect("Invalid regex for UID_MAX");

        // Get UID_MIN.
        let min_uid = if let Some(num) = min_uid_regex
            .captures(text)
            .and_then(|capture| capture.get(1))
        {
            num.as_str()
                .parse()
                .expect("UID_MIN regex didn't capture an integer")
        } else {
            warn!("Failed to find UID_MIN in login file: {LOGIN_FILE}");
            DEFAULT_UID_MIN
        };

        // Get UID_MAX.
        let max_uid = if let Some(num) = max_uid_regex
            .captures(text)
            .and_then(|capture| capture.get(1))
        {
            num.as_str()
                .parse()
                .expect("UID_MAX regex didn't capture an integer")
        } else {
            warn!("Failed to find UID_MAX in login file: {LOGIN_FILE}");
            DEFAULT_UID_MAX
        };

        Ok((min_uid, max_uid))
    }

    /// Get the list of regular users.
    ///
    /// These are defined as a list of users with UID between `UID_MIN` and `UID_MAX`.
    fn init_users() -> IOResult<(UserMap, ShellMap)> {
        let (min_uid, max_uid) = Self::get_uid_limits()?;
        debug!("UID_MIN: {min_uid}, UID_MAX: {max_uid}");

        let mut users = HashMap::new();
        let mut shells = HashMap::new();

        // Iterate over all users in /etc/passwd.
        for entry in Passwd::iter() {
            if entry.uid > max_uid || entry.uid < min_uid {
                // Non-standard user, eg. git or root
                continue;
            };

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
    fn init_sessions(
        default_sess_dirs: &str,
        xdg_sess_dir: &str,
        cmd_prefix: Vec<String>,
        sess_type: SessionType,
    ) -> IOResult<SessionMap> {
        let mut found_session_names = HashSet::new();
        let mut sessions = HashMap::new();

        // Use the XDG spec if available, else use the one that's compiled.
        // The XDG env var can change after compilation in some distros like NixOS.
        let session_dirs = if let Ok(sess_parent_dirs) = env::var(XDG_DIR_ENV_VAR) {
            debug!("Found XDG env var {XDG_DIR_ENV_VAR}: {sess_parent_dirs}");
            match sess_parent_dirs
                .split(':')
                .map(|parent_dir| format!("{parent_dir}/{xdg_sess_dir}"))
                .reduce(|a, b| a + ":" + &b)
            {
                None => default_sess_dirs.to_string(),
                Some(dirs) => dirs,
            }
        } else {
            default_sess_dirs.to_string()
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
                    let mut cmd = cmd_prefix.clone();
                    cmd.extend(Shlex::new(cmd_str.as_str()));
                    if cmd.len() > cmd_prefix.len() {
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
                sessions.insert(
                    name.to_string(),
                    SessionInfo {
                        command: cmd,
                        sess_type,
                    },
                );
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
