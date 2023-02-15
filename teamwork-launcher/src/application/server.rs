use {
    crate::application::{country::Country, ip_port::IpPort, promised_value::PromisedValue, servers_source::SourceKey},
    iced::widget::image,
    std::{str::FromStr, sync::Arc, time::Duration},
};

/// Store information about a server.
#[derive(Debug, Hash, Clone)]
pub struct Server {
    pub name: String,
    pub max_players_count: u8,
    pub current_players_count: u8,
    pub map: String,
    pub map_thumbnail: PromisedValue<image::Handle>,
    pub ip_port: IpPort,
    pub country: PromisedValue<Country>,
    pub ping: PromisedValue<Duration>,
    pub source_key: Option<SourceKey>,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            name: Default::default(),
            max_players_count: Default::default(),
            current_players_count: Default::default(),
            map: Default::default(),
            map_thumbnail: PromisedValue::None,
            ip_port: IpPort::default(),
            country: PromisedValue::None,
            ping: PromisedValue::None,
            source_key: None,
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
            map: server.map_name,
            map_thumbnail: PromisedValue::Loading,
            current_players_count: server.players,
            max_players_count: server.max_players,
            ip_port: IpPort::new(std::net::Ipv4Addr::from_str(&server.ip).expect("parse IP"), server.port),
            country: PromisedValue::Loading,
            ping: PromisedValue::Loading,
            source_key: None,
        }
    }
}
