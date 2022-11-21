use std::sync::Arc;

use iced::{widget::vertical_space, Application as IcedApplication, Command, Length, Theme};

use crate::{
    icons::Icons,
    launcher::{LaunchParams, Launcher},
    servers::{self, Server, ServersProvider, SourceId},
    settings::UserSettings,
    setup::setup_launcher,
    views::{filter_view, header_view, refreshing_view, servers_view},
};

#[derive(Debug, Clone)]
pub enum Messages {
    RefreshServers,
    ServersRefreshed(Result<Vec<(Server, SourceId)>, servers::Error>),
    FilterChanged(String),
    StartGame(LaunchParams),
    CopyToClipboard(String),
    FavoriteClicked(String),
    EditFavorites(bool),
}

pub struct Application {
    settings: UserSettings,
    icons: Icons,
    servers_provider: Arc<ServersProvider>,
    servers: Vec<(Server, SourceId)>,
    edit_favorites: bool,
    reloading: bool,
    launcher: Box<dyn Launcher>,
}

enum States {
    ShowServers,
    EditServers,
    Reloading,
}

impl Application {
    fn refresh_command(&mut self) -> Command<Messages> {
        let servers_provider = self.servers_provider.clone();

        self.reloading = true;
        self.servers.clear();

        Command::perform(
            async move { servers_provider.refresh().await },
            Messages::ServersRefreshed,
        )
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
            Messages::ServersRefreshed(result) => {
                self.reloading = false;

                match result {
                    Ok(servers) => self.servers = servers,
                    Err(error) => println!("{}", error),
                };
            }
            Messages::RefreshServers => return self.refresh_command(),
            Messages::FilterChanged(text_filter) => self.settings.filter = text_filter,
            Messages::StartGame(params) => self.launch(&params),
            Messages::CopyToClipboard(text) => return iced::clipboard::write(text),
            Messages::FavoriteClicked(server_name) => self.switch_favorite_server(&server_name),
            Messages::EditFavorites(edit_favorites) => self.edit_favorites = edit_favorites,
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message, iced::Renderer<Self::Theme>> {
        use iced::widget::column;

        let main_view: iced::Element<Self::Message, iced::Renderer<Self::Theme>> = match self.reloading {
            true => refreshing_view().into(),
            false => servers_view(
                self.filtered_servers(),
                &self.icons,
                &self.settings.favorites,
                self.edit_favorites,
            )
            .into(),
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

impl Drop for Application {
    fn drop(&mut self) {
        UserSettings::save_settings(&self.settings).expect("Write settings");
    }
}
