use {
    crate::application::{game_mode::GameModeId, Server},
    serde::{Deserialize, Serialize},
    std::collections::{btree_map::Entry::Vacant, BTreeMap},
};

#[derive(Serialize, Deserialize, Default)]
pub struct GameModeFilter {
    game_modes: BTreeMap<GameModeId, bool>,
    enabled: bool,
}

impl GameModeFilter {
    pub fn new() -> Self {
        Self {
            game_modes: BTreeMap::new(),
            enabled: false,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_mode_enabled(&mut self, id: &GameModeId, enabled: bool) {
        if let Some(enabled_value) = self.game_modes.get_mut(id) {
            *enabled_value = enabled;
        }
    }

    pub fn enable_only(&mut self, enable_id: &GameModeId) {
        for (id, enabled) in self.game_modes.iter_mut() {
            *enabled = enable_id == id;
        }
    }

    pub fn enable_all_excepted(&mut self, enable_id: &GameModeId) {
        for (id, enabled) in self.game_modes.iter_mut() {
            *enabled = enable_id != id;
        }
    }

    pub fn is_mode_enabled(&self, id: &GameModeId) -> bool {
        self.game_modes.get(id).copied().unwrap_or_default()
    }

    pub fn game_modes(&self) -> impl Iterator<Item = (&GameModeId, &bool)> {
        self.game_modes.iter()
    }

    pub fn reset(&mut self, game_modes: &[teamwork::GameMode]) {
        for mode in game_modes {
            if let Vacant(entry) = self.game_modes.entry(GameModeId::new(&mode.id)) {
                entry.insert(true);
            }
        }
    }

    pub fn accept(&self, server: &Server) -> bool {
        if !self.enabled {
            return true;
        }

        for id in &server.game_modes {
            if let Some(accepted) = self.game_modes.get(id) {
                if !accepted {
                    return false;
                }
            }
        }

        true
    }
}
