use std::collections::BTreeSet;

///! A source of data (a list of servers)
use {
    crate::{models::Server, sources::SourceKey},
    serde::{Deserialize, Serialize},
    std::collections::BTreeMap,
};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ServersSources {
    sources: BTreeMap<SourceKey, Entry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Entry {
    /// If enabled, this source will be used when refreshing the list of servers.
    pub enabled: bool,
    /// The displayable name of the source
    pub name: String,
}

impl ServersSources {
    pub fn accept_server(&self, server: &Server) -> bool {
        match &server.source {
            Some(key) => self.sources.get(key).map(|entry| entry.enabled).unwrap_or(false),
            None => false,
        }
    }

    pub fn set_available_sources(&mut self, all_source_keys: impl Iterator<Item = (String, SourceKey)>) {
        use std::collections::btree_map::Entry::{Occupied, Vacant};

        let all_source_keys: Vec<(String, SourceKey)> = all_source_keys.collect();

        for (init_name, init_key) in all_source_keys.iter() {
            match self.sources.entry(init_key.clone()) {
                Vacant(entry) => {
                    entry.insert(Entry {
                        name: init_name.clone(),
                        enabled: true,
                    });
                }
                Occupied(mut entry) => {
                    entry.get_mut().name = init_name.clone();
                }
            }
        }

        self.remove_unknown_sources(all_source_keys);
    }

    /// Remove unknown sources by checking if the key is present in the source provider.
    fn remove_unknown_sources(&mut self, all_source_keys: Vec<(String, SourceKey)>) {
        let keys_index: BTreeSet<&SourceKey> = all_source_keys.iter().map(|(_k, v)| v).collect();
        // TODO: use drain_filter when available - https://github.com/rust-lang/rust/issues/70530
        self.sources = self
            .sources
            .clone()
            .into_iter()
            .filter(|(k, _v)| keys_index.contains(k))
            .collect();
    }

    pub fn checked_sources(&self) -> impl Iterator<Item = &SourceKey> {
        self.sources
            .iter()
            .filter(|(_key, entry)| entry.enabled)
            .map(|(key, _entry)| key)
    }

    pub fn sources(&self) -> impl Iterator<Item = (String, SourceKey, bool)> + '_ {
        self.sources
            .iter()
            .map(|(key, entry)| (entry.name.clone(), key.clone(), entry.enabled))
    }

    pub fn check_source(&mut self, key: &SourceKey, checked: bool) {
        if let Some(entry) = self.sources.get_mut(key) {
            entry.enabled = checked;
        }
    }
}
