use std::{collections::BTreeSet, sync::Arc};

use iced::{
    widget::{column, vertical_space},
    Application as IcedApplication, Command, Element, Length, Theme,
};

use crate::{
    icons::Icons,
    launcher::ExecutableLauncher,
    models::{IpPort, Server},
    servers_provider::{self, ServersProvider},
    settings::UserSettings,
    sources::SourceKey,
    states::StatesStack,
    ui::{edit_favorite_servers_view, error_view, header_view, refresh_view, servers_view, settings_view},
};

#[derive(Debug, Clone)]
pub enum Messages {
    RefreshServers,
    RefreshFavoriteServers,
    ServersRefreshed(Result<Vec<Server>, servers_provider::Error>),
    FilterChanged(String),
    StartGame(IpPort),
    /// Message produced when the settings are modified and saved.
    /// This message replace the current settings by the one passed as parameter.
    ModifySettings(UserSettings),
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

#[derive(PartialEq, Eq)]
pub enum States {
    Normal,
    Favorites,
    Settings,
    Reloading,
    Error { message: String },
}

pub struct Application {
    settings: UserSettings,
    icons: Icons,
    servers_provider: Arc<ServersProvider>,
    servers: Vec<Server>,
    /// The stack managing the states.
    states: StatesStack<States>,
    launcher: ExecutableLauncher,
    theme: Theme,
}

impl Application {
    fn refresh_command(&mut self) -> Command<Messages> {
        self.make_refresh_command(None)
    }

    fn refresh_favorites_command(&mut self) -> Command<Messages> {
        self.make_refresh_command(Some(self.settings.favorite_source_keys()))
    }

    fn make_refresh_command(&mut self, source_keys: Option<BTreeSet<SourceKey>>) -> Command<Messages> {
        self.states.push(States::Reloading);
        self.servers.clear();

        let settings = self.settings.clone();
        let servers_provider = self.servers_provider.clone();

        Command::perform(
            async move {
                match source_keys {
                    Some(source_keys) => servers_provider.refresh_some(&settings, &source_keys).await,
                    None => servers_provider.refresh(&settings).await,
                }
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

    fn refresh_finished(&mut self, result: Result<Vec<Server>, servers_provider::Error>) {
        match result {
            Ok(servers) => {
                self.servers = servers;
                self.states.pop();
            }
            Err(error) => {
                self.states.reset(States::Normal);
                self.states.push(States::Error {
                    message: error.to_string(),
                });
            }
        };
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
            states: StatesStack::new(States::Normal),
            theme: Theme::Dark,
            servers: Vec::new(),
        };

        let command = match application.settings.has_favorites() {
            true => application.refresh_favorites_command(),
            false => application.refresh_command(),
        };

        (application, command)
    }

    fn title(&self) -> String {
        "TF2 launcher".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Messages::ServersRefreshed(result) => self.refresh_finished(result),
            Messages::RefreshServers => return self.refresh_command(),
            Messages::RefreshFavoriteServers => return self.refresh_favorites_command(),
            Messages::FilterChanged(text_filter) => self.settings.set_filter_servers_text(text_filter),
            Messages::StartGame(params) => self.launch_executable(&params),
            Messages::CopyToClipboard(text) => return iced::clipboard::write(text),
            Messages::FavoriteClicked(server_ip_port, source_key) => self.switch_favorite_server(server_ip_port, source_key),
            Messages::EditFavorites => self.states.push(States::Favorites),
            Messages::EditSettings => self.states.push(States::Settings),
            Messages::Back => self.states.pop(),
            Messages::ModifySettings(settings) => {
                self.settings = settings;
            }
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message, iced::Renderer<Self::Theme>> {
        self.normal_view(match self.states.current() {
            States::Normal => servers_view(self.favorite_servers_iter(), &self.icons, &self.settings, false),
            States::Favorites => edit_favorite_servers_view(self.servers_iter(), &self.icons, &self.settings),
            States::Settings => settings_view(&self.settings),
            States::Reloading => refresh_view(),
            States::Error { message } => error_view(message),
        })
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
