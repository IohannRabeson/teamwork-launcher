use std::{collections::HashSet, fs::File, io::Read, io::Write, path::Path, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Default, Serialize, Deserialize)]
pub struct UserSettings {
    pub favorites: HashSet<String>,
    pub filter: String,
}

impl UserSettings {
    const USER_SETTINGS_FILE_NAME: &'static str = "tf2-launcher";

    pub fn save_settings(settings: &UserSettings) -> Result<(), Error> {
        let json = serde_json::to_string(settings).map_err(|e| Error::Json(Arc::new(e)))?;
        let mut file = File::create(Self::USER_SETTINGS_FILE_NAME).map_err(|e| Error::Io(Arc::new(e)))?;

        file.write_all(json.as_bytes()).map_err(|e| Error::Io(Arc::new(e)))
    }

    pub fn load_settings() -> Result<UserSettings, Error> {
        if !Path::new(Self::USER_SETTINGS_FILE_NAME).is_file() {
            return Ok(UserSettings::default());
        }

        let mut file = File::open(Self::USER_SETTINGS_FILE_NAME).map_err(|e| Error::Io(Arc::new(e)))?;
        let mut json = String::new();

        file.read_to_string(&mut json).map_err(|e| Error::Io(Arc::new(e)))?;

        Ok(serde_json::from_str(&json).map_err(|e| Error::Json(Arc::new(e)))?)
    }
}
