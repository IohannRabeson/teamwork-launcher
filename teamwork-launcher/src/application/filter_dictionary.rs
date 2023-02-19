use std::collections::btree_map::Entry::Vacant;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::application::map::MapName;

#[derive(Serialize, Deserialize)]
pub struct FilterDictionary<K>
where K: Ord
{
    entries: BTreeMap<K, bool>,
}

impl<K: Ord> FilterDictionary<K> {
    pub fn new() -> Self {
        Self { entries: BTreeMap::new() }
    }

    pub fn add(&mut self, key: K) {
        if let Vacant(entry) = self.entries.entry(key) {
            entry.insert(true);
        }
    }

    pub fn extend(&mut self, keys: impl Iterator<Item = K>) {
        for key in keys {
            self.add(key)
        }
    }

    pub fn check_only(&mut self, key: &K) {
        for (k, v) in self.entries.iter_mut() {
            *v = key == k;
        }
    }

    pub fn uncheck_only(&mut self, key: &K) {
        for (k, v) in self.entries.iter_mut() {
            *v = key != k;
        }
    }

    pub fn set_checked(&mut self, key: &K, check: bool) {
        if let Some(checked) = self.entries.get_mut(key) {
            *checked = check;
        }
    }

    pub fn is_checked(&self, key: &K) -> bool {
        self.entries.get(key).copied().unwrap_or_default()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, bool)> {
        self.entries.iter().map(|(key, enabled)|(key, *enabled))
    }
}
