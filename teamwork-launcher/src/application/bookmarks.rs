use {
    crate::application::{servers_source::SourceKey, IpPort, SettingsError},
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    std::{
        collections::BTreeSet,
        io::{Read, Write},
        path::Path,
        sync::Arc,
    },
};

#[derive(Serialize, Deserialize, Default)]
pub struct Bookmarks {
    bookmarks: BTreeSet<IpPort>,
    source_keys: BTreeSet<SourceKey>,
}

impl Bookmarks {
    pub fn new() -> Self {
        Self {
            bookmarks: BTreeSet::new(),
            source_keys: BTreeSet::new(),
        }
    }
    pub fn add(&mut self, ip_port: IpPort, source_key: SourceKey) {
        self.bookmarks.insert(ip_port);
        self.source_keys.insert(source_key);
    }

    pub fn remove(&mut self, ip_port: &IpPort, source_key: &SourceKey) {
        self.bookmarks.remove(ip_port);
        self.source_keys.remove(source_key);
    }
    pub fn is_bookmarked(&self, ip_port: &IpPort) -> bool {
        self.bookmarks.contains(ip_port)
    }
    pub fn is_source_bookmarked(&self, source_key: &SourceKey) -> bool {
        self.source_keys.contains(source_key)
    }
}

pub fn write_file(settings: &impl Serialize, file_path: impl AsRef<Path>) -> Result<(), SettingsError> {
    use std::io::Write;

    let json = serde_json::to_string(settings).map_err(|e| SettingsError::Json(Arc::new(e)))?;
    let mut file = std::fs::File::create(file_path).map_err(|e| SettingsError::Io(Arc::new(e)))?;

    file.write_all(json.as_bytes()).map_err(|e| SettingsError::Io(Arc::new(e)))?;
    Ok(())
}

pub fn read_file<'de, S>(file_path: impl AsRef<Path>) -> Result<S, SettingsError>
where
    S: DeserializeOwned + Default,
{
    use std::io::Read;

    let mut file = std::fs::File::open(file_path).map_err(|e| SettingsError::Io(Arc::new(e)))?;

    Ok(serde_json::from_reader(file).map_err(|e| SettingsError::Json(Arc::new(e)))?)
}
