use std::{net::{Ipv4Addr, IpAddr}, time::Duration};
use log::debug;
use surge_ping::{Client, Config, PingIdentifier, PingSequence, IcmpPacket};
use log::error;

#[derive(Clone)]
pub struct PingService {
    client: Option<Client>,
}

const PAYLOAD: &[u8] = &[0u8; 56];

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Ping request timed out")]
    Timeout,
    #[error("Client is disabled")]
    ClientDisabled,
}

impl PingService {
    pub async fn ping(&self, ip: &Ipv4Addr) -> Result<Duration, Error> {
        if let Some(client) = self.client.as_ref() {
            let mut pinger = client.pinger(IpAddr::from(*ip), PingIdentifier(111)).await;

            pinger.timeout(Duration::from_secs(1));
            
            match pinger.ping(PingSequence(0), &PAYLOAD).await {
                Ok((IcmpPacket::V4(_reply), dur)) => {
                    Ok(dur)
                }
                Ok((IcmpPacket::V6(_reply), dur)) => {
                    Ok(dur)
                }
                Err(e) => {
                    debug!("Ping service error: {}", e);
                    Err(Error::Timeout)
                }
            }
        } else {
            Err(Error::ClientDisabled)
        }
    }
}

impl Default for PingService {
    fn default() -> Self {
        let config = Config::default();
        let client = match Client::new(&config) {
            Ok(client) => Some(client),
            Err(error) => {
                error!("Failed to create ping service: {}", error);
                None
            },
        };

        Self { client }
    }
}
