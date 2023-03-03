use {
    crate::application::{filter::default_true, PromisedValue, Server},
    serde::{Deserialize, Serialize},
};

#[derive(Serialize, Deserialize, Default)]
pub struct PingFilter {
    pub max_ping: u32,
    pub accept_ping_timeout: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl PingFilter {
    pub fn accept(&self, server: &Server) -> bool {
        if !self.enabled {
            return true;
        }

        match server.ping {
            PromisedValue::Ready(ping) => ping.as_millis() <= self.max_ping as u128,
            PromisedValue::Loading => true,
            PromisedValue::None => self.accept_ping_timeout,
        }
    }
}
