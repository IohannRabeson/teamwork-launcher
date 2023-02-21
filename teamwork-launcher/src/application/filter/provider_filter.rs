use {
    crate::application::{filter::filter_dictionary::FilterDictionary, Server},
    serde::{Deserialize, Serialize},
};

#[derive(Serialize, Deserialize)]
pub struct ProviderFilter {
    pub enabled: bool,
    pub dictionary: FilterDictionary<String>,
}

impl Default for ProviderFilter {
    fn default() -> Self {
        Self {
            enabled: false,
            dictionary: FilterDictionary::new(),
        }
    }
}

impl ProviderFilter {
    pub fn accept(&self, server: &Server) -> bool {
        if !self.enabled {
            return true;
        }

        self.dictionary.is_checked(&server.provider)
    }
}
