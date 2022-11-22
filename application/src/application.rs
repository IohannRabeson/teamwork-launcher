use std::{sync::Arc};

use iced::{
    widget::{column, vertical_space},
    Application as IcedApplication, Command, Element, Length, Theme,
};

use crate::states::{StatesStack, States};
use crate::views::error_view;
use crate::{
    icons::Icons,
    launcher::{LaunchParams, Launcher},
    servers::{self, Server, ServersProvider, SourceId},
    settings::UserSettings,
    setup::setup_launcher,
    views::{header_view, refreshing_view, servers_view, settings_view, edit_favorite_servers_view},
};

#[derive(Debug, Clone)]
pub enum Messages {
    RefreshServers,
    ServersRefreshed(Result<Vec<(Server, SourceId)>, servers::Error>),
    FilterChanged(String),
    StartGame(LaunchParams),
    /// Text passed as parameter will be copied to the clipboard.
    CopyToClipboard(String),
    /// The server is identified by its name.
    FavoriteClicked(String),
    EditFavorites,
    EditSettings,
    Back,
}


pub struct Application {
    settings: UserSettings,
    icons: Icons,
    servers_provider: Arc<ServersProvider>,
    servers: Vec<(Server, SourceId)>,
    states: StatesStack,
    launcher: Box<dyn Launcher>,
}

impl Application {
    fn refresh_command(&mut self) -> Command<Messages> {
        let servers_provider = self.servers_provider.clone();

        self.states.push(States::Reloading);
        self.servers.clear();

        Command::perform(async move { servers_provider.refresh().await }, Messages::ServersRefreshed)
    }

    fn favorite_servers_iter(&self) -> impl Iterator<Item = &(Server, SourceId)> {
        self.servers
            .iter()
            .filter(move |(server, _id)| self.filter_server(server))
    }

    fn filter_server(&self, server: &Server) -> bool {
        let text_filter = self.settings.filter.trim().to_lowercase();

        if !self.states.current_is(States::Favorites) && !self.settings.favorites.contains(&server.name) {
            return false;
        }

        if text_filter.is_empty() {
            return true;
        }

        server.name.as_str().to_lowercase().contains(&text_filter)
    }

    fn launch_executable(&mut self, params: &LaunchParams) {
        if let Err(error) = self.launcher.launch(params) {
            self.states.push(States::Error { message: error.message });
        }
    }

    fn switch_favorite_server(&mut self, server_name: &str) {
        match self.settings.favorites.contains(server_name) {
            true => self.settings.favorites.remove(server_name),
            false => self.settings.favorites.insert(server_name.to_string()),
        };
    }

    fn refresh_finished(&mut self, result: Result<Vec<(Server, SourceId)>, servers::Error>) {
        match result {
            Ok(servers) => {
                self.servers = servers;
                self.states.pop();
            },
            Err(error) => {
                self.states.reset(States::Normal);
                self.states.push(States::Error{ message: error.to_string() });
            },
        };
    }

    /// Display a content with a title and a header.
    fn normal_view<'a>(&self, content: Element<'a, Messages>) -> Element<'a, Messages> {
        column![
            header_view(&self.title(), &self.icons, self.states.current()),
            vertical_space(Length::Units(4)),
            content,
            // Elements after the content might be invisible if it is tall enough.
        ]
        .padding(12)
        .into()
    }
}

impl IcedApplication for Application {
    type Executor = iced::executor::Default;
    type Message = Messages;
    type Flags = UserSettings;
    type Theme = Theme;

    fn new(settings: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let theme = Theme::default();
        let servers_provider = Arc::new(ServersProvider::default());
        let mut launcher = Self {
            icons: Icons::new(&theme),
            servers_provider,
            servers: Vec::new(),
            settings,
            launcher: setup_launcher(),
            states: StatesStack::new(States::Normal),
        };
        let command = launcher.refresh_command();

        (launcher, command)
    }

    fn title(&self) -> String {
        "TF2 launcher".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Messages::ServersRefreshed(result) => self.refresh_finished(result),
            Messages::RefreshServers => return self.refresh_command(),
            Messages::FilterChanged(text_filter) => self.settings.filter = text_filter,
            Messages::StartGame(params) => self.launch_executable(&params),
            Messages::CopyToClipboard(text) => return iced::clipboard::write(text),
            Messages::FavoriteClicked(server_name) => self.switch_favorite_server(&server_name),
            Messages::EditFavorites => self.states.push(States::Favorites),
            Messages::EditSettings => self.states.push(States::Settings),
            Messages::Back => self.states.pop(),
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message, iced::Renderer<Self::Theme>> {
        match self.states.current() {
            States::Normal => self.normal_view(servers_view(self.favorite_servers_iter(), &self.icons, &self.settings, false)),
            States::Favorites => self.normal_view(edit_favorite_servers_view(self.servers.iter(), &self.icons, &self.settings)),
            States::Settings => self.normal_view(settings_view()),
            States::Reloading => self.normal_view(refreshing_view()),
            States::Error{ message } => self.normal_view(error_view(message)),
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        UserSettings::save_settings(&self.settings).expect("Write settings");
    }
}
