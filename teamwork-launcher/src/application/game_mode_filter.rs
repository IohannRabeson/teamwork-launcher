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
                if *accepted {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use crate::application::game_mode::{GameMode, GameModeId};
    use crate::application::game_mode_filter::GameModeFilter;
    use crate::application::{IpPort, Server};

    #[test]
    fn test_accept() {
        let mut filter = GameModeFilter::new();
        let gma = teamwork::GameMode {
            id: "a".to_string(),
            title: "A".to_string(),
            description: "A desc".to_string(),
            color: None,
        };
        let gmb = teamwork::GameMode {
            id: "b".to_string(),
            title: "A".to_string(),
            description: "A desc".to_string(),
            color: None,
        };

        filter.reset(&vec![gma.clone(), gmb.clone()]);

        let server_gma = Server {
            name: "hey".to_string(),
            max_players_count: 0,
            current_players_count: 0,
            map: Default::default(),
            map_thumbnail: Default::default(),
            ip_port: IpPort::new(Ipv4Addr::new(127, 0, 0, 1), 12345),
            country: Default::default(),
            ping: Default::default(),
            source_key: None,
            game_modes: vec![GameModeId::new(gma.id.clone())],
        };

        let server_gmb = Server {
            name: "hoy".to_string(),
            max_players_count: 0,
            current_players_count: 0,
            map: Default::default(),
            map_thumbnail: Default::default(),
            ip_port: IpPort::new(Ipv4Addr::new(127, 0, 0, 1), 567),
            country: Default::default(),
            ping: Default::default(),
            source_key: None,
            game_modes: vec![GameModeId::new(gmb.id.clone())],
        };

        filter.set_enabled(true);
        filter.set_mode_enabled(&GameModeId::new(gma.id.clone()), true);
        filter.set_mode_enabled(&GameModeId::new(gmb.id.clone()), false);

        assert_eq!(filter.accept(&server_gma), true);
        assert_eq!(filter.accept(&server_gmb), false);
    }
}