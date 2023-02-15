use itertools::{chain, Itertools};
use {
    crate::application::{Country, PromisedValue},
    serde::{Deserialize, Serialize},
    std::collections::{
        btree_map::{Entry, Entry::Vacant},
        BTreeMap, BTreeSet,
    },
};

#[derive(Serialize, Deserialize, Default)]
pub struct CountryFilter {
    #[serde(skip)]
    available_countries: BTreeSet<Country>,
    countries: BTreeMap<Country, bool>,
    no_countries: bool,
}

impl CountryFilter {
    pub fn new() -> Self {
        Self {
            available_countries: BTreeSet::new(),
            countries: BTreeMap::new(),
            no_countries: true,
        }
    }

    pub fn accept(&self, country: &PromisedValue<Country>) -> bool {
        match country {
            PromisedValue::Ready(country) => self.available_countries.contains(country) && self.is_checked(country),
            PromisedValue::Loading => true,
            PromisedValue::None => self.no_countries,
        }
    }

    pub fn available_countries(&self) -> impl Iterator<Item = &Country> {
        chain!(self.available_countries.iter(), self.countries.keys().into_iter()).unique().sorted_by(|l, r|{
            l.name().cmp(&r.name())
        })
    }

    pub fn add_available(&mut self, country: Country) {
        self.available_countries.insert(country.clone());
        if let Vacant(mut vacant) = self.countries.entry(country) {
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
        self.countries.get(&country).map(|v| *v).unwrap_or_default()
    }

    pub fn accept_no_country(&self) -> bool {
        self.no_countries
    }

    pub fn set_accept_no_country(&mut self, accept: bool) {
        self.no_countries = accept;
    }
}
