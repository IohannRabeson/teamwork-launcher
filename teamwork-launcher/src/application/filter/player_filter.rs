use {
    crate::application::Server,
    serde::{Deserialize, Serialize},
};

fn default_true() -> bool { true }

#[derive(Serialize, Deserialize, Default)]
pub struct PlayerFilter {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub minimum_players: u8,
    #[serde(skip)]
    pub maximum_players: u8,
    pub minimum_free_slots: u8,
    #[serde(skip)]
    pub maximum_free_slots: u8,
}

impl PlayerFilter {
    pub fn accept(&self, server: &Server) -> bool {
        if !self.enabled {
            return true
        }

        server.current_players_count >= self.minimum_players && server.free_slots() >= self.minimum_free_slots
    }
}
