mod bookmarks;
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
pub mod servers_source;
mod text_filter;
mod thumbnail;
mod user_settings;
mod views;

use {
    iced::widget::{
        column,
        pane_grid::{self, Axis, Pane},
    },
    serde::{Deserialize, Serialize},
    std::{
        cmp::Ordering,
        collections::btree_map::Entry::{Occupied, Vacant},
        ops::Add,
        path::PathBuf,
    },
};

use {
    crate::{application::views::Views, ui},
    iced::{
        futures::{
            channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
            FutureExt, SinkExt, TryFutureExt,
        },
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

use crate::application::{
    bookmarks::{read_file, write_file},
    launcher::{ExecutableLauncher, LaunchError},
    servers_source::{ServersSource, SourceKey},
};
pub use {
    crate::application::user_settings::UserSettings,
    bookmarks::Bookmarks,
    country::Country,
    fetch_servers::{fetch_servers, FetchServersEvent},
    filter_servers::Filter,
    ip_port::IpPort,
    message::{
        CountryServiceMessage, FetchServersMessage, FilterMessage, Message, PaneMessage, PingServiceMessage,
        SettingsMessage, ThumbnailMessage,
    },
    promised_value::PromisedValue,
    server::Server,
};

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
    pub panes: pane_grid::State<PaneView>,
}

impl MainView {
    pub fn new(pane_ratio: f32) -> Self {
        let (mut panes, servers_pane) = pane_grid::State::new(PaneView::new(PaneId::Servers));

        if let Some((_filter_pane, split)) = panes.split(Axis::Vertical, &servers_pane, PaneView::new(PaneId::Filters)) {
            panes.resize(&split, pane_ratio);
        }

        Self { panes }
    }
}

pub struct TeamworkLauncher {
    views: Views<Screens>,
    servers: Vec<Server>,
    user_settings: UserSettings,
    filter: Filter,
    servers_sources: Vec<ServersSource>,
    launcher: ExecutableLauncher,
    bookmarks: Bookmarks,
    country_sender: Option<UnboundedSender<Ipv4Addr>>,
    ping_sender: Option<UnboundedSender<Ipv4Addr>>,
    thumbnail_sender: Option<UnboundedSender<String>>,
    fetch_servers_subscription_id: u64,
}

impl TeamworkLauncher {
    fn new_servers(&mut self, mut new_servers: Vec<Server>) {
        let countries: Vec<Country> = new_servers
            .iter()
            .filter_map(|server| server.country.get())
            .unique()
            .cloned()
            .collect();

        self.filter.country.extend_available(&countries);
        self.servers.extend(new_servers.into_iter());
        self.servers.sort_by(Self::sort_servers);
    }

    fn on_finish(&mut self) {
        let mut servers_refs: Vec<&Server> = self.servers.iter().collect();

        servers_refs.sort_by(|l, r| {
            let left = self.bookmarks.is_bookmarked(&l.ip_port);
            let right = self.bookmarks.is_bookmarked(&r.ip_port);

            right.cmp(&left)
        });

        for map_name in servers_refs.iter().map(|server| server.map.clone()).unique() {
            if let Some(thumbnail_sender) = &mut self.thumbnail_sender {
                thumbnail_sender
                    .send(map_name.clone())
                    .unwrap_or_else(|e| eprintln!("thumbnail sender {}", e))
                    .now_or_never();
            }
        }

        for ip in servers_refs.iter().map(|server| server.ip_port.ip()).unique().cloned() {
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
    }

    fn sort_servers(l: &Server, r: &Server) -> Ordering {
        l.ip_port.cmp(&r.ip_port)
    }

    fn refresh_servers(&mut self) {
        if let Some(Screens::Main(view)) = self.views.current_mut() {
            self.servers.clear();
            self.filter.country.clear_available();
        }
        self.fetch_servers_subscription_id += 1;
    }

    fn country_found(&mut self, ip: Ipv4Addr, country: Country) {
        self.filter.country.add_available(country.clone());
        for server in self.servers.iter_mut().filter(|server| server.ip_port.ip() == &ip) {
            server.country = PromisedValue::Ready(country.clone());
        }
    }

    fn ping_found(&mut self, ip: Ipv4Addr, duration: Option<Duration>) {
        for server in self.servers.iter_mut().filter(|server| server.ip_port.ip() == &ip) {
            server.ping = duration.into();
        }
    }

    fn thumbnail_ready(&mut self, map_name: String, thumbnail: Option<image::Handle>) {
        for server in self.servers.iter_mut().filter(|server| server.map == map_name) {
            server.map_thumbnail = thumbnail.clone().into();
        }
    }

    fn process_pane_message(&mut self, message: PaneMessage) {
        match message {
            PaneMessage::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                if let Some(Screens::Main(view)) = self.views.current_mut() {
                    self.user_settings.servers_filter_pane_ratio = ratio;
                    view.panes.resize(&split, ratio);
                }
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
            SettingsMessage::SourceEnabled(source_key, enabled) => {
                if let Some(source) = self.servers_sources.iter_mut().find(|source| source.key() == &source_key) {
                    source.set_enabled(enabled);
                }
            }
        }
    }

    fn process_filter_message(&mut self, message: FilterMessage) {
        match message {
            FilterMessage::CountryChecked(country, checked) => {
                self.filter.country.set_checked(&country, checked);
            }
            FilterMessage::NoCountryChecked(checked) => {
                self.filter.country.set_accept_no_country(checked);
            }
            FilterMessage::TextChanged(text) => {
                self.filter.text.set_text(&text);
            }
            FilterMessage::BookmarkedOnlyChecked(checked) => {
                self.filter.bookmarked_only = checked;
            }
        }
    }

    /// Get the list of URLS to get the servers information.
    ///
    /// The order is specified by the bookmarks. The rule is
    /// the source with the greater count of servers goes first.
    fn get_sources_urls(&self) -> Vec<(SourceKey, UrlWithKey)> {
        let urls = self
            .servers_sources
            .iter()
            .filter_map(|source| match source.enabled() {
                true => Some((
                    source.key().clone(),
                    UrlWithKey::new(source.url(), &self.user_settings.teamwork_api_key),
                )),
                false => None,
            })
            .collect();

        urls
    }

    fn bookmark(&mut self, ip_port: IpPort, bookmarked: bool) {
        match bookmarked {
            true => {
                if let Some(source_key) = self
                    .servers
                    .iter()
                    .find(|server| server.ip_port == ip_port)
                    .map(|server| server.source_key.clone())
                    .flatten()
                {
                    self.bookmarks.add(ip_port, source_key);
                }
            }
            false => {
                if let Some(source_key) = self
                    .servers
                    .iter()
                    .find(|server| server.ip_port == ip_port)
                    .map(|server| server.source_key.as_ref())
                    .flatten()
                {
                    self.bookmarks.remove(&ip_port, source_key);
                }
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
            std::fs::create_dir_all(&configuration_directory).unwrap_or_else(|error| {
                eprintln!(
                    "Failed to create configuration directory '{}': {}",
                    configuration_directory.display(),
                    error
                )
            });
        }
        let bookmarks_file_path = configuration_directory.join("bookmarks.json");
        let settings_file_path = configuration_directory.join("settings.json");
        let filters_file_path = configuration_directory.join("filters.json");
        let sources_file_path = configuration_directory.join("sources.json");

        write_file(&self.bookmarks, &bookmarks_file_path).unwrap_or_else(|error| {
            eprintln!(
                "Failed to write bookmarks file '{}': {}",
                bookmarks_file_path.display(),
                error
            )
        });
        write_file(&self.user_settings, &settings_file_path).unwrap_or_else(|error| {
            eprintln!("Failed to write settings file '{}': {}", settings_file_path.display(), error)
        });
        write_file(&self.filter, &filters_file_path)
            .unwrap_or_else(|error| eprintln!("Failed to write filters file '{}': {}", filters_file_path.display(), error));
        write_file(&self.servers_sources, &sources_file_path)
            .unwrap_or_else(|error| eprintln!("Failed to write sources file '{}': {}", sources_file_path.display(), error));
    }
}

pub enum PaneId {
    Servers,
    Filters,
}

pub struct PaneView {
    pub id: PaneId,
}

impl PaneView {
    fn new(id: PaneId) -> Self {
        Self { id }
    }
}

impl iced::Application for TeamworkLauncher {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let configuration_directory = get_configuration_directory();
        let bookmarks: Bookmarks = read_file(&configuration_directory.join("bookmarks.json")).unwrap_or_default();
        let user_settings: UserSettings = read_file(&configuration_directory.join("settings.json")).unwrap_or_default();
        let filter: Filter = read_file(&configuration_directory.join("filters.json")).unwrap_or_default();
        let servers_sources: Vec<ServersSource> =
            read_file(&configuration_directory.join("sources.json")).unwrap_or_else(|error| {
                vec![
                    ServersSource::new("Payload", "https://teamwork.tf/api/v1/quickplay/payload/servers"),
                    ServersSource::new("Payload Race", "https://teamwork.tf/api/v1/quickplay/payload-race/servers"),
                    ServersSource::new("King Of The Hill", "https://teamwork.tf/api/v1/quickplay/koth/servers"),
                    ServersSource::new("Capture The Flag", "https://teamwork.tf/api/v1/quickplay/ctf/servers"),
                    ServersSource::new("Attack/Defend", "https://teamwork.tf/api/v1/quickplay/attack-defend/servers"),
                    ServersSource::new("Control Point", "https://teamwork.tf/api/v1/quickplay/control-point/servers"),
                    ServersSource::new("Medieval Mode", "https://teamwork.tf/api/v1/quickplay/medieval-mode/servers"),
                ]
            });

        (
            Self {
                views: Views::new(Screens::Main(MainView::new(user_settings.servers_filter_pane_ratio))),
                servers: Vec::new(),
                user_settings,
                filter,
                servers_sources,
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
                self.on_finish();
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
            Message::Pane(message) => {
                self.process_pane_message(message);
            }
            Message::Bookmarked(ip_port, bookmarked) => {
                self.bookmark(ip_port, bookmarked);
            }
            Message::CopyToClipboard(connection_string) => return iced::clipboard::write(connection_string),
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message, Renderer<Self::Theme>> {
        let current = self.views.current().expect("valid view");

        column![
            ui::header::header_view("Teamwork Launcher", current),
            match current {
                Screens::Main(view) => ui::main::view(view, &self.servers, &self.bookmarks, &self.filter),
                Screens::Settings => ui::settings::view(&self.user_settings, &self.servers_sources),
            }
        ]
        .into()
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        use iced::futures::StreamExt;

        let urls = self.get_sources_urls();
        let server_stream = fetch_servers(urls).map(|event| Message::from(event));

        Subscription::batch([
            subscription::run(self.fetch_servers_subscription_id, server_stream),
            geolocation::subscription().map(Message::from),
            ping::subscription().map(Message::from),
            thumbnail::subscription(&self.user_settings.teamwork_api_key).map(Message::from),
        ])
    }
}
