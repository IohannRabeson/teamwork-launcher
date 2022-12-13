use {
    crate::{models::Server, sources::SourceKey},
    serde::{Deserialize, Serialize},
    std::collections::BTreeMap,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SourceFilter {
    sources: BTreeMap<SourceKey, Entry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Entry {
    pub checked: bool,
    pub name: String,
}

impl Default for SourceFilter {
    fn default() -> Self {
        Self {
            sources: BTreeMap::new(),
        }
    }
}

impl SourceFilter {
    pub fn accept_server(&self, server: &Server) -> bool {
        match &server.source {
            Some(key) => self.sources.get(key).map(|entry| entry.checked).unwrap_or(false),
            None => false,
        }
    }

    pub fn set_available_sources(&mut self, all_source_keys: impl Iterator<Item = (String, SourceKey)>) {
        use std::collections::btree_map::Entry::{Vacant, Occupied};

        for (init_name, init_key) in all_source_keys {
            match self.sources.entry(init_key) {
                Vacant(entry) => {
                    entry.insert(Entry {
                        name: init_name,
                        checked: true,
                    });
                }
                Occupied(mut entry) => {
                    entry.get_mut().name = init_name;
                },
            }
        }
    }

    pub fn checked_sources(&self) -> impl Iterator<Item = &SourceKey> {
        self.sources
            .iter()
            .filter(|(_key, entry)| entry.checked)
            .map(|(key, _entry)| key)
    }

    pub fn sources(&self) -> impl Iterator<Item = (String, SourceKey, bool)> + '_ {
        self.sources
            .iter()
            .map(|(key, entry)| (entry.name.clone(), key.clone(), entry.checked))
    }

    pub fn check_source(&mut self, key: &SourceKey, checked: bool) {
        if let Some(entry) = self.sources.get_mut(key) {
            entry.checked = checked;
        }
    }
}
