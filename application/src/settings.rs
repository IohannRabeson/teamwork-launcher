use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use {
    log::{error, info},
    serde::{Deserialize, Serialize},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] Arc<serde_json::Error>),
    #[error("IO error: {0}")]
    Io(#[from] Arc<std::io::Error>),
}

#[derive(Serialize, Deserialize)]
pub struct UserSettings {
    pub favorites: BTreeSet<String>,
    pub filter: String,
    pub game_executable_path: OsString,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            favorites: Default::default(),
            filter: Default::default(),
            game_executable_path: r"C:\Program Files (x86)\Steam\Steam.exe".into(),
        }
    }
}

impl UserSettings {
    fn file_settings_path(create_directory: bool) -> Option<PathBuf> {
        let mut path = platform_dirs::AppDirs::new("tf2-launcher".into(), false)
            .map(|dirs| dirs.config_dir)
            .expect("config directory path");

        if !path.exists() && create_directory {
            info!("Create directory '{}'", path.to_string_lossy());

            if let Err(error) = std::fs::create_dir_all(&path) {
                error!("Unable to create directory '{}': {}", path.to_string_lossy(), error);
                return None;
            }
        }

        path.push("settings.json");
        Some(path)
    }

    pub fn save_settings(settings: &UserSettings) -> Result<(), Error> {
        let json = serde_json::to_string(settings).map_err(|e| Error::Json(Arc::new(e)))?;
        if let Some(settings_file_path) = Self::file_settings_path(true) {
            info!("Write settings '{}'", settings_file_path.to_string_lossy());

            let mut file = File::create(settings_file_path).map_err(|e| Error::Io(Arc::new(e)))?;

            file.write_all(json.as_bytes()).map_err(|e| Error::Io(Arc::new(e)))
        } else {
            error!("Failed to get the file settings path");
            // We can't get the directory path or we can't create the directory to store
            // the settings file. In this case we just give up silently.
            Ok(())
        }
    }

    pub fn load_settings() -> Result<UserSettings, Error> {
        if let Some(settings_file_path) = Self::file_settings_path(false) {
            info!("Read settings '{}'", settings_file_path.to_string_lossy());

            if !Path::new(&settings_file_path).is_file() {
                info!("File '{}' does not exists", settings_file_path.to_string_lossy());
                return Ok(UserSettings::default());
            }

            let mut file = File::open(settings_file_path).map_err(|e| Error::Io(Arc::new(e)))?;
            let mut json = String::new();

            file.read_to_string(&mut json).map_err(|e| Error::Io(Arc::new(e)))?;

            serde_json::from_str(&json).map_err(|e| Error::Json(Arc::new(e)))
        } else {
            // We can't get the directory path or we can't create the directory to store
            // the settings file. In this case we just give up silently.
            error!("Failed to get the file settings path");
            Ok(UserSettings::default())
        }
    }
}
