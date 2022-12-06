use std::{fmt::Display, net::Ipv4Addr, str::FromStr};

use {
    iced::widget::image,
    serde::{Deserialize, Serialize},
};

use crate::{promised_value::PromisedValue, sources::SourceKey};

/// The unique key identifiying a server.
#[derive(Debug, Hash, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct IpPort {
    ip: Ipv4Addr,
    port: u16,
}

impl IpPort {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Self { ip, port }
    }
    pub fn steam_connection_string(&self) -> String {
        format!("connect {}:{}", self.ip, self.port)
    }
    pub fn ip(&self) -> &Ipv4Addr {
        &self.ip
    }
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Default for IpPort {
    fn default() -> Self {
        Self {
            ip: Ipv4Addr::UNSPECIFIED,
            port: Default::default(),
        }
    }
}

impl From<(Ipv4Addr, u16)> for IpPort {
    fn from(input: (Ipv4Addr, u16)) -> Self {
        Self::new(input.0, input.1)
    }
}

pub type Thumbnail = PromisedValue<image::Handle>;

/// Store information about a server.
///
/// Currently it's clonable but it could be better to make it "privately clonable" only.
#[derive(Debug, Hash, Clone)]
pub struct Server {
    pub name: String,
    pub max_players_count: u8,
    pub current_players_count: u8,
    pub map: String,
    pub map_thumbnail: PromisedValue<image::Handle>,
    pub ip_port: IpPort,
    pub source: Option<SourceKey>,
    pub country: PromisedValue<Country>,
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
            source: None,
            country: PromisedValue::None,
        }
    }
}

#[derive(Debug, Hash, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Country {
    code: String,
}

impl Country {
    pub fn new(code: &impl ToString) -> Self {
        Self {
            code: code.to_string()
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> String {
        match iso_country::Country::from_str(&self.code) {
            Ok(country) => country.name().to_string(),
            Err(_error) => self.code.clone(),
        }
    }
}

impl Display for Country {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
