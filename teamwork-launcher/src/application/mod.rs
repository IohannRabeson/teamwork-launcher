pub mod country;
mod country_filter;
pub mod fetch_servers;
pub mod filter_servers;
mod geolocation;
pub mod ip_port;
mod launcher;
mod message;
mod ping;
mod process_detection;
pub mod promised_value;
pub mod server;
mod text_filter;
mod thumbnail;
mod views;
mod bookmarks;
mod user_settings;

use std::cmp::Ordering;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use {
    crate::{application::views::Views, ui},
    iced::{
        Command,
        Element,
        futures::{
            channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
            FutureExt, SinkExt, TryFutureExt,
        },
        Renderer, subscription, Subscription, widget::image,
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

use crate::application::launcher::{ExecutableLauncher, LaunchError};
pub use {
    bookmarks::Bookmarks,
    country::Country,
    fetch_servers::{fetch_servers, FetchServersEvent},
    filter_servers::Filter,
    ip_port::IpPort,
    message::{
        CountryServiceMessage, FetchServersMessage, FilterMessage, Message, PingServiceMessage, SettingsMessage,
        ThumbnailMessage,
    },
    promised_value::PromisedValue,
    server::Server,
};
pub use crate::application::user_settings::UserSettings;

#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    #[error("JSON error: {0}")]
    Json(#[from] Arc<serde_json::Error>),
    #[error("IO error: {0}")]
    Io(#[from] Arc<std::io::Error>),
}

pub enum Screens {
    Main(MainView),
    Settings,
}

pub struct MainView {
    pub servers: Vec<Server>,
    pub filter: Filter,
}

pub struct TeamworkLauncher {
    views: Views<Screens>,
    user_settings: UserSettings,
    server_urls: Vec<String>,
    launcher: ExecutableLauncher,
    bookmarks: Bookmarks,
    country_sender: Option<UnboundedSender<Ipv4Addr>>,
    ping_sender: Option<UnboundedSender<Ipv4Addr>>,
    thumbnail_sender: Option<UnboundedSender<String>>,
    fetch_servers_subscription_id: u64,
}

impl TeamworkLauncher {
    fn process_filter_message(&mut self, message: FilterMessage) {
        match message {
            FilterMessage::CountryChecked(country, checked) => {
                if let Some(Screens::Main(view)) = self.views.current_mut() {
                    view.filter.country.set_checked(&country, checked);
                }
            }
            FilterMessage::NoCountryChecked(checked) => {
                if let Some(Screens::Main(view)) = self.views.current_mut() {
                    view.filter.country.set_accept_no_country(checked);
                }
            }
            FilterMessage::TextChanged(text) => {
                if let Some(Screens::Main(view)) = self.views.current_mut() {
                    view.filter.text.set_text(&text);
                }
            }
            FilterMessage::BookmarkedOnlyChecked(checked) => {
                if let Some(Screens::Main(view)) = self.views.current_mut() {
                    view.filter.bookmarked_only = checked;
                }
            }
        }
    }
}

impl TeamworkLauncher {
    fn new_servers(&mut self, new_servers: Vec<Server>) {
        if let Some(Screens::Main(view)) = self.views.current_mut() {
            for ip in new_servers.iter().map(|server| server.ip_port.ip()).unique().cloned() {
                if let Some(country_sender) = &mut self.country_sender {
                    country_sender
                        .send(ip.clone())
                        .unwrap_or_else(|e| eprintln!("country sender {}", e))
                        .now_or_never();
                }

                if let Some(ping_sender) = &mut self.ping_sender {
                    ping_sender
                        .send(ip.clone())
                        .unwrap_or_else(|e| eprintln!("ping sender {}", e))
                        .now_or_never();
                }
            }

            for map_name in new_servers.iter().map(|server| server.map.clone()).unique() {
                if let Some(thumbnail_sender) = &mut self.thumbnail_sender {
                    thumbnail_sender
                        .send(map_name.clone())
                        .unwrap_or_else(|e| eprintln!("thumbnail sender {}", e))
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
            view.servers.sort_by(Self::sort_servers);
        }
    }

    fn sort_servers(l: &Server, r: &Server) -> Ordering {
        l.ip_port.cmp(&r.ip_port)
    }

    fn refresh_servers(&mut self) {
        if let Some(Screens::Main(view)) = self.views.current_mut() {
            view.servers.clear();
            view.filter.country.clear_available();
        }
        self.fetch_servers_subscription_id += 1;
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

    fn process_settings_message(&mut self, message: SettingsMessage) {
        match message {
            SettingsMessage::TeamworkApiKeyChanged(key) => {
                self.user_settings.teamwork_api_key = key;
            }
            SettingsMessage::SteamExecutableChanged(executable_path) => {
                self.user_settings.steam_executable_path = executable_path;
            }
        }
    }
}

const APPLICATION_NAME: &str = "teamwork-launcher2";

pub fn get_configuration_directory() -> PathBuf {
    platform_dirs::AppDirs::new(APPLICATION_NAME.into(), false)
        .map(|dirs| dirs.config_dir)
        .expect("config directory path")
}

impl Drop for TeamworkLauncher {
    fn drop(&mut self) {
        let configuration_directory = get_configuration_directory();

        if !configuration_directory.is_dir() {
            std::fs::create_dir_all(&configuration_directory).unwrap_or_else(|e|eprintln!("Failed to create configuration directory '{}': {}", configuration_directory.display(), e));
        }

        self.bookmarks.write_file(&configuration_directory.join("bookmarks.json")).unwrap_or_else(|e|eprintln!("Failed to write bookmarks: {}", e));
        self.user_settings.write_file(&configuration_directory.join("settings.json")).unwrap_or_else(|e|eprintln!("Failed to write settings: {}", e));
    }
}

impl iced::Application for TeamworkLauncher {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let configuration_directory = get_configuration_directory();
        let bookmarks = Bookmarks::read_file(&configuration_directory.join("bookmarks.json")).unwrap_or_default();
        let user_settings = UserSettings::read_file(&configuration_directory.join("settings.json")).unwrap_or_default();

        (
            Self {
                views: Views::new(Screens::Main(MainView {
                    servers: Vec::new(),
                    filter: Filter::new(),
                })),
                user_settings,
                server_urls: vec![
                    String::from("https://teamwork.tf/api/v1/quickplay/payload/servers"),
                    String::from("https://teamwork.tf/api/v1/quickplay/koth/servers"),
                    String::from("https://teamwork.tf/api/v1/quickplay/ctf/servers"),
                ],
                launcher: ExecutableLauncher::new(true),
                bookmarks,
                country_sender: None,
                ping_sender: None,
                thumbnail_sender: None,
                fetch_servers_subscription_id: 0,
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
            Message::Filter(message) => {
                self.process_filter_message(message);
            }
            Message::Back => {
                self.views.pop();
            }
            Message::ShowSettings => {
                self.views.push(Screens::Settings);
            }
            Message::LaunchGame(ip_port) => {
                if let Err(error) = self.launcher.launch(&self.user_settings.steam_executable_path, &ip_port) {
                    eprintln!("Error: {}", error);
                }
            }
            Message::Settings(settings_message) => {
                self.process_settings_message(settings_message);
            }
            Message::Bookmarked(ip_port, bookmarked) => {
                match bookmarked {
                    true => self.bookmarks.add(ip_port),
                    false => self.bookmarks.remove(&ip_port),
                }
            }
            Message::CopyToClipboard(connection_string) => {
                return iced::clipboard::write(connection_string)
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message, Renderer<Self::Theme>> {
        match self.views.current().expect("valid view") {
            Screens::Main(view) => ui::main::view(view, &self.bookmarks),
            Screens::Settings => ui::settings::view(&self.user_settings),
        }
        .into()
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        use iced::futures::StreamExt;

        let urls = self
            .server_urls
            .iter()
            .map(|server_url| UrlWithKey::new(server_url, &self.user_settings.teamwork_api_key))
            .collect();
        let server_stream = fetch_servers(urls).map(|event| Message::from(event));

        Subscription::batch([
            subscription::run(self.fetch_servers_subscription_id, server_stream),
            geolocation::subscription().map(Message::from),
            ping::subscription().map(Message::from),
            thumbnail::subscription(&self.user_settings.teamwork_api_key).map(Message::from),
        ])
    }
}
