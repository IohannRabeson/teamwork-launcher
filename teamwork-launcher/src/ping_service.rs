use std::{net::{Ipv4Addr, IpAddr}, time::Duration};
use log::debug;
use surge_ping::{Client, Config, PingIdentifier, PingSequence, IcmpPacket};

#[derive(Clone)]
pub struct PingService {
    client: Client,
}

const PAYLOAD: &[u8] = &[0u8; 56];

impl PingService {
    pub async fn ping(&self, ip: &Ipv4Addr) -> Result<Duration, ()> {
        let mut pinger = self.client.pinger(IpAddr::from(*ip), PingIdentifier(111)).await;

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
                Err(())
            }
        }
    }
}

impl Default for PingService {
    fn default() -> Self {
        let config = Config::default();

        Self {
            client: Client::new(&config).expect("create ping client"),
        }
    }
}
