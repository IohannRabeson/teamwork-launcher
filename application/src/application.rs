use std::sync::Arc;

use iced::{widget::column, widget::vertical_space, Application as IcedApplication, Command, Element, Length, Theme};

use crate::{
    icons::Icons,
    launcher::{LaunchParams, Launcher},
    servers::{self, Server, ServersProvider, SourceId},
    settings::UserSettings,
    setup::setup_launcher,
    views::{filter_view, header_view, refreshing_view, servers_view, settings_view},
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
    EditFavorites(bool),
    EditSettings(bool),
}

pub struct Application {
    settings: UserSettings,
    icons: Icons,
    servers_provider: Arc<ServersProvider>,
    servers: Vec<(Server, SourceId)>,
    edit_favorites: bool,
    edit_settings: bool,
    reloading: bool,
    launcher: Box<dyn Launcher>,
}

impl Application {
    fn refresh_command(&mut self) -> Command<Messages> {
        let servers_provider = self.servers_provider.clone();

        self.reloading = true;
        self.servers.clear();

        Command::perform(async move { servers_provider.refresh().await }, Messages::ServersRefreshed)
    }

    fn filtered_servers(&self) -> impl Iterator<Item = &Server> {
        self.servers
            .iter()
            .filter(move |(server, _id)| self.filter_server(server))
            .map(|(server, _id)| server)
    }

    fn filter_server(&self, server: &Server) -> bool {
        let text_filter = self.settings.filter.trim().to_lowercase();

        if !self.edit_favorites && !self.settings.favorites.contains(&server.name) {
            return false;
        }

        if text_filter.is_empty() {
            return true;
        }

        server.name.as_str().to_lowercase().contains(&text_filter)
    }

    fn launch(&mut self, params: &LaunchParams) {
        if let Err(error) = self.launcher.launch(params) {
            println!("Error: {:?}", error)
        }
    }

    fn switch_favorite_server(&mut self, server_name: &str) {
        match self.settings.favorites.contains(server_name) {
            true => self.settings.favorites.remove(server_name),
            false => self.settings.favorites.insert(server_name.to_string()),
        };
    }

    fn refresh_finished(&mut self, result: Result<Vec<(Server, SourceId)>, servers::Error>) {
        self.reloading = false;
        match result {
            Ok(servers) => self.servers = servers,
            Err(error) => println!("{}", error),
        };
    }

    fn main_view(&self) -> Element<Messages> {
        let main_view = match self.reloading {
            true => refreshing_view(),
            false => servers_view(
                self.filtered_servers(),
                &self.icons,
                &self.settings.favorites,
                self.edit_favorites,
            ),
        };
        column![
            header_view(&self.title(), &self.icons, self.edit_favorites),
            vertical_space(Length::Units(4)),
            filter_view(&self.settings.filter, &self.icons),
            vertical_space(Length::Units(4)),
            main_view,
            // Elements after the servers view might be invisible if there are enough
            // server in the list.234
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
            edit_favorites: false,
            edit_settings: false,
            reloading: false,
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
            Messages::StartGame(params) => self.launch(&params),
            Messages::CopyToClipboard(text) => return iced::clipboard::write(text),
            Messages::FavoriteClicked(server_name) => self.switch_favorite_server(&server_name),
            Messages::EditFavorites(edit_favorites) => self.edit_favorites = edit_favorites,
            Messages::EditSettings(edit_settings) => self.edit_settings = edit_settings,
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message, iced::Renderer<Self::Theme>> {
        match self.edit_settings {
            true => settings_view(&self.icons),
            false => self.main_view(),
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        UserSettings::save_settings(&self.settings).expect("Write settings");
    }
}
