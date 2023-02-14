use {
    crate::application::{
        country::Country,
        message::{CountryServiceMessage, Message},
    },
    async_stream::stream,
    iced::{
        futures::{
            channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
            StreamExt,
        },
        subscription, Subscription,
    },
    serde::{Deserialize, Serialize},
    std::{
        collections::{btree_map::Entry, BTreeMap},
        net::Ipv4Addr,
    },
};

#[derive(thiserror::Error, Debug, Clone)]
#[error("Failed to geo-localize IP '{ip}': {message}")]
pub struct Error {
    pub ip: String,
    pub message: String,
}

impl Error {
    pub fn new(ip: String, message: &impl ToString) -> Self {
        Self {
            ip,
            message: message.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CountryIsResponse {
    ip: String,
    country: String,
}

async fn locate(ip: &Ipv4Addr) -> Result<Country, Error> {
    const COUNTRY_IS_API_URL: &str = "https://api.country.is";

    let url = format!("{}/{}", COUNTRY_IS_API_URL, ip);
    let ip = ip.to_string();
    let raw_text = reqwest::get(url)
        .await
        .map_err(|error| Error::new(ip.clone(), &error))?
        .text()
        .await
        .map_err(|error| Error::new(ip.clone(), &error))?;

    let response: CountryIsResponse = serde_json::from_str(&raw_text).map_err(|error| Error::new(ip.clone(), &error))?;

    Ok(Country::new(&response.country))
}

enum State {
    Starting,
    Ready(UnboundedReceiver<Ipv4Addr>, BTreeMap<Ipv4Addr, Country>),
}

pub fn subscription() -> Subscription<CountryServiceMessage> {
    subscription::unfold((), State::Starting, |state| async move {
        match state {
            State::Starting => {
                let (sender, receiver) = unbounded();

                (
                    Some(CountryServiceMessage::Started(sender)),
                    State::Ready(receiver, BTreeMap::new()),
                )
            }
            State::Ready(mut receiver, mut cache) => {
                use iced::futures::StreamExt;

                let ip = receiver.select_next_some().await;

                match cache.entry(ip) {
                    Entry::Vacant(vacant) => match locate(&ip).await {
                        Ok(country) => {
                            vacant.insert(country.clone());
                            (
                                Some(CountryServiceMessage::CountryFound(ip, country)),
                                State::Ready(receiver, cache),
                            )
                        }
                        Err(error) => (Some(CountryServiceMessage::Error(error)), State::Ready(receiver, cache)),
                    },
                    Entry::Occupied(occupied) => (
                        Some(CountryServiceMessage::CountryFound(ip, occupied.get().clone())),
                        State::Ready(receiver, cache),
                    ),
                }
            }
        }
    })
}
