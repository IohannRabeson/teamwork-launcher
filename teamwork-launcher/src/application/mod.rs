pub mod country;
mod country_filter;
pub mod fetch_servers;
pub mod filter_servers;
mod geolocation;
pub mod ip_port;
mod ping;
pub mod promised_value;
pub mod server;
mod text_filter;
mod thumbnail;
mod views;

use {
    crate::{application::views::Views, ui},
    futures::{FutureExt, SinkExt, TryFutureExt},
    iced::{
        futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
        subscription,
        widget::image,
        Command, Element, Renderer, Subscription,
    },
    itertools::Itertools,
    std::{
        collections::{btree_map::Entry, BTreeMap, BTreeSet},
        net::Ipv4Addr,
        sync::Arc,
        time::Duration,
    },
    teamwork::UrlWithKey,
};

pub use {
    country::Country,
    fetch_servers::{fetch_servers, FetchServersEvent},
    filter_servers::Filter,
    ip_port::IpPort,
    promised_value::PromisedValue,
    server::Server,
};

#[derive(Debug, Clone)]
pub enum FetchServersMessage {
    FetchServersStart,
    FetchServersFinish,
    FetchServersError(Arc<teamwork::Error>),
    NewServers(Vec<Server>),
}

#[derive(Debug, Clone)]
pub enum CountryServiceMessage {
    Started(UnboundedSender<Ipv4Addr>),
    CountryFound(Ipv4Addr, Country),
    Error(geolocation::Error),
}

#[derive(Debug, Clone)]
pub enum PingServiceMessage {
    Started(UnboundedSender<Ipv4Addr>),
    Answer(Ipv4Addr, Duration),
    Error(Ipv4Addr, ping::Error),
}

#[derive(Debug, Clone)]
pub enum ThumbnailMessage {
    Started(UnboundedSender<String>),
    Thumbnail(String, image::Handle),
    Error(String, Arc<teamwork::Error>),
}

#[derive(Debug, Clone)]
pub enum FilterMessage {
    CountryChecked(Country, bool),
    NoCountryChecked(bool),
    TextChanged(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    Servers(FetchServersMessage),
    Country(CountryServiceMessage),
    Ping(PingServiceMessage),
    Thumbnail(ThumbnailMessage),
    Filter(FilterMessage),
    RefreshServers,
}

impl From<FetchServersEvent> for Message {
    fn from(value: FetchServersEvent) -> Self {
        match value {
            FetchServersEvent::Start => Message::Servers(FetchServersMessage::FetchServersStart),
            FetchServersEvent::Finish => Message::Servers(FetchServersMessage::FetchServersFinish),
            FetchServersEvent::Servers(servers) => Message::Servers(FetchServersMessage::NewServers(servers)),
            FetchServersEvent::Error(error) => Message::Servers(FetchServersMessage::FetchServersError(error)),
        }
    }
}

impl From<CountryServiceMessage> for Message {
    fn from(value: CountryServiceMessage) -> Self {
        Message::Country(value)
    }
}

impl From<PingServiceMessage> for Message {
    fn from(message: PingServiceMessage) -> Self {
        Message::Ping(message)
    }
}

impl From<ThumbnailMessage> for Message {
    fn from(value: ThumbnailMessage) -> Self {
        Message::Thumbnail(value)
    }
}

pub enum Screens {
    Main(MainView),
}

pub struct MainView {
    pub servers: Vec<Server>,
    pub filter: Filter,
    pub fetch_servers_subscription_id: u64,
}

struct Settings {
    teamwork_api_key: String,
    server_urls: Vec<String>,
}

pub struct TeamworkLauncher {
    views: Views<Screens>,
    settings: Settings,
    country_sender: Option<UnboundedSender<Ipv4Addr>>,
    ping_sender: Option<UnboundedSender<Ipv4Addr>>,
    thumbnail_sender: Option<UnboundedSender<String>>,
}

impl TeamworkLauncher {
    fn new_servers(&mut self, new_servers: Vec<Server>) {
        if let Some(Screens::Main(view)) = self.views.current_mut() {
            for ip in new_servers.iter().map(|server| server.ip_port.ip()).unique().cloned() {
                if let Some(country_sender) = &mut self.country_sender {
                    country_sender
                        .send(ip.clone())
                        .unwrap_or_else(|e| eprintln!("country_sender {}", e))
                        .now_or_never();
                }

                if let Some(ping_sender) = &mut self.ping_sender {
                    ping_sender
                        .send(ip.clone())
                        .unwrap_or_else(|e| eprintln!("ping_sender {}", e))
                        .now_or_never();
                }
            }

            for map_name in new_servers.iter().map(|server| server.map.clone()).unique() {
                if let Some(thumbnail_sender) = &mut self.thumbnail_sender {
                    thumbnail_sender
                        .send(map_name.clone())
                        .unwrap_or_else(|e| eprintln!("thumbnail_sender {}", e))
                        .now_or_never();
                }
            }

            let countries: Vec<Country> = new_servers
                .iter()
                .filter_map(|server| server.country.get())
                .unique()
                .cloned()
                .collect();
            view.filter.country.extend_available(&countries);

            view.servers.extend(new_servers.into_iter());
            view.servers.sort_by(|l, r| l.ip_port.cmp(&r.ip_port));
        }
    }

    fn refresh_servers(&mut self) {
        if let Some(Screens::Main(view)) = self.views.current_mut() {
            view.servers.clear();
            view.filter.country.clear_available();
            view.fetch_servers_subscription_id = view.fetch_servers_subscription_id.wrapping_add(1);
        }
    }

    fn country_found(&mut self, ip: Ipv4Addr, country: Country) {
        if let Some(Screens::Main(view)) = self.views.current_mut() {
            view.filter.country.add_available(country.clone());
            for server in view.servers.iter_mut().filter(|server| server.ip_port.ip() == &ip) {
                server.country = PromisedValue::Ready(country.clone());
            }
        }
    }

    fn ping_found(&mut self, ip: Ipv4Addr, duration: Option<Duration>) {
        if let Some(Screens::Main(view)) = self.views.current_mut() {
            for server in view.servers.iter_mut().filter(|server| server.ip_port.ip() == &ip) {
                server.ping = duration.into();
            }
        }
    }

    fn thumbnail_ready(&mut self, map_name: String, thumbnail: Option<image::Handle>) {
        if let Some(Screens::Main(view)) = self.views.current_mut() {
            for server in view.servers.iter_mut().filter(|server| server.map == map_name) {
                server.map_thumbnail = thumbnail.clone().into();
            }
        }
    }
}

impl iced::Application for TeamworkLauncher {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                views: Views::new(Screens::Main(MainView {
                    servers: Vec::new(),
                    filter: Filter::new(),
                    fetch_servers_subscription_id: 0u64,
                })),
                settings: Settings {
                    teamwork_api_key: std::env::var("TEST_TEAMWORK_API_KEY").unwrap(),
                    server_urls: vec![
                        String::from("https://teamwork.tf/api/v1/quickplay/payload/servers"),
                        String::from("https://teamwork.tf/api/v1/quickplay/koth/servers"),
                        String::from("https://teamwork.tf/api/v1/quickplay/ctf/servers"),
                    ],
                },
                country_sender: None,
                ping_sender: None,
                thumbnail_sender: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Teamwork launcher")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Servers(FetchServersMessage::FetchServersStart) => {
                println!("Start");
            }
            Message::Servers(FetchServersMessage::FetchServersFinish) => {
                println!("Finish");
            }
            Message::Servers(FetchServersMessage::FetchServersError(error)) => {
                eprintln!("Error: {}", error);
            }
            Message::Servers(FetchServersMessage::NewServers(new_servers)) => self.new_servers(new_servers),
            Message::RefreshServers => self.refresh_servers(),
            Message::Country(CountryServiceMessage::Started(country_sender)) => {
                self.country_sender = Some(country_sender);
                eprintln!("country service started");
            }
            Message::Country(CountryServiceMessage::CountryFound(ip, country)) => {
                self.country_found(ip, country);
            }
            Message::Country(CountryServiceMessage::Error(error)) => {
                eprintln!("Error: {}", error);
            }
            Message::Ping(PingServiceMessage::Started(sender)) => {
                self.ping_sender = Some(sender);
                eprintln!("ping service started");
            }
            Message::Ping(PingServiceMessage::Answer(ip, duration)) => {
                self.ping_found(ip, Some(duration));
            }
            Message::Ping(PingServiceMessage::Error(ip, error)) => {
                eprintln!("Error: {}", error);
                self.ping_found(ip, None);
            }
            Message::Thumbnail(ThumbnailMessage::Started(sender)) => {
                self.thumbnail_sender = Some(sender);
                eprintln!("thumbnail service started");
            }
            Message::Thumbnail(ThumbnailMessage::Thumbnail(map_name, thumbnail)) => {
                self.thumbnail_ready(map_name, Some(thumbnail));
            }
            Message::Thumbnail(ThumbnailMessage::Error(map_name, error)) => {
                self.thumbnail_ready(map_name, None);
                eprintln!("Error: {}", error);
            }
            Message::Filter(FilterMessage::CountryChecked(country, checked)) => {
                if let Some(Screens::Main(view)) = self.views.current_mut() {
                    view.filter.country.set_checked(&country, checked);
                }
            }
            Message::Filter(FilterMessage::NoCountryChecked(checked)) => {
                if let Some(Screens::Main(view)) = self.views.current_mut() {
                    view.filter.country.set_accept_no_country(checked);
                }
            }
            Message::Filter(FilterMessage::TextChanged(text)) => {
                if let Some(Screens::Main(view)) = self.views.current_mut() {
                    view.filter.text.set_text(&text);
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message, Renderer<Self::Theme>> {
        match self.views.current().expect("valid view") {
            Screens::Main(view) => ui::main_view(view),
        }
        .into()
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        use iced::futures::StreamExt;

        match self.views.current().expect("valid view") {
            Screens::Main(view) => {
                let subscription_id = view.fetch_servers_subscription_id;
                let urls = self
                    .settings
                    .server_urls
                    .iter()
                    .map(|server_url| UrlWithKey::new(server_url, &self.settings.teamwork_api_key))
                    .collect();
                let server_stream = fetch_servers(urls).map(|event| Message::from(event));

                Subscription::batch([
                    subscription::run(subscription_id, server_stream),
                    geolocation::subscription().map(Message::from),
                    ping::subscription().map(Message::from),
                    thumbnail::subscription(&self.settings.teamwork_api_key).map(Message::from),
                ])
            }
        }
    }
}
