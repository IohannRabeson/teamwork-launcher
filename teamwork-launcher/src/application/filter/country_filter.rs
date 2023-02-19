use {
    crate::application::{Country, PromisedValue},
    serde::{Deserialize, Serialize},
};
use crate::application::filter::filter_dictionary::FilterDictionary;

#[derive(Serialize, Deserialize)]
pub struct CountryFilter {
    pub dictionary: FilterDictionary<Country>,
    pub no_countries: bool,
    pub enabled: bool,
}

impl Default for CountryFilter {
    fn default() -> Self {
        Self {
            dictionary: FilterDictionary::new(),
            no_countries: true,
            enabled: false,
        }
    }
}

impl CountryFilter {
    pub fn accept(&self, country: &PromisedValue<Country>) -> bool {
        if !self.enabled {
            return true;
        }

        match country {
            PromisedValue::Ready(country) => self.dictionary.is_checked(country),
            PromisedValue::Loading => true,
            PromisedValue::None => self.no_countries,
        }
    }
}
