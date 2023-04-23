use {
    serde::{Deserialize, Serialize},
    std::{
        fmt::{Display, Formatter},
        net::Ipv4Addr,
    },
};

/// The unique key identifying a server.
#[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

impl Display for IpPort {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip(), self.port())
    }
}
