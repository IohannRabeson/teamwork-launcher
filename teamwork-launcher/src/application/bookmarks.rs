use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::sync::Arc;
use crate::application::{IpPort, SettingsError};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Bookmarks {
    bookmarks: BTreeSet<IpPort>
}

impl Bookmarks {
    pub fn new() -> Self { Self { bookmarks: BTreeSet::new() } }
    pub fn add(&mut self, ip_port: IpPort) {
        self.bookmarks.insert(ip_port);
    }
    pub fn remove(&mut self, ip_port: &IpPort) {
        self.bookmarks.remove(ip_port);
    }
    pub fn is_bookmarked(&self, ip_port: &IpPort) -> bool { self.bookmarks.contains(ip_port) }
    pub fn write_file(&self, file_path: &std::path::Path) -> Result<(), SettingsError> {
        use std::io::Write;

        let json = serde_json::to_string(&self).map_err(|e| SettingsError::Json(Arc::new(e)))?;
        let mut file = std::fs::File::create(file_path).map_err(|e| SettingsError::Io(Arc::new(e)))?;

        file.write_all(json.as_bytes()).map_err(|e| SettingsError::Io(Arc::new(e)))?;
        Ok(())
    }

    pub fn read_file(file_path: &std::path::Path) -> Result<Self, SettingsError> {
        use std::io::Read;

        let mut file = std::fs::File::open(file_path).map_err(|e| SettingsError::Io(Arc::new(e)))?;

        Ok(serde_json::from_reader(file).map_err(|e| SettingsError::Json(Arc::new(e)))?)
    }
}