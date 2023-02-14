use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::application::SettingsError;

#[derive(Serialize, Deserialize, Default)]
pub struct UserSettings {
    pub teamwork_api_key: String,
    pub steam_executable_path: String,
}

impl UserSettings {
    pub fn write_file(&self, file_path: &std::path::Path) -> Result<(), SettingsError> {
        use std::io::Write;

        let json = serde_json::to_string(&self).map_err(|e|SettingsError::Json(Arc::new(e)))?;
        let mut file = std::fs::File::create(file_path).map_err(|e|SettingsError::Io(Arc::new(e)))?;

        file.write_all(json.as_bytes()).map_err(|e|SettingsError::Io(Arc::new(e)))?;
        Ok(())
    }

    pub fn read_file(file_path: &std::path::Path) -> Result<Self, SettingsError> {
        use std::io::Read;

        let mut file = std::fs::File::open(file_path).map_err(|e|SettingsError::Io(Arc::new(e)))?;

        Ok(serde_json::from_reader(file).map_err(|e|SettingsError::Json(Arc::new(e)))?)
    }
}
