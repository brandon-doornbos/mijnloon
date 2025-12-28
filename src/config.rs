use std::path::PathBuf;

use confy::ConfyError;
use file_lock::{FileLock, FileOptions};

use crate::schedule::Event;

#[derive(Default, serde::Deserialize, serde::Serialize, Clone)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub summaries: Vec<String>,
    pub frequency: u64,
    pub events: Vec<Event>,
    pub custom_events: Vec<Event>,
}

pub fn load(path: &std::path::PathBuf) -> Result<Config, confy::ConfyError> {
    let options = file_lock::FileOptions::new().read(true);
    FileLock::lock(path, true, options).map_err(|error| ConfyError::GeneralLoadError(error))?;
    confy::load_path(path)
}

pub fn store(path: &PathBuf, config: Config) -> Result<(), ConfyError> {
    let options = FileOptions::new().write(true).create(true);
    FileLock::lock(path, true, options).map_err(|error| ConfyError::GeneralLoadError(error))?;
    confy::store_path(path, config)?;
    Ok(())
}
