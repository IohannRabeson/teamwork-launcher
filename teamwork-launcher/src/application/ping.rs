use {
    crate::application::message::PingServiceMessage,
    iced::{
        futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
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

const PAYLOAD: &[u8] = &[0u8; 56];

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Ping request timed out")]
    Timeout,
    #[error("Client is disabled")]
    ClientDisabled,
}

impl PingService {
    pub fn is_enabled(&self) -> bool {
        self.client.is_some()
    }

    pub async fn ping(&self, ip: &Ipv4Addr) -> Result<Duration, Error> {
        if let Some(client) = self.client.as_ref() {
            let mut pinger = client.pinger(IpAddr::from(*ip), PingIdentifier(111)).await;

            pinger.timeout(Duration::from_secs(1));

            match pinger.ping(PingSequence(0), PAYLOAD).await {
                Ok((IcmpPacket::V4(_reply), dur)) => Ok(dur),
                Ok((IcmpPacket::V6(_reply), dur)) => Ok(dur),
                Err(e) => Err(Error::Timeout),
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
            Err(error) => None,
        };

        Self { client }
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
