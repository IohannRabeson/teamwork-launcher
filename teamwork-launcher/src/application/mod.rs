mod bookmarks;
pub mod country;
pub mod fetch_servers;
pub mod filter;
pub mod game_mode;
mod geolocation;
pub mod ip_port;
mod launcher;
pub mod map;
pub mod message;
mod mods_management;
pub mod notifications;
pub mod palettes;
pub mod paths;
mod ping;
mod process_detection;
pub mod progress;
pub mod promised_value;
pub mod screens;
pub mod screenshots;
pub mod server;
pub mod servers_counts;
pub mod servers_source;
mod thumbnail;
pub mod user_settings;

use {
    crate::ui::{self, main::ViewContext},
    iced::{
        futures::{channel::mpsc::UnboundedSender, FutureExt, SinkExt, TryFutureExt},
        subscription, theme,
        widget::{column, container, image, pane_grid, scrollable},
        Command, Element, Renderer, Subscription, Theme,
    },
    iced_native::widget::pane_grid::Axis,
    iced_views::Views,
    itertools::Itertools,
    log::{debug, error, trace},
    std::{
        collections::{
            btree_map::Entry::{Occupied, Vacant},
            BTreeMap, BTreeSet,
        },
        net::Ipv4Addr,
        sync::Arc,
        time::{Duration, Instant},
    },
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
            message::{KeyboardMessage, NotificationMessage, ScreenshotsMessage},
            notifications::{Notification, NotificationKind, Notifications},
            paths::PathsProvider,
            process_detection::ProcessDetection,
            progress::Progress,
            screens::{PaneId, PaneView},
            screenshots::Screenshots,
            servers_source::{ServersSource, SourceKey},
            thumbnail::ThumbnailCache,
        },
        common_settings::{write_bin_file, write_file},
        ui::{main::ServersList, styles::MainBackground},
        ApplicationFlags,
    },
    mods_manager::{ModName, Registry},
    screens::{Screens, ServerView},
    server::Property,
    servers_counts::ServersCounts,
};

#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    #[error("JSON error: {0}")]
    Json(#[from] Arc<serde_json::Error>),
    #[error("IO error: {0}")]
    Io(#[from] Arc<std::io::Error>),
    #[error("Invalid file format")]
    InvalidFileFormat,
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
    notifications: Notifications,
    screenshots: Screenshots,
    servers_list: ServersList,
    mods_registry: Registry,
    selected_mod: Option<ModName>,
    paths: Box<dyn PathsProvider>,
    testing_mode_enabled: bool,

    country_request_sender: Option<UnboundedSender<Ipv4Addr>>,
    ping_request_sender: Option<UnboundedSender<Ipv4Addr>>,
    map_thumbnail_request_sender: Option<UnboundedSender<MapName>>,
    thumbnails_cache: ThumbnailCache,
    progress: Progress,

    fetch_servers_subscription_id: u64,
    shift_pressed: bool,
    theme: Theme,
    is_loading_servers: bool,
    is_loading_mods: bool,

    panes: pane_grid::State<PaneView>,
    panes_split: pane_grid::Split,
    server_view_compact_mode: bool,
}

impl iced::Application for TeamworkLauncher {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ApplicationFlags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let theme: Theme = flags.user_settings.theme.into();
        let thumbnail_cache_directory = flags.paths.get_thumbnails_directory();
        let mut thumbnails_cache = ThumbnailCache::new(thumbnail_cache_directory);
        let mut notifications = Notifications::new();
        let mods_directory = flags.paths.get_mods_directory();
        if let Err(error) = thumbnails_cache.load() {
            error!("Failed to load thumbnails cache: {}", error);
        }

        if !flags.user_settings.has_teamwork_api_key() {
            notifications.push(Notification::new(
                "No Teamwork.tf API key specified.\nSet your API key in the settings.",
                None,
                NotificationKind::Error,
            ));
        }

        let (mut panes, servers_pane) = pane_grid::State::new(PaneView::new(PaneId::Servers));
        let (_filter_pane, panes_split) = panes
            .split(Axis::Vertical, &servers_pane, PaneView::new(PaneId::Filters))
            .expect("split pane vertically");

        panes.resize(&panes_split, flags.user_settings.servers_filter_pane_ratio);

        (
            Self {
                views: Views::new(Screens::Main),
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
                is_loading_servers: false,
                notifications,
                screenshots: Screenshots::new(),
                servers_list: ServersList::new(),
                thumbnails_cache,
                progress: Progress::default(),
                paths: flags.paths,
                testing_mode_enabled: flags.testing_mode_enabled,
                mods_registry: Registry::new(),
                selected_mod: None,
                is_loading_mods: false,
                panes,
                panes_split,
                server_view_compact_mode: false,
            },
            mods_management::commands::scan_mods_directory(mods_directory),
        )
    }

    fn title(&self) -> String {
        let mut title = String::from("Teamwork launcher");

        if self.testing_mode_enabled {
            title.push_str(" - TESTING MODE");
        }

        title
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::RefreshServers => self.refresh_servers(),
            Message::Servers(message) => {
                self.process_server_message(message);
            }
            Message::Mods(message) => {
                return self.process_mods_message(message);
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
            Message::Screenshots(message) => {
                self.process_screenshots_message(message);
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

                // This is the case where the user has just pasted his API key.
                // Instead of waiting for the user, we refresh spontaneously.
                if !self.is_loading_servers && self.servers.is_empty() && self.user_settings.has_teamwork_api_key() {
                    self.refresh_servers();
                }

                return scrollable::snap_to(self.servers_list.id.clone(), self.servers_list.scroll_position);
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
            Message::ShowServer(ip_port, map_name) => {
                self.views.push(Screens::Server(ServerView::new(ip_port)));
                self.screenshots.set(PromisedValue::Loading);
                return screenshots::fetch_screenshot(map_name, self.user_settings.teamwork_api_key());
            }
            Message::ServerListScroll(position) => {
                self.servers_list.scroll_position = position;
            }
            Message::ShowMods => {
                self.views.push(Screens::Mods);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message, Renderer<Self::Theme>> {
        let current = self.views.current().expect("valid view");

        container(column![
            ui::header::header_view("Teamwork Launcher", current, &self.notifications),
            match current {
                Screens::Main => {
                    ui::main::view(ViewContext {
                        panes_split: &self.panes_split,
                        panes: &self.panes,
                        servers: &self.servers,
                        bookmarks: &self.bookmarks,
                        filter: &self.filter,
                        game_modes: &self.game_modes,
                        counts: &self.servers_counts,
                        servers_list: &self.servers_list,
                        progress: &self.progress,
                        is_loading: self.is_loading_servers,
                        compact_mode: self.server_view_compact_mode,
                    })
                }
                Screens::Server(view) => {
                    ui::server_details::view(&self.servers, &self.game_modes, &view.ip_port, &self.screenshots)
                }
                Screens::Settings => {
                    ui::settings::view(
                        &self.user_settings,
                        &self.servers_sources,
                        self.paths.get_configuration_directory(),
                    )
                }
                Screens::Mods => {
                    ui::mods_view::view(&self.mods_registry, self.selected_mod.as_ref(), self.is_loading_mods)
                }
                Screens::AddMod(context) => {
                    ui::add_mod_view::view(context)
                }
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

impl TeamworkLauncher {
    fn new_servers(&mut self, new_servers: Vec<Server>) {
        let countries = new_servers.iter().filter_map(|server| server.country.get()).unique().cloned();

        self.filter.country.dictionary.extend(countries);

        self.servers.extend(new_servers.into_iter());
        self.sort_server();
    }

    fn on_finish(&mut self) {
        self.is_loading_servers = false;

        self.sort_server();

        for server in self.servers.iter_mut() {
            if let Some(image) = self.thumbnails_cache.get(&server.map) {
                trace!("Image for {} fetch from cache", &server.map);
                server.map_thumbnail = PromisedValue::Ready(image);
            }
        }

        let mut servers_refs: Vec<&Server> = self.servers.iter().collect();

        servers_refs.sort_by(|l, r| {
            let left = self.bookmarks.is_bookmarked(&l.ip_port);
            let right = self.bookmarks.is_bookmarked(&r.ip_port);

            right.cmp(&left)
        });

        // For each map, request the thumbnail
        let mut unique_map_names: BTreeSet<MapName> = BTreeSet::new();

        for server in servers_refs.iter() {
            if unique_map_names.contains(&server.map) || server.map_thumbnail.is_ready() {
                continue;
            }

            unique_map_names.insert(server.map.clone());

            if let Some(thumbnail_sender) = &mut self.map_thumbnail_request_sender {
                thumbnail_sender
                    .send(server.map.clone())
                    .unwrap_or_else(|e| error!("thumbnail sender {}", e))
                    .now_or_never();

                self.progress.increment_total();
            }
        }

        // Request country for each server
        for ip in servers_refs.iter().map(|server| server.ip_port.ip()).unique() {
            if let Some(country_sender) = &mut self.country_request_sender {
                country_sender
                    .send(*ip)
                    .unwrap_or_else(|e| error!("country sender {}", e))
                    .now_or_never();

                self.progress.increment_total();
            }

            if let Some(ping_sender) = &mut self.ping_request_sender {
                ping_sender
                    .send(*ip)
                    .unwrap_or_else(|e| error!("ping sender {}", e))
                    .now_or_never();

                self.progress.increment_total();
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

        for map_name in self.servers.iter().map(|server| &server.map) {
            self.filter.maps.dictionary.add(map_name.clone());
        }
    }

    fn refresh_servers(&mut self) {
        if !self.user_settings.has_teamwork_api_key() {
            self.push_notification(
                "No Teamwork.tf API key specified.\nSet your API key in the settings.",
                NotificationKind::Error,
            );
        } else {
            self.is_loading_servers = true;
            self.progress.reset();
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

        self.progress.increment_current();
    }

    fn ping_found(&mut self, ip: Ipv4Addr, duration: Option<Duration>) {
        for server in self.servers.iter_mut().filter(|server| server.ip_port.ip() == &ip) {
            if duration.is_none() {
                self.servers_counts.timeouts += 1;
            }

            server.ping = duration.into();
        }

        self.sort_server();

        self.progress.increment_current();
    }

    fn thumbnail_ready(&mut self, map_name: MapName, thumbnail: Option<image::Handle>) {
        if let Some(image) = thumbnail.as_ref() {
            self.thumbnails_cache.insert(map_name.clone(), image.clone());
        }

        for server in self.servers.iter_mut().filter(|server| server.map == map_name) {
            if !server.map_thumbnail.is_ready() {
                server.map_thumbnail = thumbnail.clone().into();
            }
        }

        self.progress.increment_current();
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

    fn require_compact_mode(&self, ratio: f32) -> bool {
        match self.user_settings.window.as_ref() {
            None => false,
            Some(window) => {
                const MIN_RIGHT_PANE_WIDTH: f32 = 600.0;
                let min_ratio = MIN_RIGHT_PANE_WIDTH / window.window_width as f32;

                ratio > min_ratio
            }
        }
    }

    fn constraint_pane_ratio(&self, ratio: f32) -> f32 {
        match self.user_settings.window.as_ref() {
            None => ratio,
            Some(window) => {
                const MAX_LEFT_PANE_WIDTH: f32 = 341.0;
                let max_ratio = (window.window_width as f32 - MAX_LEFT_PANE_WIDTH) / window.window_width as f32;

                if ratio > max_ratio {
                    max_ratio
                } else {
                    ratio
                }
            }
        }
    }

    fn process_pane_message(&mut self, message: PaneMessage) {
        match message {
            PaneMessage::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.user_settings.servers_filter_pane_ratio = self.constraint_pane_ratio(ratio);
                self.panes.resize(&split, self.user_settings.servers_filter_pane_ratio);
                self.server_view_compact_mode = self.require_compact_mode(ratio);
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

                    self.user_settings.servers_filter_pane_ratio =
                        self.constraint_pane_ratio(self.user_settings.servers_filter_pane_ratio);
                    self.panes
                        .resize(&self.panes_split, self.user_settings.servers_filter_pane_ratio);
                    self.server_view_compact_mode = self.require_compact_mode(self.user_settings.servers_filter_pane_ratio);
                }
            }
            SettingsMessage::ThemeChanged(theme) => {
                self.user_settings.theme = theme;
                self.theme = theme.into();
            }
            SettingsMessage::OpenDirectory(directory) => {
                if directory.is_dir() {
                    if let Err(error) = open::that(&directory) {
                        self.push_notification(
                            format!("Failed to open configuration directory:\n{}", error),
                            NotificationKind::Error,
                        );
                    }
                }
            }
            SettingsMessage::MaxCacheSizeChanged(value) => {
                self.user_settings.max_thumbnails_cache_size_mb = value;
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
                self.filter.ping.max_ping = max_ping;
            }
            FilterMessage::AcceptPingTimeoutChanged(checked) => {
                self.filter.ping.accept_ping_timeout = checked;
            }
            FilterMessage::PingFilterEnabled(enabled) => {
                self.filter.ping.enabled = enabled;
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
            FilterMessage::PlayerFilterEnabled(enabled) => {
                self.filter.players.enabled = enabled;
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
                    Self::obfuscate_api_key(&self.user_settings.teamwork_api_key(), error)
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
                    Self::obfuscate_api_key(&self.user_settings.teamwork_api_key(), error)
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
                    Self::obfuscate_api_key(&self.user_settings.teamwork_api_key(), error)
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

    fn process_screenshots_message(&mut self, message: ScreenshotsMessage) {
        match message {
            ScreenshotsMessage::Screenshots(screenshots) => {
                self.screenshots.set(PromisedValue::Ready(screenshots));
            }
            ScreenshotsMessage::Error(error) => {
                error!(
                    "Screenshots fetch failed: {}",
                    Self::obfuscate_api_key(&self.user_settings.teamwork_api_key(), error)
                );
            }
            ScreenshotsMessage::Next => {
                self.screenshots.next();
            }
            ScreenshotsMessage::Previous => {
                self.screenshots.previous();
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
        let text = Self::obfuscate_api_key(&self.user_settings.teamwork_api_key(), text);
        let duration = match kind {
            NotificationKind::Error => None,
            NotificationKind::Feedback => Some(Duration::from_secs(NOTIFICATION_DURATION_SECS)),
        };
        self.notifications.push(Notification::new(text, duration, kind));
    }

    fn obfuscate_api_key(key: &str, text: impl ToString) -> String {
        if key.is_empty() {
            return text.to_string();
        }

        text.to_string().replace(key, "****")
    }
}

impl Drop for TeamworkLauncher {
    fn drop(&mut self) {
        let configuration_directory = self.paths.get_configuration_directory();

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
        let mods_registry_file_path = configuration_directory.join("mods.registry");

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

        if let Err(error) = self
            .thumbnails_cache
            .write(self.user_settings.max_thumbnails_cache_size_mb * 1024 * 1024)
        {
            error!("Failed to write thumbnails cache: {}", error);
        }

        write_bin_file(&self.mods_registry, &mods_registry_file_path).unwrap_or_else(|error| {
            error!(
                "Failed to write mods registry file '{}': {}",
                mods_registry_file_path.display(),
                error
            )
        });
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
