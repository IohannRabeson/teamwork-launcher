use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::models::{Country, Server};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AdvancedServerFilter {
    pub minimum_players_count: u8,
    pub countries: BTreeMap<Country, bool>,
}

impl AdvancedServerFilter {
    pub fn accept_server(&self, server: &Server) -> bool {
        server.current_players_count >= self.minimum_players_count
    }
}
