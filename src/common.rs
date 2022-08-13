//! Common stuff used by the codebase
use std::fs::read;
use std::path::Path;

use log::{info, warn};
use serde::de::DeserializeOwned;

/// Contains possible errors when loading/saving TOML from/to disk
#[derive(thiserror::Error, Debug)]
pub enum TOMLFileError {
    #[error("I/O error")]
    IO(#[from] std::io::Error),
    #[error("Error decoding TOML file contents")]
    TOMLDecode(#[from] toml::de::Error),
    #[error("Error encoding into TOML")]
    TOMLEncode(#[from] toml::ser::Error),
}

pub type TOMLFileResult<T> = Result<T, TOMLFileError>;

/// Load the TOML file from disk without any checks
fn load_raw_toml<T: DeserializeOwned>(path: &Path) -> TOMLFileResult<T> {
    Ok(toml::from_slice(read(path)?.as_slice())?)
}

/// Load the TOML file from disk
///
/// If loading fails, then this returns the default value of the struct
pub fn load_toml<T>(path: &str) -> T
where
    T: DeserializeOwned + Default,
{
    let path = Path::new(path);
    if path.exists() {
        match load_raw_toml(path) {
            Ok(item) => {
                info!("Loaded TOML file: {}", path.display());
                item
            }
            Err(err) => {
                warn!("Error loading TOML file '{}': {}", path.display(), err);
                T::default()
            }
        }
    } else {
        warn!("Missing TOML file: {}", path.display());
        T::default()
    }
}

/// Capitalize the first letter of the string
pub fn capitalize(string: &str) -> String {
    string[0..1].to_uppercase() + &string[1..]
}
