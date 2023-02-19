mod bookmarks;
pub mod country;
mod country_filter;
pub mod fetch_servers;
pub mod filter_servers;
pub mod game_mode;
mod game_mode_filter;
mod geolocation;
pub mod ip_port;
mod launcher;
mod map;
mod message;
mod ping;
mod process_detection;
pub mod promised_value;
pub mod properties_filter;
pub mod server;
pub mod servers_source;
pub mod sort_servers;
mod text_filter;
mod thumbnail;
pub mod user_settings;
mod views;

use {
    iced::{
        system, theme,
        widget::{
            column, container,
            pane_grid::{self, Axis},
        },
        Background, Color, Theme,
    },
    std::collections::{
        btree_map::Entry::{Occupied, Vacant},
        BTreeMap,
    },
};

use {
    crate::{application::views::Views, ui},
    iced::{
        futures::{channel::mpsc::UnboundedSender, FutureExt, SinkExt, TryFutureExt},
        subscription,
        widget::image,
        Command, Element, Renderer, Subscription,
    },
    itertools::Itertools,
    std::{net::Ipv4Addr, sync::Arc, time::Duration},
    teamwork::UrlWithKey,
};

use crate::{
    application::{
        game_mode::{GameModeId, GameModes},
        launcher::ExecutableLauncher,
        map::MapName,
        message::KeyboardMessage,
        servers_source::{ServersSource, SourceKey},
        sort_servers::{sort_servers, SortDirection},
    },
    common_settings::{get_configuration_directory, write_file},
    ApplicationFlags,
};
pub use {
    crate::application::user_settings::UserSettings,
    bookmarks::Bookmarks,
    country::Country,
    fetch_servers::{fetch_servers, FetchServersEvent},
    filter_servers::Filter,
    ip_port::IpPort,
    message::{
        CountryServiceMessage, FetchServersMessage, FilterMessage, GameModesMessage, Message, PaneMessage,
        PingServiceMessage, SettingsMessage, ThumbnailMessage,
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
    Server(IpPort),
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

#[derive(Default)]
pub struct ServersCounts {
    pub bookmarks: usize,
    pub timeouts: usize,
    pub countries: BTreeMap<Country, usize>,
    pub game_modes: BTreeMap<GameModeId, usize>,
    pub properties: BTreeMap<Property, usize>,
}

impl ServersCounts {
    pub fn reset(&mut self) {
        *self = ServersCounts::default();
    }

    pub fn add_country(&mut self, country: Country) {
        match self.countries.entry(country) {
            Vacant(vacant) => {
                vacant.insert(1);
            }
            Occupied(mut occupied) => {
                *occupied.get_mut() += 1;
            }
        };
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Property {
    Rtd,
    AllTalk,
    NoRespawnTime,
    Password,
    VacSecured,
}

pub struct TeamworkLauncher {
    views: Views<Screens>,
    servers: Vec<Server>,
    servers_counts: ServersCounts,
    user_settings: UserSettings,
    filter: Filter,
    servers_sources: Vec<ServersSource>,
    launcher: ExecutableLauncher,
    bookmarks: Bookmarks,
    game_modes: GameModes,
    system_info: Option<system::Information>,
    country_request_sender: Option<UnboundedSender<Ipv4Addr>>,
    ping_request_sender: Option<UnboundedSender<Ipv4Addr>>,
    map_thumbnail_request_sender: Option<UnboundedSender<MapName>>,
    fetch_servers_subscription_id: u64,
    shift_pressed: bool,
    theme: Theme,
}

impl TeamworkLauncher {
    fn new_servers(&mut self, new_servers: Vec<Server>) {
        let countries: Vec<Country> = new_servers
            .iter()
            .filter_map(|server| server.country.get())
            .unique()
            .cloned()
            .collect();

        self.filter.country.extend_available(&countries);
        self.servers.extend(new_servers.into_iter());
        self.sort_server();
    }

    fn on_finish(&mut self) {
        self.sort_server();

        let mut servers_refs: Vec<&Server> = self.servers.iter().collect();

        servers_refs.sort_by(|l, r| {
            let left = self.bookmarks.is_bookmarked(&l.ip_port);
            let right = self.bookmarks.is_bookmarked(&r.ip_port);

            right.cmp(&left)
        });

        for map_name in servers_refs.iter().map(|server| server.map.clone()).unique() {
            if let Some(thumbnail_sender) = &mut self.map_thumbnail_request_sender {
                thumbnail_sender
                    .send(map_name.clone())
                    .unwrap_or_else(|e| eprintln!("thumbnail sender {}", e))
                    .now_or_never();
            }
        }

        for ip in servers_refs.iter().map(|server| server.ip_port.ip()).unique() {
            if let Some(country_sender) = &mut self.country_request_sender {
                country_sender
                    .send(*ip)
                    .unwrap_or_else(|e| eprintln!("country sender {}", e))
                    .now_or_never();
            }

            if let Some(ping_sender) = &mut self.ping_request_sender {
                ping_sender
                    .send(*ip)
                    .unwrap_or_else(|e| eprintln!("ping sender {}", e))
                    .now_or_never();
            }
        }

        self.servers_counts.bookmarks =
            self.servers
                .iter()
                .fold(0usize, |count, server| match self.bookmarks.is_bookmarked(&server.ip_port) {
                    true => count + 1,
                    false => count,
                });
        self.servers_counts.countries =
            Self::histogram(self.servers.iter().filter_map(|server| server.country.get()).cloned());
        self.servers_counts.properties = Self::count_properties(&self.servers);
        self.servers_counts.game_modes = Self::histogram(self.servers.iter().flat_map(|server| server.game_modes.clone()));
        self.servers_counts.timeouts = self.servers.iter().filter(|server| server.ping.is_none()).count();

        self.filter.players.maximum_players = self.servers.iter().fold(0u8, |mut max, server| {
            if max < server.current_players_count {
                max = server.current_players_count;
            }
            max
        });

        self.filter.players.maximum_free_slots = self.servers.iter().fold(0u8, |mut max, server| {
            if max < server.max_players_count {
                max = server.max_players_count;
            }
            max
        });
    }

    fn refresh_servers(&mut self) {
        self.servers_counts.reset();
        self.servers.clear();

        self.filter.country.clear_available();
        self.filter.players.maximum_free_slots = 0;
        self.filter.players.maximum_players = 0;

        self.fetch_servers_subscription_id += 1;
    }

    fn country_found(&mut self, ip: Ipv4Addr, country: Country) {
        self.filter.country.add_available(country.clone());
        for server in self.servers.iter_mut().filter(|server| server.ip_port.ip() == &ip) {
            server.country = PromisedValue::Ready(country.clone());
        }
        self.servers_counts.add_country(country);
    }

    fn ping_found(&mut self, ip: Ipv4Addr, duration: Option<Duration>) {
        for server in self.servers.iter_mut().filter(|server| server.ip_port.ip() == &ip) {
            if duration.is_none() {
                self.servers_counts.timeouts += 1;
            }

            server.ping = duration.into();
        }
    }

    fn thumbnail_ready(&mut self, map_name: MapName, thumbnail: Option<image::Handle>) {
        for server in self.servers.iter_mut().filter(|server| server.map == map_name) {
            if !server.map_thumbnail.is_ready() {
                server.map_thumbnail = thumbnail.clone().into();
            }
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
            SettingsMessage::QuitWhenLaunchChecked(checked) => {
                self.user_settings.quit_on_launch = checked;
            }
            SettingsMessage::QuitWhenCopyChecked(checked) => {
                self.user_settings.quit_on_copy = checked;
            }
            SettingsMessage::WindowMoved { x, y } => {
                if let Some(settings) = &mut self.user_settings.window {
                    settings.window_x = x;
                    settings.window_y = y;
                }
            }
            SettingsMessage::WindowResized { width, height } => {
                if let Some(settings) = &mut self.user_settings.window {
                    settings.window_width = width;
                    settings.window_height = height;
                }
            }
            SettingsMessage::ThemeChanged(theme) => {
                self.user_settings.theme = theme;
                self.theme = theme.into();
            }
        }
    }

    fn process_filter_message(&mut self, message: FilterMessage) {
        match message {
            FilterMessage::CountryChecked(country, checked) => {
                if self.shift_pressed {
                    match self.filter.country.is_checked(&country) {
                        false => self.filter.country.check_all_excepted(&country),
                        true => self.filter.country.check_only(&country),
                    }
                } else {
                    self.filter.country.set_checked(&country, checked);
                }
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
            FilterMessage::IgnoreCaseChanged(checked) => {
                self.filter.text.ignore_case = checked;
            }
            FilterMessage::IgnoreAccentChanged(checked) => {
                self.filter.text.ignore_accents = checked;
            }
            FilterMessage::MaxPingChanged(max_ping) => {
                self.filter.max_ping = max_ping;
            }
            FilterMessage::AcceptPingTimeoutChanged(checked) => {
                self.filter.accept_ping_timeout = checked;
            }
            FilterMessage::GameModeChecked(id, checked) => {
                if self.shift_pressed {
                    match self.filter.game_modes.is_mode_enabled(&id) {
                        false => self.filter.game_modes.enable_all_excepted(&id),
                        true => self.filter.game_modes.enable_only(&id),
                    }
                } else {
                    self.filter.game_modes.set_mode_enabled(&id, checked);
                }
            }
            FilterMessage::CountryFilterEnabled(checked) => {
                self.filter.country.set_enabled(checked);
            }
            FilterMessage::GameModeFilterEnabled(checked) => {
                self.filter.game_modes.set_enabled(checked);
            }
            FilterMessage::VacSecuredChanged(checked) => {
                self.filter.vac_secured = checked;
            }
            FilterMessage::RtdChanged(checked) => {
                self.filter.rtd = checked;
            }
            FilterMessage::AllTalkChanged(checked) => {
                self.filter.all_talk = checked;
            }
            FilterMessage::NoRespawnTimeChanged(checked) => {
                self.filter.no_respawn_time = checked;
            }
            FilterMessage::PasswordChanged(checked) => {
                self.filter.password = checked;
            }
            FilterMessage::SortCriterionChanged(criterion) => {
                self.filter.sort_criterion = criterion;
                self.sort_server();
            }
            FilterMessage::SortDirectionChanged(direction) => {
                self.filter.sort_direction = direction;
                self.sort_server();
            }
            FilterMessage::MinimumPlayersChanged(value) => {
                self.filter.players.minimum_players = value;
            }
            FilterMessage::MinimumFreeSlotsChanged(value) => {
                self.filter.players.minimum_free_slots = value;
            }
        }
    }

    fn sort_server(&mut self) {
        match self.filter.sort_direction {
            SortDirection::Ascending => {
                self.servers.sort_by(|l, r| sort_servers(self.filter.sort_criterion, l, r));
            }
            SortDirection::Descending => {
                self.servers.sort_by(|l, r| sort_servers(self.filter.sort_criterion, r, l));
            }
        }
    }

    fn process_game_modes_message(&mut self, message: GameModesMessage) {
        match message {
            GameModesMessage::GameModes(game_modes) => {
                self.game_modes.reset(&game_modes);
                self.filter.game_modes.reset(&game_modes);
            }
            GameModesMessage::Error(error) => {
                eprintln!("Failed to fetch game modes: {}", error)
            }
        }
    }

    fn process_keyboard_message(&mut self, message: KeyboardMessage) {
        match message {
            KeyboardMessage::ShiftPressed => {
                self.shift_pressed = true;
            }
            KeyboardMessage::ShiftReleased => {
                self.shift_pressed = false;
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

    #[allow(clippy::map_flatten)]
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
                    self.servers_counts.bookmarks += 1;
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
                    self.servers_counts.bookmarks -= 1;
                }
            }
        }
    }

    fn launch_game(&self, ip_port: &IpPort) -> Command<Message> {
        match self.launcher.launch(&self.user_settings.steam_executable_path, ip_port) {
            Err(error) => {
                eprintln!("Error: {}", error);
            }
            Ok(()) => {
                if self.user_settings.quit_on_launch {
                    return iced::window::close();
                }
            }
        }

        Command::none()
    }

    fn copy_connection_string(&self, ip_port: IpPort) -> Command<Message> {
        let connection_string = ip_port.steam_connection_string();

        match self.user_settings.quit_on_copy {
            false => Command::batch([iced::clipboard::write(connection_string)]),
            true => Command::batch([iced::clipboard::write(connection_string), iced::window::close()]),
        }
    }

    /// Count each element.
    /// For example with this collection `[3, 3, 3, 2, 2, 1]`
    /// The result will be: `3 -> 3, 2 -> 2, 1 -> 1`
    fn histogram<T: Ord>(values: impl Iterator<Item = T>) -> BTreeMap<T, usize> {
        values.fold(BTreeMap::new(), |mut count, value| {
            match count.entry(value) {
                Vacant(vacant) => {
                    vacant.insert(1usize);
                }
                Occupied(mut occupied) => {
                    *occupied.get_mut() += 1;
                }
            }

            count
        })
    }

    fn increment_count(count: &mut BTreeMap<Property, usize>, property: Property) {
        match count.entry(property) {
            Vacant(vacant) => {
                vacant.insert(1usize);
            }
            Occupied(mut occupied) => {
                *occupied.get_mut() += 1;
            }
        }
    }

    /// Count how many servers with each properties.
    /// I can't use `histogram`.
    fn count_properties(servers: &[Server]) -> BTreeMap<Property, usize> {
        let mut count = BTreeMap::new();

        for server in servers {
            if server.need_password {
                Self::increment_count(&mut count, Property::Password);
            } else if server.has_no_respawn_time {
                Self::increment_count(&mut count, Property::NoRespawnTime);
            } else if server.has_rtd {
                Self::increment_count(&mut count, Property::Rtd);
            } else if server.has_all_talk {
                Self::increment_count(&mut count, Property::AllTalk);
            } else if server.vac_secured {
                Self::increment_count(&mut count, Property::VacSecured);
            }
        }

        count
    }
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

mod palettes {
    use iced::{theme, Color};

    pub fn create_blue_palette() -> theme::Custom {
        theme::Custom::new(theme::palette::Palette {
            background: Color::from_rgb8(38, 35, 33),
            text: Color::WHITE,
            primary: Color::from_rgb8(57, 92, 120),
            success: Default::default(),
            danger: Default::default(),
        })
    }

    pub fn create_red_palette() -> theme::Custom {
        theme::Custom::new(theme::palette::Palette {
            background: Color::from_rgb8(38, 35, 33),
            text: Color::WHITE,
            primary: Color::from_rgb8(159, 49, 47),
            success: Default::default(),
            danger: Default::default(),
        })
    }
}

impl iced::Application for TeamworkLauncher {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ApplicationFlags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let theme: Theme = flags.user_settings.theme.into();

        (
            Self {
                views: Views::new(Screens::Main(MainView::new(flags.user_settings.servers_filter_pane_ratio))),
                servers: Vec::new(),
                servers_counts: ServersCounts::default(),
                user_settings: flags.user_settings,
                filter: flags.filter,
                servers_sources: flags.servers_sources,
                bookmarks: flags.bookmarks,
                launcher: ExecutableLauncher::new(false),
                game_modes: GameModes::new(),
                country_request_sender: None,
                ping_request_sender: None,
                map_thumbnail_request_sender: None,
                fetch_servers_subscription_id: 0,
                shift_pressed: false,
                system_info: None,
                theme,
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
                self.country_request_sender = Some(country_sender);
                eprintln!("country service started");
            }
            Message::Country(CountryServiceMessage::CountryFound(ip, country)) => {
                self.country_found(ip, country);
            }
            Message::Country(CountryServiceMessage::Error(error)) => {
                eprintln!("Error: {}", error);
            }
            Message::Ping(PingServiceMessage::Started(sender)) => {
                self.ping_request_sender = Some(sender);
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
                self.map_thumbnail_request_sender = Some(sender);
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
            Message::GameModes(message) => {
                self.process_game_modes_message(message);
            }
            Message::Keyboard(message) => {
                self.process_keyboard_message(message);
            }
            Message::Back => {
                self.views.pop();
            }
            Message::ShowSettings => {
                self.views.push(Screens::Settings);
                return system::fetch_information(Message::SystemInfoUpdated);
            }
            Message::LaunchGame(ip_port) => {
                return self.launch_game(&ip_port);
            }
            Message::CopyConnectionString(ip_port) => {
                return self.copy_connection_string(ip_port);
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
            Message::CopyToClipboard(text) => {
                return iced::clipboard::write(text);
            }
            Message::ShowServer(ip_port) => {
                self.views.push(Screens::Server(ip_port));
            }
            Message::SystemInfoUpdated(information) => {
                self.system_info = Some(information);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message, Renderer<Self::Theme>> {
        let current = self.views.current().expect("valid view");

        container(column![
            ui::header::header_view("Teamwork Launcher", current),
            match current {
                Screens::Main(view) => ui::main::view(
                    view,
                    &self.servers,
                    &self.bookmarks,
                    &self.filter,
                    &self.game_modes,
                    &self.servers_counts
                ),
                Screens::Server(ip_port) => ui::server::view(&self.servers, &self.game_modes, ip_port),
                Screens::Settings =>
                    ui::settings::view(&self.user_settings, &self.servers_sources, self.system_info.as_ref()),
            }
        ])
        .style(theme::Container::Custom(Box::new(MainBackground {})))
        .into()
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        use iced::futures::StreamExt;

        let urls = self.get_sources_urls();
        let server_stream = fetch_servers(urls).map(Message::from);

        Subscription::batch([
            subscription::run(self.fetch_servers_subscription_id, server_stream),
            geolocation::subscription().map(Message::from),
            ping::subscription().map(Message::from),
            thumbnail::subscription(&self.user_settings.teamwork_api_key).map(Message::from),
            game_mode::subscription(self.fetch_servers_subscription_id, &self.user_settings.teamwork_api_key)
                .map(Message::from),
            keyboard::subscription().map(Message::from),
            window::subscription(),
        ])
    }
}

struct MainBackground;

impl container::StyleSheet for MainBackground {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb8(23, 21, 20))),
            ..Default::default()
        }
    }
}

mod window {
    use {
        crate::application::{Message, SettingsMessage},
        iced::{event, subscription, window, Event, Subscription},
    };

    pub fn subscription() -> Subscription<Message> {
        subscription::events_with(|event, status| {
            if let event::Status::Captured = status {
                return None;
            }

            if let Event::Window(window::Event::Moved { x, y }) = event {
                return Some(Message::Settings(SettingsMessage::WindowMoved { x, y }));
            }

            if let Event::Window(window::Event::Resized { width, height }) = event {
                return Some(Message::Settings(SettingsMessage::WindowResized { width, height }));
            }

            None
        })
    }
}

mod keyboard {
    use {
        crate::application::message::KeyboardMessage,
        iced::{
            event,
            keyboard::{self, KeyCode},
            subscription, Event, Subscription,
        },
    };

    pub fn subscription() -> Subscription<KeyboardMessage> {
        subscription::events_with(|event, status| {
            if let event::Status::Captured = status {
                return None;
            }

            match event {
                Event::Keyboard(keyboard::Event::KeyPressed { modifiers: _, key_code }) => {
                    if key_code == KeyCode::LShift || key_code == KeyCode::RShift {
                        return Some(KeyboardMessage::ShiftPressed);
                    }
                }
                Event::Keyboard(keyboard::Event::KeyReleased { modifiers: _, key_code }) => {
                    if key_code == KeyCode::LShift || key_code == KeyCode::RShift {
                        return Some(KeyboardMessage::ShiftReleased);
                    }
                }
                _ => {}
            }

            None
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::application::TeamworkLauncher;

    #[test]
    fn test_histogram() {
        let numbers = vec![3, 3, 3, 2, 2, 1];
        let h = TeamworkLauncher::histogram(numbers.iter());

        assert_eq!(h.get(&3), Some(&3));
        assert_eq!(h.get(&2), Some(&2));
        assert_eq!(h.get(&1), Some(&1));
        assert_eq!(h.get(&0), None);
    }
}
