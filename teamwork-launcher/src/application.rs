use std::{cmp::Ordering, collections::BTreeSet, net::Ipv4Addr, sync::Arc};

use {
    iced::{
        widget::{column, image, vertical_space},
        Application as IcedApplication, Command, Element, Length, Theme,
    },
    itertools::Itertools,
    log::error,
};

use {enum_as_inner::EnumAsInner, iced::Subscription};

use log::info;

use crate::{
    geolocation::IpGeolocationService,
    icons::Icons,
    launcher::ExecutableLauncher,
    models::{Country, IpPort, Server, Thumbnail},
    servers_provider::{self, ServersProvider},
    settings::UserSettings,
    sources::SourceKey,
    states::StatesStack,
    ui::{
        error_view, header_view, no_favorite_servers_view, refresh_view, servers_view, servers_view_edit_favorites,
        settings_view,
    },
};

#[derive(Debug, Clone)]
pub enum Messages {
    RefreshServers,
    RefreshFavoriteServers,
    ServersRefreshed(Result<Vec<Server>, servers_provider::Error>),
    MapThumbnailReady(String, Thumbnail),
    CountryForIpReady(Ipv4Addr, Option<Country>),
    FilterChanged(String),
    StartGame(IpPort),
    /// Message produced when the settings are modified and saved.
    /// This message replace the current settings by the one passed as parameter.
    SettingsChanged(UserSettings),
    /// Text passed as parameter will be copied to the clipboard.
    CopyToClipboard(String),
    /// The server is identified by its name.
    FavoriteClicked(IpPort, Option<SourceKey>),
    /// Show the page to edit the favorite servers.
    EditFavorites,
    /// Show the page to edit the application settings.
    EditSettings,
    /// Pop the current state.
    Back,
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
    teamwork_client: teamwork::Client,
    servers_provider: Arc<ServersProvider>,
    servers: Vec<Server>,
    /// The stack managing the states.
    states: StatesStack<States>,
    launcher: ExecutableLauncher,
    theme: Theme,
    ip_geoloc_service: IpGeolocationService,
}

impl Application {
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
        self.make_refresh_command(None)
    }

    fn refresh_favorites_command(&mut self) -> Command<Messages> {
        self.make_refresh_command(Some(self.settings.favorite_source_keys()))
    }

    fn sort_servers_by_favorites<'r, 's>(left: &'r Server, right: &'s Server, settings: &UserSettings) -> Ordering {
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
    fn make_refresh_command(&mut self, source_keys: Option<BTreeSet<SourceKey>>) -> Command<Messages> {
        self.states.push(States::Reloading);
        self.servers.clear();

        let settings = self.settings.clone();
        let servers_provider = self.servers_provider.clone();

        Command::perform(
            async move {
                let servers = if source_keys.is_none() || source_keys.as_ref().unwrap().is_empty() {
                    servers_provider.refresh(&settings).await
                } else {
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
        self.servers.iter().filter(|server| self.filter_server_by_text(server))
    }

    /// Returns the favorites servers, filtered by text.
    fn favorite_servers_iter(&self) -> impl Iterator<Item = &Server> {
        self.servers.iter().filter(move |server| self.filter_favorite_server(server))
    }

    fn filter_server_by_text(&self, server: &Server) -> bool {
        self.settings.filter_servers_by_text(&server.name)
    }

    fn filter_favorite_server(&self, server: &Server) -> bool {
        self.settings.filter_servers_favorite(&server) && self.filter_server_by_text(server)
    }

    fn launch_executable(&mut self, ip_port: &IpPort) {
        if let Err(error) = self.launcher.launch(&self.settings.game_executable_path(), ip_port) {
            self.states.push(States::Error { message: error.message });
        }
    }

    fn switch_favorite_server(&mut self, ip_port: IpPort, source_key: Option<SourceKey>) {
        self.settings.switch_favorite_server(ip_port, source_key)
    }

    fn make_map_thumbnail_command(&self, server: &Server) -> Command<Messages> {
        let client = self.teamwork_client.clone();
        let map_name = server.map.clone();
        let api_key = self.settings.teamwork_api_key().clone();
        let thumbnail_ready_key = server.map.clone();

        Command::perform(
            async move {
                client
                    .get_map_thumbnail(&api_key, &map_name.clone(), image::Handle::from_memory)
                    .await
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

    fn refresh_finished(&mut self, result: Result<Vec<Server>, servers_provider::Error>) -> Command<Messages> {
        match result {
            Ok(servers) => {
                self.servers = servers;
                self.states.pop();

                info!("Fetched {} servers", self.servers.len());

                let servers_favorites_first: Vec<&Server> = self
                    .servers
                    .iter()
                    // Sort the servers favorites first to ensure the first servers visible have their thumbnail first.
                    .sorted_by(|left, right| Self::sort_servers_by_favorites(left, right, &self.settings))
                    .collect();
                let thumbnail_commands = servers_favorites_first
                    .iter()
                    // Sort the servers favorites first to ensure the first servers visible have their thumbnail first.
                    .unique_by(|server| &server.map)
                    .map(|server| self.make_map_thumbnail_command(server));
                let ip_geoloc_commands = servers_favorites_first
                    .iter()
                    .map(|server| server.ip_port.ip())
                    .unique()
                    .cloned()
                    .map(|ip| self.make_geolocalize_ip_command(ip));

                return Command::batch(thumbnail_commands.chain(ip_geoloc_commands));
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
    fn normal_view<'a>(&self, content: Element<'a, Messages>) -> Element<'a, Messages> {
        column![
            header_view(&self.title(), &self.icons, self.states.current()),
            vertical_space(Length::Units(4)),
            content,
            // Elements after the content might be invisible if it is tall enough.
            // There are no grid layout yet (see https://github.com/iced-rs/iced/issues/34).
        ]
        .padding(12)
        .into()
    }
}

pub struct Flags {
    pub settings: UserSettings,
    pub launcher: ExecutableLauncher,
}

impl IcedApplication for Application {
    type Executor = iced::executor::Default;
    type Message = Messages;
    type Flags = Flags;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let theme = Theme::default();
        let servers_provider = Arc::new(ServersProvider::default());
        let mut application = Self {
            icons: Icons::new(&theme),
            servers_provider,
            settings: flags.settings,
            launcher: flags.launcher,
            states: StatesStack::new(States::ShowServers),
            theme: Theme::Dark,
            servers: Vec::new(),
            teamwork_client: teamwork::Client::default(),
            ip_geoloc_service: IpGeolocationService::default(),
        };

        let command = match application.settings.has_favorites() {
            true => application.refresh_favorites_command(),
            false => application.refresh_command(),
        };

        (application, command)
    }

    fn title(&self) -> String {
        "Teamwork Launcher".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Messages::ServersRefreshed(result) => return self.refresh_finished(result),
            Messages::RefreshServers => return self.refresh_command(),
            Messages::RefreshFavoriteServers => return self.refresh_favorites_command(),
            Messages::FilterChanged(text_filter) => self.settings.set_filter_servers_text(text_filter),
            Messages::SettingsChanged(settings) => self.settings = settings,
            Messages::StartGame(params) => self.launch_executable(&params),
            Messages::CopyToClipboard(text) => return iced::clipboard::write(text),
            Messages::FavoriteClicked(server_ip_port, source_key) => self.switch_favorite_server(server_ip_port, source_key),
            Messages::EditFavorites => self.states.push(States::EditFavoriteServers),
            Messages::EditSettings => self.states.push(States::Settings),
            Messages::Back => self.states.pop(),
            Messages::MapThumbnailReady(map_name, image) => self.map_thumbnail_ready(&map_name, image),
            Messages::CountryForIpReady(ip, country) => self.country_for_ip_ready(ip, country),
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.states.current().is_normal() && self.settings.auto_refresh_favorite() {
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

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        self.settings.update_favorites(self.servers.iter());
        UserSettings::save_settings(&self.settings).expect("Write settings");
    }
}
