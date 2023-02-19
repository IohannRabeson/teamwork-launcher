use std::collections::BTreeMap;
use std::collections::btree_map::Entry::{Occupied, Vacant};
use crate::application::{Country, Property};
use crate::application::game_mode::GameModeId;
use crate::application::map::MapName;

#[derive(Default)]
pub struct ServersCounts {
    pub bookmarks: usize,
    pub timeouts: usize,
    pub countries: BTreeMap<Country, usize>,
    pub game_modes: BTreeMap<GameModeId, usize>,
    pub properties: BTreeMap<Property, usize>,
    pub maps: BTreeMap<MapName, usize>,
}

impl ServersCounts {
    pub fn reset(&mut self) {
        *self = ServersCounts::default();
    }

    pub fn add_country(&mut self, country: Country) {
        match self.countries.entry(country) {
            Vacant(vacant) => {
                vacant.insert(1);
            }
            Occupied(mut occupied) => {
                *occupied.get_mut() += 1;
            }
        };
    }
}
