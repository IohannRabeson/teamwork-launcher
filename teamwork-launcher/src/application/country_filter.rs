use {
    crate::application::{Country, PromisedValue},
    serde::{Deserialize, Serialize},
    std::collections::{btree_map::Entry, BTreeMap, BTreeSet},
};

#[derive(Serialize, Deserialize, Default)]
pub struct CountryFilter {
    #[serde(skip)]
    available_countries: BTreeSet<Country>,
    hidden_countries: BTreeSet<Country>,
    no_countries: bool,
}

impl CountryFilter {
    pub fn new() -> Self {
        Self {
            available_countries: BTreeSet::new(),
            hidden_countries: BTreeSet::new(),
            no_countries: true,
        }
    }

    pub fn accept(&self, country: &PromisedValue<Country>) -> bool {
        match country {
            PromisedValue::Ready(country) => {
                self.available_countries.contains(country) && !self.hidden_countries.contains(country)
            }
            PromisedValue::Loading => true,
            PromisedValue::None => self.no_countries,
        }
    }

    pub fn available_countries(&self) -> impl Iterator<Item = &Country> {
        self.available_countries.iter()
    }

    pub fn add_available(&mut self, country: Country) {
        self.available_countries.insert(country);
    }

    pub fn clear_available(&mut self) {
        self.available_countries.clear();
    }

    pub fn extend_available(&mut self, countries: &[Country]) {
        self.available_countries.extend(countries.iter().cloned());
    }

    pub fn set_checked(&mut self, country: &Country, checked: bool) {
        match checked {
            true => {
                self.hidden_countries.remove(country);
            }
            false => {
                self.hidden_countries.insert(country.clone());
            }
        }
    }

    pub fn is_checked(&self, country: &Country) -> bool {
        !self.hidden_countries.contains(country)
    }

    pub fn accept_no_country(&self) -> bool {
        self.no_countries
    }

    pub fn set_accept_no_country(&mut self, accept: bool) {
        self.no_countries = accept;
    }
}
