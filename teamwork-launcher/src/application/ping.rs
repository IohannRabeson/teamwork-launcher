use {
    crate::application::message::PingServiceMessage,
    iced::{
        futures::channel::mpsc::{unbounded, UnboundedReceiver},
        subscription, Subscription,
    },
    std::{
        net::{IpAddr, Ipv4Addr},
        time::Duration,
    },
    surge_ping::{Client, Config, IcmpPacket, PingIdentifier, PingSequence},
};

#[derive(Clone)]
struct PingService {
    client: Option<Client>,
}

const PAYLOAD: &[u8; 56] = &[
    59, 58, 33, 146, 20, 170, 170, 13, 15, 49, 219, 36, 228, 142, 124, 241, 230, 113, 211, 158, 229, 9, 136, 36, 17, 35,
    106, 80, 211, 241, 71, 161, 100, 22, 146, 168, 159, 186, 221, 73, 30, 159, 225, 231, 106, 202, 249, 97, 154, 146, 139,
    248, 239, 231, 2, 65,
];

#[derive(thiserror::Error, Debug, Clone)]
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

            match pinger.ping(PingSequence(0), PAYLOAD).await {
                Ok((IcmpPacket::V4(_reply), dur)) => Ok(dur),
                Ok((IcmpPacket::V6(_reply), dur)) => Ok(dur),
                Err(_e) => Err(Error::Timeout),
            }
        } else {
            Err(Error::ClientDisabled)
        }
    }
}

impl Default for PingService {
    fn default() -> Self {
        let config = Config::default();

        Self {
            client: Client::new(&config).ok(),
        }
    }
}

enum State {
    Starting,
    Ready(UnboundedReceiver<Ipv4Addr>, PingService),
}

pub fn subscription() -> Subscription<PingServiceMessage> {
    subscription::unfold((), State::Starting, |state| async move {
        match state {
            State::Starting => {
                let (sender, receiver) = unbounded();

                (
                    Some(PingServiceMessage::Started(sender)),
                    State::Ready(receiver, PingService::default()),
                )
            }
            State::Ready(mut receiver, service) => {
                use iced::futures::StreamExt;

                let ip = receiver.select_next_some().await;

                match service.ping(&ip).await {
                    Ok(duration) => (
                        Some(PingServiceMessage::Answer(ip, duration)),
                        State::Ready(receiver, service),
                    ),
                    Err(error) => (Some(PingServiceMessage::Error(ip, error)), State::Ready(receiver, service)),
                }
            }
        }
    })
}
