use {
    crate::application::{Country, PromisedValue},
    itertools::{chain, Itertools},
    serde::{Deserialize, Serialize},
    std::collections::{btree_map::Entry::Vacant, BTreeMap, BTreeSet},
};

#[derive(Serialize, Deserialize)]
pub struct CountryFilter {
    #[serde(skip)]
    available_countries: BTreeSet<Country>,
    countries: BTreeMap<Country, bool>,
    no_countries: bool,
    enabled: bool,
}

impl Default for CountryFilter {
    fn default() -> Self {
        Self {
            available_countries: BTreeSet::new(),
            countries: BTreeMap::new(),
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
            PromisedValue::Ready(country) => self.available_countries.contains(country) && self.is_checked(country),
            PromisedValue::Loading => true,
            PromisedValue::None => self.no_countries,
        }
    }

    pub fn available_countries(&self) -> impl Iterator<Item = &Country> {
        chain!(self.available_countries.iter(), self.countries.keys())
            .unique()
            .sorted_by(|l, r| l.name().cmp(&r.name()))
    }

    pub fn add_available(&mut self, country: Country) {
        self.available_countries.insert(country.clone());
        if let Vacant(vacant) = self.countries.entry(country) {
            vacant.insert(true);
        }
    }

    pub fn clear_available(&mut self) {
        self.available_countries.clear();
    }

    pub fn extend_available(&mut self, countries: &[Country]) {
        for country in countries {
            self.add_available(country.clone());
        }
    }

    pub fn set_checked(&mut self, country: &Country, checked: bool) {
        self.countries.insert(country.clone(), checked);
    }

    pub fn is_checked(&self, country: &Country) -> bool {
        self.countries.get(country).copied().unwrap_or_default()
    }

    pub fn check_only(&mut self, country_to_check: &Country) {
        for (country, enabled) in self.countries.iter_mut() {
            *enabled = country == country_to_check;
        }
    }

    pub fn check_all_excepted(&mut self, excluded_country: &Country) {
        for (country, enabled) in self.countries.iter_mut() {
            *enabled = country != excluded_country;
        }
    }

    pub fn accept_no_country(&self) -> bool {
        self.no_countries
    }

    pub fn set_accept_no_country(&mut self, accept: bool) {
        self.no_countries = accept;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
