use std::path::PathBuf;

use confy::ConfyError;
use file_lock::{FileLock, FileOptions};

pub mod custom_event;
pub mod util;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub summaries: Vec<String>,
    pub descriptions: bool,
    pub frequency: u64,
    pub custom_events: Vec<(String, String)>,
}

pub fn load_config(path: &std::path::PathBuf) -> Result<Config, confy::ConfyError> {
    let options = file_lock::FileOptions::new().read(true);
    FileLock::lock(path, true, options).map_err(|error| ConfyError::GeneralLoadError(error))?;
    confy::load_path(path)
}

pub fn store_config(path: &PathBuf, config: Config) -> Result<(), ConfyError> {
    let options = FileOptions::new().write(true).create(true);
    FileLock::lock(path, true, options).map_err(|error| ConfyError::GeneralLoadError(error))?;
    confy::store_path(path, config)?;
    Ok(())
}
