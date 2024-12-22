// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Utility for caching info between logins

mod lru;

use std::fs::{create_dir_all, write};
use std::num::NonZeroUsize;
use std::path::Path;

use serde::{Deserialize, Serialize};

use self::lru::LruCache;
use crate::constants::CACHE_PATH;
use crate::tomlutils::{load_toml, TomlFileResult};

/// Limit to the size of the user to last-used session mapping.
const CACHE_LIMIT: usize = 100;

/// Holds info needed to persist between logins
#[derive(Deserialize, Serialize)]
pub struct Cache {
    /// The last user who logged in
    last_user: Option<String>,
    /// The last-used session for each user
    user_to_last_sess: LruCache<String, String>,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            last_user: None,
            user_to_last_sess: LruCache::new(CACHE_LIMIT),
        }
    }
}

impl Cache {
    /// Load the cache file from disk.
    pub fn new() -> Self {
        let mut cache: Self = load_toml(CACHE_PATH);
        // Make sure that the LRU can contain the needed amount of mappings.
        cache
            .user_to_last_sess
            .resize(NonZeroUsize::new(CACHE_LIMIT).expect("Cache limit cannot be zero"));
        cache
    }

    /// Save the cache file to disk.
    pub fn save(&self) -> TomlFileResult<()> {
        let cache_path = Path::new(CACHE_PATH);
        if !cache_path.exists() {
            // Create the cache directory.
            if let Some(cache_dir) = cache_path.parent() {
                info!("Creating missing cache directory: {}", cache_dir.display());
                create_dir_all(cache_dir)?;
            };
        }

        info!("Saving cache to disk");
        write(cache_path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Get the last user to login.
    pub fn get_last_user(&self) -> Option<&str> {
        self.last_user.as_deref()
    }

    /// Get the last used session by the given user.
    pub fn get_last_session(&mut self, user: &str) -> Option<&str> {
        self.user_to_last_sess.get(user).map(String::as_str)
    }

    /// Set the last user to login.
    pub fn set_last_user(&mut self, user: &str) {
        self.last_user = Some(String::from(user));
    }

    /// Set the last used session by the given user.
    pub fn set_last_session(&mut self, user: &str, session: &str) {
        self.user_to_last_sess
            .push(String::from(user), String::from(session));
    }
}
