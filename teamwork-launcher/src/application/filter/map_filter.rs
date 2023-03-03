use {
    crate::application::{filter::filter_dictionary::FilterDictionary, map::MapName},
    serde::{Deserialize, Serialize},
};

#[derive(Serialize, Deserialize)]
pub struct MapFilter {
    pub dictionary: FilterDictionary<MapName>,
    pub enabled: bool,
    pub text: String,
}

impl Default for MapFilter {
    fn default() -> Self {
        Self {
            dictionary: FilterDictionary::new(),
            enabled: false,
            text: String::new(),
        }
    }
}
