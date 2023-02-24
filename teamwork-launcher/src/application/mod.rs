mod bookmarks;
pub mod country;
pub mod fetch_servers;
pub mod filter;
pub mod game_mode;
mod geolocation;
pub mod ip_port;
mod launcher;
mod map;
pub mod message;
pub mod notifications;
mod ping;
mod process_detection;
pub mod promised_value;
pub mod server;
pub mod servers_counts;
pub mod servers_source;
mod thumbnail;
pub mod user_settings;
mod views;

use {
    iced::{
        theme,
        widget::{
            column, container,
            pane_grid::{self, Axis},
        },
        Background, Color, Theme,
    },
    log::{debug, error},
    std::{
        collections::{
            btree_map::Entry::{Occupied, Vacant},
            BTreeMap,
        },
        time::Instant,
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

pub use {
    crate::application::user_settings::UserSettings,
    bookmarks::Bookmarks,
    country::Country,
    fetch_servers::{fetch_servers, FetchServersEvent},
    ip_port::IpPort,
    message::{
        CountryServiceMessage, FetchServersMessage, FilterMessage, GameModesMessage, Message, PaneMessage,
        PingServiceMessage, SettingsMessage, ThumbnailMessage,
    },
    promised_value::PromisedValue,
    server::Server,
};
use {
    crate::{
        application::{
            filter::{
                filter_servers::Filter,
                sort_servers::{sort_servers, SortDirection},
            },
            game_mode::{GameModeId, GameModes},
            launcher::ExecutableLauncher,
            map::MapName,
            message::{KeyboardMessage, NotificationMessage},
            notifications::{Notification, NotificationKind, Notifications},
            process_detection::ProcessDetection,
            servers_source::{ServersSource, SourceKey},
        },
        common_settings::{get_configuration_directory, write_file},
        ApplicationFlags,
    },
    servers_counts::ServersCounts,
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

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Property {
    Rtd,
    AllTalk,
    NoRespawnTime,
    Password,
    VacSecured,
    RandomCrits,
}

pub struct TeamworkLauncher {
    views: Views<Screens>,
    servers: Vec<Server>,
    servers_counts: ServersCounts,
    user_settings: UserSettings,
    filter: Filter,
    servers_sources: Vec<ServersSource>,
    launcher: ExecutableLauncher,
    process_detection: ProcessDetection,
    bookmarks: Bookmarks,
    game_modes: GameModes,
    country_request_sender: Option<UnboundedSender<Ipv4Addr>>,
    ping_request_sender: Option<UnboundedSender<Ipv4Addr>>,
    map_thumbnail_request_sender: Option<UnboundedSender<MapName>>,
    fetch_servers_subscription_id: u64,
    shift_pressed: bool,
    theme: Theme,
    is_loading: bool,
    notifications: Notifications,
}

impl TeamworkLauncher {
    fn new_servers(&mut self, new_servers: Vec<Server>) {
        let countries: Vec<Country> = new_servers
            .iter()
            .filter_map(|server| server.country.get())
            .unique()
            .cloned()
            .collect();

        self.filter.country.dictionary.extend(countries.into_iter());

        self.servers.extend(new_servers.into_iter());
        self.sort_server();
    }

    fn on_finish(&mut self) {
        self.is_loading = false;
        self.sort_server();

        let mut servers_refs: Vec<&Server> = self.servers.iter().collect();

        servers_refs.sort_by(|l, r| {
            let left = self.bookmarks.is_bookmarked(&l.ip_port);
            let right = self.bookmarks.is_bookmarked(&r.ip_port);

            right.cmp(&left)
        });

        let unique_maps: Vec<&MapName> = servers_refs.iter().map(|server| &server.map).unique().collect();

        // For each map, request the thumbnail
        for map_name in unique_maps.iter().map(|name| (*name).clone()) {
            if let Some(thumbnail_sender) = &mut self.map_thumbnail_request_sender {
                thumbnail_sender
                    .send(map_name)
                    .unwrap_or_else(|e| error!("thumbnail sender {}", e))
                    .now_or_never();
            }
        }

        // Request country for each server
        for ip in servers_refs.iter().map(|server| server.ip_port.ip()).unique() {
            if let Some(country_sender) = &mut self.country_request_sender {
                country_sender
                    .send(*ip)
                    .unwrap_or_else(|e| error!("country sender {}", e))
                    .now_or_never();
            }

            if let Some(ping_sender) = &mut self.ping_request_sender {
                ping_sender
                    .send(*ip)
                    .unwrap_or_else(|e| error!("ping sender {}", e))
                    .now_or_never();
            }
        }

        // Update counts
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
        self.servers_counts.maps = Self::histogram(self.servers.iter().map(|server| server.map.clone()));
        self.servers_counts.providers = Self::histogram(self.servers.iter().map(|server| server.provider.clone()));

        // Update filters
        self.filter
            .providers
            .dictionary
            .extend(self.servers.iter().map(|server| server.provider.clone()));

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

        for map_name in unique_maps.into_iter() {
            self.filter.maps.dictionary.add(map_name.clone());
        }
    }

    fn refresh_servers(&mut self) {
        if self.user_settings.teamwork_api_key().trim().is_empty() {
            self.push_notification(
                "No Teamwork.tf API key specified.\nSet your API key in the settings.",
                NotificationKind::Error,
            );
        } else {
            self.is_loading = true;
            self.servers_counts.reset();
            self.servers.clear();
            self.filter.players.maximum_free_slots = 0;
            self.filter.players.maximum_players = 0;
            self.fetch_servers_subscription_id += 1;
        }
    }

    fn country_found(&mut self, ip: Ipv4Addr, country: Country) {
        self.filter.country.dictionary.add(country.clone());

        for server in self.servers.iter_mut().filter(|server| server.ip_port.ip() == &ip) {
            server.country = PromisedValue::Ready(country.clone());
        }

        self.servers_counts.add_country(country);

        self.sort_server();
    }

    fn ping_found(&mut self, ip: Ipv4Addr, duration: Option<Duration>) {
        for server in self.servers.iter_mut().filter(|server| server.ip_port.ip() == &ip) {
            if duration.is_none() {
                self.servers_counts.timeouts += 1;
            }

            server.ping = duration.into();
        }

        self.sort_server();
    }

    fn thumbnail_ready(&mut self, map_name: MapName, thumbnail: Option<image::Handle>) {
        for server in self.servers.iter_mut().filter(|server| server.map == map_name) {
            if !server.map_thumbnail.is_ready() {
                server.map_thumbnail = thumbnail.clone().into();
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
                self.user_settings.set_teamwork_api_key(key);
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
                    match self.filter.country.dictionary.is_checked(&country) {
                        false => self.filter.country.dictionary.uncheck_only(&country),
                        true => self.filter.country.dictionary.check_only(&country),
                    }
                } else {
                    self.filter.country.dictionary.set_checked(&country, checked);
                }
            }
            FilterMessage::NoCountryChecked(checked) => {
                self.filter.country.no_countries = checked;
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
                    match self.filter.game_modes.dictionary.is_checked(&id) {
                        false => self.filter.game_modes.dictionary.uncheck_only(&id),
                        true => self.filter.game_modes.dictionary.check_only(&id),
                    }
                } else {
                    self.filter.game_modes.dictionary.set_checked(&id, checked);
                }
            }
            FilterMessage::CountryFilterEnabled(checked) => {
                self.filter.country.enabled = checked;
            }
            FilterMessage::GameModeFilterEnabled(checked) => {
                self.filter.game_modes.enabled = checked;
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
            FilterMessage::RandomCritsChanged(checked) => {
                self.filter.random_crits = checked;
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
            FilterMessage::MapChecked(map, enabled) => {
                if self.shift_pressed {
                    match self.filter.maps.dictionary.is_checked(&map) {
                        false => self.filter.maps.dictionary.uncheck_only(&map),
                        true => self.filter.maps.dictionary.check_only(&map),
                    }
                } else {
                    self.filter.maps.dictionary.set_checked(&map, enabled);
                }
            }
            FilterMessage::MapFilterEnabled(enabled) => {
                self.filter.maps.enabled = enabled;
            }
            FilterMessage::ProviderChecked(provider, checked) => {
                self.filter.providers.dictionary.set_checked(&provider, checked);
            }
            FilterMessage::ProviderFilterEnabled(enabled) => {
                self.filter.providers.enabled = enabled;
            }
            FilterMessage::MapNameFilterChanged(text) => {
                self.filter.maps.text = text;
            }
        }
    }

    fn process_thumbnail_message(&mut self, message: ThumbnailMessage) {
        match message {
            ThumbnailMessage::Started(sender) => {
                self.map_thumbnail_request_sender = Some(sender);
                debug!("Thumbnail service started");
            }
            ThumbnailMessage::Thumbnail(map_name, thumbnail) => {
                self.thumbnail_ready(map_name, thumbnail);
            }
            ThumbnailMessage::Error(map_name, error) => {
                self.thumbnail_ready(map_name, None);
                error!(
                    "Thumbnail error: {}",
                    Self::remove_api_key(&self.user_settings.teamwork_api_key(), error)
                );
            }
        }
    }

    fn process_ping_message(&mut self, message: PingServiceMessage) {
        match message {
            PingServiceMessage::Started(sender) => {
                self.ping_request_sender = Some(sender);
                debug!("Ping service started");
            }
            PingServiceMessage::Answer(ip, duration) => {
                self.ping_found(ip, Some(duration));
            }
            PingServiceMessage::Error(ip, error) => {
                error!("Ping service error: {}", error);
                self.ping_found(ip, None);
            }
        }
    }

    fn process_country_message(&mut self, message: CountryServiceMessage) {
        match message {
            CountryServiceMessage::Started(country_sender) => {
                self.country_request_sender = Some(country_sender);
                debug!("country service started");
            }
            CountryServiceMessage::CountryFound(ip, country) => {
                self.country_found(ip, country);
            }
            CountryServiceMessage::Error(error) => {
                error!("Country service error: {}", error);
            }
        }
    }

    fn process_server_message(&mut self, message: FetchServersMessage) {
        match message {
            FetchServersMessage::FetchServersStart => {
                debug!("Start");
            }
            FetchServersMessage::FetchServersFinish => {
                debug!("Finish");
                self.on_finish();
            }
            FetchServersMessage::FetchServersError(error) => {
                error!(
                    "Error: {}",
                    Self::remove_api_key(&self.user_settings.teamwork_api_key(), &error.to_string())
                );
            }
            FetchServersMessage::NewServers(new_servers) => self.new_servers(new_servers),
        }
    }

    fn process_game_modes_message(&mut self, message: GameModesMessage) {
        match message {
            GameModesMessage::GameModes(game_modes) => {
                self.game_modes.reset(&game_modes);
                self.filter
                    .game_modes
                    .dictionary
                    .extend(game_modes.into_iter().map(|mode| GameModeId::new(mode.id)));
            }
            GameModesMessage::Error(error) => {
                self.push_notification(
                    "Failed to fetch game modes.\nFiltering by game modes is disabled.",
                    NotificationKind::Error,
                );
                error!(
                    "Failed to fetch game modes: {}",
                    Self::remove_api_key(&self.user_settings.teamwork_api_key(), error)
                );
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

    fn process_notification_message(&mut self, message: NotificationMessage) {
        match message {
            NotificationMessage::Update => {
                self.notifications.update(Instant::now());
            }
            NotificationMessage::Clear => {
                self.notifications.clear_current();
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
                    UrlWithKey::new(source.url(), &self.user_settings.teamwork_api_key()),
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

    fn launch_game(&mut self, ip_port: &IpPort) -> Command<Message> {
        if self.user_settings.steam_executable_path.trim().is_empty() {
            self.push_notification(
                "Steam executable not specified.\nSet the Steam executable in the settings.",
                NotificationKind::Error,
            );
        } else {
            match self.launcher.launch(&self.user_settings.steam_executable_path, ip_port) {
                Err(error) => {
                    self.push_notification(error, NotificationKind::Error);
                }
                Ok(()) => {
                    self.push_notification("Starting game!", NotificationKind::Feedback);
                    if self.user_settings.quit_on_launch {
                        return iced::window::close();
                    }
                }
            }
        }
        Command::none()
    }

    fn copy_connection_string(&mut self, ip_port: IpPort) -> Command<Message> {
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
            Self::increment_count(&mut count, value);

            count
        })
    }

    fn increment_count<K: Ord>(count: &mut BTreeMap<K, usize>, key: K) {
        match count.entry(key) {
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
            } else if server.has_random_crits {
                Self::increment_count(&mut count, Property::RandomCrits);
            }
        }

        count
    }

    fn push_notification(&mut self, text: impl ToString, kind: NotificationKind) {
        const NOTIFICATION_DURATION_SECS: u64 = 2;
        let text = Self::remove_api_key(&self.user_settings.teamwork_api_key(), text);
        let duration = match kind {
            NotificationKind::Error => None,
            NotificationKind::Feedback => Some(Duration::from_secs(NOTIFICATION_DURATION_SECS)),
        };
        self.notifications.push(Notification::new(text, duration, kind));
    }

    fn remove_api_key(key: &str, text: impl ToString) -> String {
        if key.is_empty() {
            return text.to_string();
        }

        text.to_string().replace(key, "****")
    }
}

impl Drop for TeamworkLauncher {
    fn drop(&mut self) {
        let configuration_directory = get_configuration_directory();

        if !configuration_directory.is_dir() {
            std::fs::create_dir_all(&configuration_directory).unwrap_or_else(|error| {
                error!(
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
            error!(
                "Failed to write bookmarks file '{}': {}",
                bookmarks_file_path.display(),
                error
            )
        });
        write_file(&self.user_settings, &settings_file_path)
            .unwrap_or_else(|error| error!("Failed to write settings file '{}': {}", settings_file_path.display(), error));
        write_file(&self.filter, &filters_file_path)
            .unwrap_or_else(|error| error!("Failed to write filters file '{}': {}", filters_file_path.display(), error));
        write_file(&self.servers_sources, &sources_file_path)
            .unwrap_or_else(|error| error!("Failed to write sources file '{}': {}", sources_file_path.display(), error));
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
            text: Color::from([0.9, 0.9, 0.9]),
            primary: Color::from_rgb8(57, 92, 120),
            success: Color::from_rgb8(75, 116, 28),
            danger: Color::from_rgb8(189, 59, 59),
        })
    }

    pub fn create_red_palette() -> theme::Custom {
        theme::Custom::new(theme::palette::Palette {
            background: Color::from_rgb8(38, 35, 33),
            text: Color::from([0.9, 0.9, 0.9]),
            primary: Color::from_rgb8(159, 49, 47),
            success: Color::from_rgb8(75, 116, 28),
            danger: Color::from_rgb8(189, 59, 59),
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
                process_detection: ProcessDetection::default(),
                game_modes: GameModes::new(),
                country_request_sender: None,
                ping_request_sender: None,
                map_thumbnail_request_sender: None,
                fetch_servers_subscription_id: 0,
                shift_pressed: false,
                theme,
                is_loading: false,
                notifications: Notifications::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Teamwork launcher")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::RefreshServers => self.refresh_servers(),
            Message::Servers(message) => {
                self.process_server_message(message);
            }
            Message::Country(message) => {
                self.process_country_message(message);
            }
            Message::Ping(message) => {
                self.process_ping_message(message);
            }
            Message::Thumbnail(message) => {
                self.process_thumbnail_message(message);
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
            Message::Notification(message) => {
                self.process_notification_message(message);
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
                self.push_notification("Copied to clipboard!", NotificationKind::Feedback);
                return iced::clipboard::write(text);
            }
            Message::Back => {
                self.views.pop();
            }
            Message::ShowSettings => {
                self.views.push(Screens::Settings);
            }
            Message::LaunchGame(ip_port) => {
                return if self.process_detection.is_game_detected() {
                    self.push_notification(
                        "The game is already started.\nConnection string copied to clipboard!",
                        NotificationKind::Feedback,
                    );
                    self.copy_connection_string(ip_port)
                } else {
                    self.launch_game(&ip_port)
                }
            }
            Message::CopyConnectionString(ip_port) => {
                self.push_notification("Copied to clipboard!", NotificationKind::Feedback);
                return self.copy_connection_string(ip_port);
            }
            Message::ShowServer(ip_port) => {
                self.views.push(Screens::Server(ip_port));
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message, Renderer<Self::Theme>> {
        let current = self.views.current().expect("valid view");

        container(column![
            ui::header::header_view("Teamwork Launcher", current, &self.notifications),
            match current {
                Screens::Main(view) => ui::main::view(
                    view,
                    &self.servers,
                    &self.bookmarks,
                    &self.filter,
                    &self.game_modes,
                    &self.servers_counts,
                    self.is_loading,
                ),
                Screens::Server(ip_port) => ui::server_details::view(&self.servers, &self.game_modes, ip_port),
                Screens::Settings => ui::settings::view(&self.user_settings, &self.servers_sources),
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
            thumbnail::subscription(self.fetch_servers_subscription_id, &self.user_settings.teamwork_api_key())
                .map(Message::from),
            game_mode::subscription(self.fetch_servers_subscription_id, &self.user_settings.teamwork_api_key())
                .map(Message::from),
            keyboard::subscription().map(Message::from),
            window::subscription(),
            self.notifications.subscription().map(Message::from),
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
