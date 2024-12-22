// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Convenient TOML loading utilities

use std::ffi::OsStr;
use std::fs::read;
use std::path::Path;

use serde::de::DeserializeOwned;

/// Contains possible errors when loading/saving TOML from/to disk
#[derive(thiserror::Error, Debug)]
pub enum TomlFileError {
    #[error("I/O error")]
    IO(#[from] std::io::Error),
    #[error("Error decoding UTF-8")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Error decoding TOML file contents")]
    TomlDecode(#[from] toml::de::Error),
    #[error("Error encoding into TOML")]
    TomlEncode(#[from] toml::ser::Error),
}

pub type TomlFileResult<T> = Result<T, TomlFileError>;

/// Load the TOML file from disk without any checks.
fn load_raw_toml<T: DeserializeOwned>(path: &Path) -> TomlFileResult<T> {
    Ok(toml::from_str(std::str::from_utf8(
        read(path)?.as_slice(),
    )?)?)
}

/// Load the TOML file from disk.
///
/// If loading fails, then this returns the default value of the struct.
pub fn load_toml<P, R>(path: &P) -> R
where
    P: AsRef<OsStr> + ?Sized,
    R: DeserializeOwned + Default,
{
    let path = Path::new(path);
    if path.exists() {
        match load_raw_toml(path) {
            Ok(item) => {
                info!("Loaded TOML file: {}", path.display());
                item
            }
            Err(err) => {
                warn!("Error loading TOML file '{}': {err}", path.display());
                R::default()
            }
        }
    } else {
        warn!("Missing TOML file: {}", path.display());
        R::default()
    }
}
