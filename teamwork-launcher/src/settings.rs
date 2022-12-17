use crate::{directories, advanced_filter::AdvancedServerFilter, servers_sources::ServersSources, text_filter::TextFilter};

use {
    crate::{
        models::{IpPort, Server},
        sources::SourceKey,
    },
    serde_with::serde_as,
    std::{
        collections::{btree_map::Entry::Occupied, BTreeMap, BTreeSet},
        fs::File,
        io::{Read, Write},
        path::Path,
        sync::Arc,
    },
};

use {
    async_rwlock::RwLock,
    log::{error, info},
    serde::{Deserialize, Deserializer, Serialize, Serializer},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] Arc<serde_json::Error>),
    #[error("IO error: {0}")]
    Io(#[from] Arc<std::io::Error>),
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
struct InnerUserSettings {
    /// Favorites servers
    /// This map store the IP and the port, with the source key. This allows to query only the source
    /// for the favorites servers.
    #[serde(default)]
    // It's needed to convert this BTreeMap to a Vec to avoid the error where serde_json try to
    // write invalid JSON with an invalid key.
    #[serde_as(as = "Vec<(_, _)>")]
    pub favorites: BTreeMap<IpPort, Option<SourceKey>>,
    #[serde(default)]
    pub servers_text_filter: TextFilter,
    #[serde(default)]
    pub servers_filter: AdvancedServerFilter,
    #[serde(default)]
    pub servers_source_filter: ServersSources,
    #[serde(default)]
    pub game_executable_path: String,
    #[serde(default)]
    pub teamwork_api_key: String,
    #[serde(default)]
    pub auto_refresh_favorite: bool,
    #[serde(default)]
    pub quit_on_launch: bool,
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
            servers_text_filter: Default::default(),
            servers_source_filter: Default::default(),
            #[cfg(target_os = "windows")]
            game_executable_path: r"C:\Program Files (x86)\Steam\Steam.exe".into(),
            #[cfg(not(target_os = "windows"))]
            game_executable_path: Default::default(),
            teamwork_api_key: Default::default(),
            auto_refresh_favorite: true,
            servers_filter: Default::default(),
            quit_on_launch: false,
        }
    }
}

impl UserSettings {
    pub fn set_quit_on_launch(&mut self, value: bool) {
        let mut inner = self.storage.try_write().unwrap();

        inner.quit_on_launch = value;
    }

    pub fn quit_on_launch(&self) -> bool {
        let inner = self.storage.try_read().unwrap();

        inner.quit_on_launch
    }

    pub fn set_minimum_players_count(&mut self, value: u8) {
        let mut inner = self.storage.try_write().unwrap();

        inner.servers_filter.minimum_players_count = value;
    }

    pub fn server_filter(&self) -> AdvancedServerFilter {
        let inner = self.storage.try_read().unwrap();

        inner.servers_filter.clone()
    }

    pub fn set_available_sources(&mut self, all_source_keys: impl Iterator<Item = (String, SourceKey)>) {
        let mut inner = self.storage.try_write().unwrap();

        inner.servers_source_filter.set_available_sources(all_source_keys);
    }

    pub fn check_source_filter(&mut self, key: &SourceKey, checked: bool) {
        let mut inner = self.storage.try_write().unwrap();

        inner.servers_source_filter.check_source(key, checked);
    }

    pub fn source_filter(&self) -> Vec<(String, SourceKey, bool)> {
        let inner = self.storage.try_read().unwrap();

        inner
            .servers_source_filter
            .sources()
            .map(|(name, key, checked)| (name, key, checked))
            .collect()
    }

    pub fn set_teamwork_api_key<S: AsRef<str>>(&mut self, api_key: S) {
        let mut inner = self.storage.try_write().unwrap();

        inner.teamwork_api_key = api_key.as_ref().to_string();
    }

    pub fn teamwork_api_key(&self) -> String {
        let inner = self.storage.try_write().unwrap();

        inner.teamwork_api_key.clone()
    }

    pub fn set_filter_servers_text<S: AsRef<str>>(&mut self, text: S) {
        let mut inner = self.storage.try_write().unwrap();

        inner.servers_text_filter.set_text(text.as_ref());
    }

    pub fn servers_filter_text(&self) -> String {
        let inner = self.storage.try_read().unwrap();

        inner.servers_text_filter.text().to_string()
    }

    pub fn filter_servers(&self, server: &Server) -> bool {
        let inner = self.storage.try_read().unwrap();

        inner.servers_text_filter.accept(&server.name.to_lowercase())
            && inner.servers_source_filter.accept_server(server)
            && inner.servers_filter.accept_server(server)
    }

    pub fn filter_servers_favorite(&self, server: &Server) -> bool {
        let inner = self.storage.try_read().unwrap();

        inner.favorites.contains_key(&server.ip_port)
    }

    pub fn switch_favorite_server(&mut self, ip_port: IpPort, source_key: Option<SourceKey>) {
        let mut inner = self.storage.try_write().unwrap();
        let favorites = &mut inner.favorites;

        match favorites.contains_key(&ip_port) {
            true => favorites.remove(&ip_port),
            false => favorites.insert(ip_port, source_key),
        };
    }

    pub fn favorite_source_keys(&self) -> BTreeSet<SourceKey> {
        let inner = self.storage.try_read().unwrap();

        inner.favorites.iter().filter_map(|(_, source)| source.clone()).collect()
    }

    pub fn checked_source_keys(&self) -> BTreeSet<SourceKey> {
        let inner = self.storage.try_read().unwrap();

        inner.servers_source_filter.checked_sources().cloned().collect()
    }

    /// Update the information about the favorites servers.
    pub fn update_favorites<'a>(&mut self, servers: impl Iterator<Item = &'a Server>) {
        let mut inner = self.storage.try_write().unwrap();

        for server in servers {
            if let Occupied(mut source) = inner.favorites.entry(server.ip_port.clone()) {
                source.insert(server.source.clone());
            }
        }
    }

    pub fn set_game_executable_path<S: AsRef<str>>(&mut self, path: S) {
        let mut inner = self.storage.try_write().unwrap();

        inner.game_executable_path = path.as_ref().to_string();
    }

    pub fn game_executable_path(&self) -> String {
        let inner = self.storage.try_read().unwrap();

        inner.game_executable_path.clone()
    }

    pub fn auto_refresh_favorite(&self) -> bool {
        let inner = self.storage.try_read().unwrap();

        inner.auto_refresh_favorite
    }

    pub fn set_auto_refresh_favorite(&mut self, value: bool) {
        let mut inner = self.storage.try_write().unwrap();

        inner.auto_refresh_favorite = value;
    }

    pub fn save_settings(settings: &UserSettings) -> Result<(), Error> {
        let settings_file_path = directories::get_settings_file_path();

        info!("Write settings '{}'", settings_file_path.to_string_lossy());

        let json = serde_json::to_string(&settings).map_err(|e| Error::Json(Arc::new(e)))?;
        let mut file = File::create(settings_file_path).map_err(|e| Error::Io(Arc::new(e)))?;

        file.write_all(json.as_bytes()).map_err(|e| Error::Io(Arc::new(e)))
    }

    pub fn load_settings() -> Result<UserSettings, Error> {
        let settings_file_path = directories::get_settings_file_path();

        info!("Read settings '{}'", settings_file_path.to_string_lossy());

        if !Path::new(&settings_file_path).is_file() {
            info!("File '{}' does not exists", settings_file_path.to_string_lossy());
            return Ok(UserSettings::default());
        }

        let mut file = File::open(settings_file_path).map_err(|e| Error::Io(Arc::new(e)))?;
        let mut json = String::new();

        file.read_to_string(&mut json).map_err(|e| Error::Io(Arc::new(e)))?;

        serde_json::from_str(&json).map_err(|e| Error::Json(Arc::new(e)))
    }
}
