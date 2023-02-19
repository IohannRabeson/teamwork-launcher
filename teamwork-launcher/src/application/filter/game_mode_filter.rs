use {
    crate::application::{game_mode::GameModeId, Server},
    serde::{Deserialize, Serialize},
};
use crate::application::filter::filter_dictionary::FilterDictionary;

#[derive(Serialize, Deserialize)]
pub struct GameModeFilter {
    pub dictionary: FilterDictionary<GameModeId>,
    pub enabled: bool,
}

impl Default for GameModeFilter {
    fn default() -> Self {
        Self {
            dictionary: FilterDictionary::new(),
            enabled: false,
        }
    }
}

impl GameModeFilter {
    pub fn accept(&self, server: &Server) -> bool {
        if !self.enabled {
            return true;
        }

        for id in &server.game_modes {
            if self.dictionary.is_checked(id) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::application::{game_mode::GameModeId, IpPort, Server},
        std::net::Ipv4Addr,
    };
    use crate::application::filter::game_mode_filter::GameModeFilter;

    #[test]
    fn test_accept() {
        let mut filter = GameModeFilter::default();
        let gma = teamwork::GameMode {
            id: "a".to_string(),
            title: "A".to_string(),
            description: "A desc".to_string(),
            color: None,
        };
        let gmb = teamwork::GameMode {
            id: "b".to_string(),
            title: "B".to_string(),
            description: "B desc".to_string(),
            color: None,
        };

        filter.dictionary.add(GameModeId::new(gma.id.clone()));
        filter.dictionary.add(GameModeId::new(gmb.id.clone()));

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
            ..Default::default()
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
            ..Default::default()
        };

        filter.enabled = true;
        filter.dictionary.set_checked(&GameModeId::new(gma.id.clone()), true);
        filter.dictionary.set_checked(&GameModeId::new(gmb.id.clone()), false);

        assert_eq!(filter.accept(&server_gma), true);
        assert_eq!(filter.accept(&server_gmb), false);
    }
}
