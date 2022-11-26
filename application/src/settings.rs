use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use {
    async_rwlock::RwLock,
    serde::{Deserializer, Serializer},
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct InnerUserSettings {
    pub favorites: BTreeSet<String>,
    #[serde(rename = "filter_text")]
    pub servers_filter_text: String,
    pub game_executable_path: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct UserSettings {
    #[serde(serialize_with = "rwlock_serde_serialize")]
    #[serde(deserialize_with = "rwlock_serde_deserialize")]
    storage: RwLock<InnerUserSettings>,
}

impl Clone for UserSettings {
    fn clone(&self) -> Self {
        let original_inner = self.storage.try_read().unwrap();

        Self {
            storage: RwLock::new(original_inner.clone()),
        }
    }
}

fn rwlock_serde_serialize<S>(val: &RwLock<InnerUserSettings>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    InnerUserSettings::serialize(&val.try_read().expect("rwlock_serde lock for read"), s)
}

fn rwlock_serde_deserialize<'de, D>(d: D) -> Result<RwLock<InnerUserSettings>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(RwLock::new(InnerUserSettings::deserialize(d)?))
}

impl Default for InnerUserSettings {
    fn default() -> Self {
        Self {
            favorites: Default::default(),
            servers_filter_text: Default::default(),
            #[cfg(target_os = "windows")]
            game_executable_path: r"C:\Program Files (x86)\Steam\Steam.exe".into(),
            #[cfg(not(target_os = "windows"))]
            game_executable_path: Default::default(),
        }
    }
}

impl UserSettings {
    pub fn set_filter_servers_text<S: AsRef<str>>(&mut self, text: S) {
        let mut inner = self.storage.try_write().unwrap();

        inner.servers_filter_text = text.as_ref().to_string()
    }

    pub fn servers_filter_text(&self) -> String {
        let inner = self.storage.try_read().unwrap();

        inner.servers_filter_text.clone()
    }

    pub fn filter_servers_by_text<S: AsRef<str>>(&self, name: S) -> bool {
        let inner = self.storage.try_read().unwrap();
        let text_filter = &inner.servers_filter_text.trim().to_lowercase();

        if text_filter.is_empty() {
            return true;
        }

        name.as_ref().to_lowercase().contains(text_filter)
    }

    pub fn filter_servers_favorite<S: AsRef<str>>(&self, name: S) -> bool {
        let inner = self.storage.try_read().unwrap();

        inner.favorites.contains(name.as_ref())
    }

    pub fn set_game_executable_path<S: AsRef<str>>(&self, path: S) {
        let mut inner = self.storage.try_write().unwrap();

        inner.game_executable_path = path.as_ref().to_string();
    }

    pub fn game_executable_path(&self) -> String {
        let inner = self.storage.try_read().unwrap();

        inner.game_executable_path.clone()
    }

    pub fn switch_favorite_server<S: AsRef<str>>(&mut self, name: S) {
        let mut inner = self.storage.try_write().unwrap();
        let favorites = &mut inner.favorites;
        let name = name.as_ref();

        match favorites.contains(name) {
            true => favorites.remove(name),
            false => favorites.insert(name.to_string()),
        };
    }

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
        let json = serde_json::to_string(&settings).map_err(|e| Error::Json(Arc::new(e)))?;
        if let Some(settings_file_path) = Self::file_settings_path(true) {
            info!("Write settings '{}'", settings_file_path.to_string_lossy());

            let mut file = File::create(settings_file_path).map_err(|e| Error::Io(Arc::new(e)))?;

            return file.write_all(json.as_bytes()).map_err(|e| Error::Io(Arc::new(e)));
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
