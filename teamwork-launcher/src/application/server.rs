use {
    crate::application::{
        country::Country, game_mode::GameModeId, ip_port::IpPort, map::MapName, promised_value::PromisedValue,
        servers_source::SourceKey,
    },
    iced::widget::image,
    std::{str::FromStr, time::Duration},
};

/// Store information about a server.
#[derive(Debug, Hash, Clone)]
pub struct Server {
    pub name: String,
    pub max_players_count: u8,
    pub current_players_count: u8,
    pub map: MapName,
    pub next_map: Option<MapName>,
    pub map_thumbnail: PromisedValue<image::Handle>,
    pub ip_port: IpPort,
    pub country: PromisedValue<Country>,
    pub ping: PromisedValue<Duration>,
    pub source_key: Option<SourceKey>,
    pub game_modes: Vec<GameModeId>,
    pub provider: String,
    pub vac_secured: bool,
    pub has_rtd: bool,
    pub has_no_respawn_time: bool,
    pub has_all_talk: bool,
    pub has_random_crits: bool,
    pub need_password: bool,
}

impl Server {
    pub fn free_slots(&self) -> u8 {
        if self.max_players_count < self.current_players_count {
            return 0;
        }
        self.max_players_count - self.current_players_count
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            name: Default::default(),
            max_players_count: Default::default(),
            current_players_count: Default::default(),
            map: Default::default(),
            next_map: None,
            map_thumbnail: PromisedValue::None,
            ip_port: IpPort::default(),
            country: PromisedValue::None,
            ping: PromisedValue::None,
            source_key: None,
            provider: String::new(),
            game_modes: Vec::new(),
            vac_secured: false,
            has_rtd: false,
            has_no_respawn_time: false,
            has_all_talk: false,
            has_random_crits: false,
            need_password: false,
        }
    }
}

/// The rational is I do not want the entire application depends on the Teamwork.tf API.
/// So even if it's a bit tedious, I prefer to have a struct Server dedicated to the application
/// to avoid using teamwork::Server.
/// Also that opens the system to extension by adding more source of data.
///
impl From<teamwork::Server> for Server {
    fn from(server: teamwork::Server) -> Self {
        Server {
            name: server.name,
            map: MapName::new(server.map_name),
            next_map: server.map_name_next.map(MapName::new),
            map_thumbnail: PromisedValue::Loading,
            current_players_count: server.players,
            max_players_count: server.max_players,
            ip_port: IpPort::new(std::net::Ipv4Addr::from_str(&server.ip).expect("parse IP"), server.port),
            country: PromisedValue::Loading,
            ping: PromisedValue::Loading,
            source_key: None,
            provider: server.provider.unwrap_or_else(|| String::from("Unspecified")),
            game_modes: server.game_modes.iter().map(GameModeId::new).collect(),
            vac_secured: server.valve_secure,
            has_all_talk: server.has_alltalk.unwrap_or_default(),
            has_rtd: server.has_rtd.unwrap_or_default(),
            has_no_respawn_time: server.has_norespawntime.unwrap_or_default(),
            has_random_crits: server.has_randomcrits.unwrap_or_default(),
            need_password: server.has_password.unwrap_or_default(),
        }
    }
}
