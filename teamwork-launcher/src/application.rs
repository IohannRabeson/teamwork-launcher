use std::path::PathBuf;

use crate::{APPLICATION_VERSION, GIT_SHA_SHORT};

use {
    crate::{
        announces::{Announce, AnnounceQueue},
        geolocation::IpGeolocationService,
        icons::Icons,
        launcher::ExecutableLauncher,
        models::{Country, IpPort, Server, Thumbnail},
        ping_service::PingService,
        servers_provider::{self, ServersProvider},
        settings::UserSettings,
        sources::SourceKey,
        states::StatesStack,
        ui::{
            announce_view, error_view, header_view, no_favorite_servers_view, refresh_view, servers_view,
            servers_view_edit_favorites, settings_view, VISUAL_SPACING_MEDIUM, VISUAL_SPACING_SMALL,
        },
        CliParameters,
    },
    enum_as_inner::EnumAsInner,
    iced::{
        widget::{column, image, vertical_space},
        Application as IcedApplication, Command, Element, Length, Subscription, Theme,
    },
    itertools::Itertools,
    log::{debug, error, info},
    std::{cmp::Ordering, collections::BTreeSet, iter, net::Ipv4Addr, sync::Arc, time::Duration},
};

#[derive(Debug, Clone)]
pub enum Messages {
    RefreshServers,
    RefreshFavoriteServers,
    ServersRefreshed(Result<Vec<Server>, servers_provider::Error>),
    MapThumbnailReady(String, Thumbnail),
    CountryForIpReady(Ipv4Addr, Option<Country>),
    PingReady(Ipv4Addr, Option<Duration>),
    FilterChanged(String),
    StartGame(IpPort),
    /// Message produced when the settings are modified and saved.
    /// This message replace the current settings by the one passed as parameter.
    SettingsChanged(UserSettings),
    /// Text passed as parameter will be copied to the clipboard.
    CopyToClipboard(String),

    FavoriteClicked(IpPort, Option<SourceKey>),

    SourceFilterClicked(SourceKey, bool),

    /// Show the page to edit the favorite servers.
    EditFavorites,
    /// Show the page to edit the application settings.
    EditSettings,
    /// Pop the current state.
    Back,
    /// Pop all the state then quit the application.
    Quit,

    PushAnnounce(Announce),

    /// Discard the current announce
    DiscardCurrentAnnounce,

    OpenConfigurationDirectory(PathBuf),
}

pub struct Flags {
    pub cli_params: CliParameters,
    pub settings: UserSettings,
    pub launcher: ExecutableLauncher,
}

#[derive(PartialEq, Eq, EnumAsInner)]
pub enum States {
    ShowServers,
    EditFavoriteServers,
    Settings,
    Reloading,
    Error { message: String },
}

pub struct Application {
    settings: UserSettings,
    icons: Icons,
    servers: Vec<Server>,
    /// The stack managing the states.
    states: StatesStack<States>,
    /// The queue of announces.
    /// The user have to click on each announce, once an announce is closed
    /// the next one appears.
    announces: AnnounceQueue,
    theme: Theme,

    launcher: ExecutableLauncher,
    teamwork_client: teamwork::Client,
    servers_provider: Arc<ServersProvider>,
    ip_geoloc_service: IpGeolocationService,
    ping_service: PingService,
    should_exit: bool,
}

impl Drop for Application {
    fn drop(&mut self) {
        self.settings.update_favorites(self.servers.iter());
        UserSettings::save_settings(&self.settings).expect("Write settings");
    }
}

impl IcedApplication for Application {
    type Executor = iced::executor::Default;
    type Message = Messages;
    type Flags = Flags;
    type Theme = Theme;

    fn new(mut flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let theme = Theme::default();
        let servers_provider = Arc::new(ServersProvider::default());

        flags.settings.set_available_sources(servers_provider.get_sources());

        let mut application = Self {
            should_exit: false,
            icons: Icons::new(&theme),
            servers_provider,
            settings: flags.settings,
            launcher: flags.launcher,
            states: StatesStack::new(States::ShowServers),
            theme: Theme::Dark,
            servers: Vec::new(),
            teamwork_client: teamwork::Client::default(),
            ip_geoloc_service: IpGeolocationService::default(),
            ping_service: PingService::default(),
            announces: AnnounceQueue::default(),
        };

        let mut command = application.refresh_command();

        if flags.cli_params.integration_test {
            // The integration test is basic, it runs the application for 5 seconds.
            command = Command::batch(iter::once(command).chain(iter::once(Command::perform(
                async { async_std::task::sleep(Duration::from_secs(5)).await },
                |_| Messages::Quit,
            ))));

            application.announces.push(Announce::new(
                "Integration test mode",
                "The application run in integration test mode. The application will close itself after 5 seconds",
            ));
        }

        if application.settings.teamwork_api_key().trim().is_empty() {
            application.announces.push(Announce::new(
                "No Teamwork.tf API key",
                "This application needs a Teamwork.tf API key to fetch all the information.\nTo get an API key, please login in teamwork.tf then go to https://teamwork.tf/settings."));
        }

        if !application.ping_service.is_enabled() {
            application.announces.push(Announce::new(
                "Ping service requires permission",
                "This application needs to be run elevated to be able to query the ping.",
            ));
        }

        info!("Version: {}-{}", APPLICATION_VERSION, GIT_SHA_SHORT);

        (application, command)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Messages::ServersRefreshed(result) => return self.refresh_finished(result),
            Messages::RefreshServers => return self.refresh_command(),
            Messages::RefreshFavoriteServers => return self.refresh_favorites_command(),
            Messages::FilterChanged(text_filter) => self.settings.set_filter_servers_text(text_filter),
            Messages::SettingsChanged(settings) => self.settings = settings,
            Messages::StartGame(params) => self.launch_executable(&params),
            Messages::CopyToClipboard(text) => return self.copy_to_clipboard_command(text),
            Messages::FavoriteClicked(server_ip_port, source_key) => self.switch_favorite_server(server_ip_port, source_key),
            Messages::SourceFilterClicked(source_key, checked) => self.source_filter_clicked(&source_key, checked),
            Messages::EditFavorites => self.states.push(States::EditFavoriteServers),
            Messages::EditSettings => self.states.push(States::Settings),
            Messages::Back => self.states.pop(),
            Messages::MapThumbnailReady(map_name, image) => self.map_thumbnail_ready(&map_name, image),
            Messages::CountryForIpReady(ip, country) => self.country_for_ip_ready(ip, country),
            Messages::PingReady(ip, duration) => self.ping_ready(ip, duration),
            Messages::Quit => self.should_exit = true,
            Messages::DiscardCurrentAnnounce => self.announces.pop(),
            Messages::PushAnnounce(announce) => self.announces.push(announce),
            Messages::OpenConfigurationDirectory(directory_path) => return self.explore_directory(directory_path),
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.states.current().is_show_servers() && self.settings.auto_refresh_favorite() {
            // Each 5 minutes refresh the favorites servers
            return iced::time::every(std::time::Duration::from_secs(60 * 5)).map(|_| Messages::RefreshFavoriteServers);
        }

        Subscription::none()
    }

    fn view(&self) -> iced::Element<Self::Message, iced::Renderer<Self::Theme>> {
        let content = match self.states.current() {
            States::ShowServers if self.servers.is_empty() => no_favorite_servers_view(),
            States::ShowServers => servers_view(self.favorite_servers_iter(), &self.icons, &self.settings),
            States::EditFavoriteServers => servers_view_edit_favorites(self.servers_iter(), &self.icons, &self.settings),
            States::Settings => settings_view(&self.settings),
            States::Reloading => refresh_view(),
            States::Error { message } => error_view(message),
        };

        self.normal_view(content)
    }

    fn title(&self) -> String {
        "Teamwork Launcher".to_string()
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

impl Application {
    fn ping_ready(&mut self, ip: Ipv4Addr, duration: Option<Duration>) {
        for server in &mut self.servers.iter_mut().filter(|server| server.ip_port.ip() == &ip) {
            server.ping = duration.into();
        }
    }

    fn country_for_ip_ready(&mut self, ip: Ipv4Addr, country: Option<Country>) {
        for server in &mut self.servers.iter_mut().filter(|server| server.ip_port.ip() == &ip) {
            server.country = country.clone().into();
        }
    }

    fn map_thumbnail_ready(&mut self, map_name: &str, thumbnail: Thumbnail) {
        for server in &mut self.servers.iter_mut().filter(|server| server.map == map_name) {
            server.map_thumbnail = thumbnail.clone();
        }
    }

    fn refresh_command(&mut self) -> Command<Messages> {
        self.make_refresh_command(None, true)
    }

    fn refresh_favorites_command(&mut self) -> Command<Messages> {
        self.make_refresh_command(Some(self.settings.favorite_source_keys()), false)
    }

    fn sort_servers_by_favorites(left: &Server, right: &Server, settings: &UserSettings) -> Ordering {
        let left = settings.filter_servers_favorite(left);
        let right = settings.filter_servers_favorite(right);

        if left == right {
            Ordering::Equal
        } else if left {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }

    fn make_refresh_command(&mut self, source_keys: Option<BTreeSet<SourceKey>>, clear: bool) -> Command<Messages> {
        if clear {
            self.states.push(States::Reloading);
            self.servers.clear();
        }

        let settings = self.settings.clone();
        let servers_provider = self.servers_provider.clone();

        Command::perform(
            async move {
                let servers = if source_keys.is_none() || source_keys.as_ref().unwrap().is_empty() {
                    debug!("Refresh all");
                    servers_provider
                        .refresh_some(&settings, &settings.checked_source_keys())
                        .await
                } else {
                    debug!("Refresh some: {:?}", source_keys);
                    servers_provider.refresh_some(&settings, &source_keys.unwrap()).await
                };

                // By default servers are sorted by name
                servers.map(|mut servers| {
                    servers.sort_by(|left, right| left.name.cmp(&right.name));
                    servers
                })
            },
            Messages::ServersRefreshed,
        )
    }

    /// Returns the servers filtered by text.
    fn servers_iter(&self) -> impl Iterator<Item = &Server> {
        self.servers.iter().filter(|server| self.filter_server(server))
    }

    /// Returns the favorites servers, filtered by text.
    fn favorite_servers_iter(&self) -> impl Iterator<Item = &Server> {
        self.servers.iter().filter(move |server| self.filter_favorite_server(server))
    }

    /// Filter server using the settings
    fn filter_server(&self, server: &Server) -> bool {
        self.settings.filter_servers(server)
    }

    fn filter_favorite_server(&self, server: &Server) -> bool {
        self.settings.filter_servers_favorite(server) && self.filter_server(server)
    }

    fn launch_executable(&mut self, ip_port: &IpPort) {
        if let Err(error) = self.launcher.launch(&self.settings.game_executable_path(), ip_port) {
            self.states.push(States::Error { message: error.message });
        } else if self.settings.quit_on_launch() {
            self.should_exit = true;
        }
    }

    fn copy_to_clipboard_command(&mut self, text: String) -> Command<Messages> {
        if self.settings.quit_on_copy() {
            Command::batch(vec![
                iced::clipboard::write(text),
                Command::perform(async move { () }, |_| Messages::Quit),
            ])
        } else {
            iced::clipboard::write(text)
        }
    }

    fn switch_favorite_server(&mut self, ip_port: IpPort, source_key: Option<SourceKey>) {
        self.settings.switch_favorite_server(ip_port, source_key)
    }

    fn source_filter_clicked(&mut self, source_key: &SourceKey, checked: bool) {
        self.settings.check_source_filter(source_key, checked);
    }

    fn make_map_thumbnail_command(&self, server: &Server) -> Command<Messages> {
        let client = self.teamwork_client.clone();
        let map_name = server.map.clone();
        let api_key = self.settings.teamwork_api_key();
        let thumbnail_ready_key = server.map.clone();

        Command::perform(
            async move {
                const MAX_RETRIES: usize = 3;
                let mut counter: usize = 0;

                loop {
                    let result = client
                        .get_map_thumbnail(&api_key, &map_name.clone(), image::Handle::from_memory)
                        .await;

                    counter += 1;

                    if result.is_ok() || counter >= MAX_RETRIES {
                        return result;
                    }

                    if let Err(error) = result.as_ref() {
                        if error.as_http_request().is_none() && error.as_teamwork_error().is_none() {
                            return result;
                        }
                        info!("Retrying to get thumbnail after a pause: {}", counter);
                        async_std::task::sleep(Duration::from_millis(1000)).await
                    }
                }
            },
            |result| match result {
                Ok(image) => Messages::MapThumbnailReady(thumbnail_ready_key, Thumbnail::Ready(image)),
                Err(error) => {
                    error!("Error while fetching thumbnail for map '{}': {}", thumbnail_ready_key, error);
                    Messages::MapThumbnailReady(thumbnail_ready_key, Thumbnail::None)
                }
            },
        )
    }

    fn make_geolocalize_ip_command(&self, ip: Ipv4Addr) -> Command<Messages> {
        let geolocalization_service = self.ip_geoloc_service.clone();

        Command::perform(
            async move {
                match geolocalization_service.locate(ip).await {
                    Ok(country) => Some(country),
                    Err(error) => {
                        error!("{}", error);
                        None
                    }
                }
            },
            move |country| Messages::CountryForIpReady(ip, country),
        )
    }

    fn make_ping_ip_command(&self, ip: &Ipv4Addr) -> Command<Messages> {
        let ping_service = self.ping_service.clone();
        let ip = *ip;

        Command::perform(
            async move {
                match ping_service.ping(&ip).await {
                    Ok(country) => Some(country),
                    Err(_error) => None,
                }
            },
            move |duration| Messages::PingReady(ip, duration),
        )
    }

    fn refresh_finished(&mut self, result: Result<Vec<Server>, servers_provider::Error>) -> Command<Messages> {
        match result {
            Ok(servers) => {
                self.servers = servers;

                if self.states.current().is_reloading() {
                    self.states.pop();
                }

                info!("Fetched {} servers", self.servers.len());

                let servers_favorites_first: Vec<&Server> = self
                    .servers
                    .iter()
                    // Sort the servers favorites first to ensure the first servers visible have their thumbnail first.
                    .sorted_by(|left, right| Self::sort_servers_by_favorites(left, right, &self.settings))
                    .collect();
                let thumbnail_commands = servers_favorites_first
                    .iter()
                    .unique_by(|server| &server.map)
                    .map(|server| self.make_map_thumbnail_command(server));
                let ip_geoloc_commands = servers_favorites_first
                    .iter()
                    .map(|server| server.ip_port.ip())
                    .unique()
                    .cloned()
                    .map(|ip| self.make_geolocalize_ip_command(ip));
                let ip_ping_commands = self
                    .servers
                    .iter()
                    .map(|server| server.ip_port.ip())
                    .unique()
                    .map(|ip| self.make_ping_ip_command(ip));

                return Command::batch(thumbnail_commands.chain(ip_geoloc_commands).chain(ip_ping_commands));
            }
            Err(error) => {
                self.states.reset(States::ShowServers);
                self.states.push(States::Error {
                    message: error.to_string(),
                });
            }
        };

        Command::none()
    }

    /// Display a content with a title and a header.
    fn normal_view<'a>(&'a self, content: Element<'a, Messages>) -> Element<'a, Messages> {
        let mut main_column = column![header_view(&self.title(), &self.icons, self.states.current())];

        if let Some(announce) = self.announces.current() {
            main_column = main_column
                .push(vertical_space(Length::Units(VISUAL_SPACING_SMALL)))
                .push(announce_view(&self.icons, announce));
        }

        main_column
            .push(vertical_space(Length::Units(VISUAL_SPACING_MEDIUM)))
            .push(content)
            .padding(12)
            .into()
    }

    fn explore_directory(&self, file_to_edit: PathBuf) -> Command<Messages> {
        let file_to_edit_for_error = file_to_edit.clone();

        Command::perform(
            async move { Self::open_directory(file_to_edit).await },
            move |result| match result {
                Ok(_) => Messages::PushAnnounce(Announce::new(
                    "Application restart needed",
                    "The application must restart to reload the edited configuration file.",
                )),
                Err(error) => Messages::PushAnnounce(Announce::new(
                    format!(
                        "Can't edit '{}'.",
                        file_to_edit_for_error
                            .file_name()
                            .map(|s| s.to_string_lossy())
                            .unwrap_or_default()
                    ),
                    format!(
                        "The file '{}' can't be edited:\n - {}",
                        file_to_edit_for_error.display(),
                        error
                    ),
                )),
            },
        )
    }

    #[cfg(target_os = "windows")]
    async fn open_directory(file_to_edit: PathBuf) -> Result<(), String> {
        tokio::task::spawn_blocking(|| {
            use std::process::Command;

            Command::new("explorer.exe")
                .args(vec![format!("/e,{}", file_to_edit.to_string())])
                .output()
                .map_err(|error| error.to_string())?;

            Ok(())
        })
        .await
        .unwrap()
    }

    #[cfg(target_os = "macos")]
    async fn open_directory(file_to_edit: PathBuf) -> Result<(), String> {
        tokio::task::spawn_blocking(move || {
            use std::process::Command;

            Command::new("open")
                .args(vec![file_to_edit.to_string_lossy().to_string()])
                .output()
                .map_err(|error| error.to_string())?;

            Ok(())
        })
        .await
        .unwrap()
    }
}
