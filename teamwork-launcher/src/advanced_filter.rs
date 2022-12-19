use serde::{Deserialize, Serialize};

use crate::models::Server;

fn default_as_true() -> bool { true }

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AdvancedServerFilter {
    #[serde(default)]
    pub minimum_players_count: u8,
    #[serde(default = "default_as_true")]
    pub with_valid_ping: bool,
}

impl AdvancedServerFilter {
    pub fn accept_server(&self, server: &Server) -> bool {
        server.current_players_count >= self.minimum_players_count && (!self.with_valid_ping || !server.ping.is_none())
    }
}
