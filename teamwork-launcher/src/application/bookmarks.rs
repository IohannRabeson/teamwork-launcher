use {
    crate::application::{IpPort, servers_source::SourceKey},
    serde::{Deserialize, Serialize},
    std::collections::BTreeSet,
};

#[derive(Serialize, Deserialize, Default)]
pub struct Bookmarks {
    bookmarks: BTreeSet<IpPort>,
    source_keys: BTreeSet<SourceKey>,
}

impl Bookmarks {
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
}
