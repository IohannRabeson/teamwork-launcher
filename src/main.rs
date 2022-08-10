use enum_as_inner::EnumAsInner;
use iced::pure::text_input;
use iced::{
    pure::{
        button, container, scrollable, text,
        widget::{Column, Row},
        Application,
    },
    Background, Command, Length, Settings, Space,
};

use iced_color_helpers::parse_color;
use iced_pure::toggler;
use serde::{Deserialize, Serialize};
use server_info::{parse_server_infos, ServerInfo};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{collections::HashSet, sync::Arc};
use thiserror::Error;

mod iced_color_helpers;
mod server_info;

#[derive(Error, Debug, Clone)]
enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] Arc<reqwest::Error>),
    #[error("UI error: {0}")]
    Ui(#[from] Arc<iced::Error>),
    #[error("JSON error: {0}")]
    Json(#[from] Arc<serde_json::Error>),
    #[error("IO error: {0}")]
    Io(#[from] Arc<std::io::Error>),
}

#[derive(Debug, EnumAsInner)]
enum States {
    Reload,
    DisplayServers,
    Error,
}

#[derive(Default, Serialize, Deserialize)]
struct UserSettings {
    pub favorites: HashSet<String>,
    pub filter: String,
}

impl UserSettings {
    const USER_SETTINGS_FILE_NAME: &'static str = "tf2-launcher";

    fn save_settings(settings: &UserSettings) -> Result<(), Error> {
        let json = serde_json::to_string(settings).map_err(|e| Error::Json(Arc::new(e)))?;
        let mut file =
            File::create(Self::USER_SETTINGS_FILE_NAME).map_err(|e| Error::Io(Arc::new(e)))?;

        file.write_all(json.as_bytes())
            .map_err(|e| Error::Io(Arc::new(e)))
    }

    fn load_settings() -> Result<UserSettings, Error> {
        if !Path::new(Self::USER_SETTINGS_FILE_NAME).is_file() {
            return Ok(UserSettings::default());
        }

        let mut file =
            File::open(Self::USER_SETTINGS_FILE_NAME).map_err(|e| Error::Io(Arc::new(e)))?;
        let mut json = String::new();

        file.read_to_string(&mut json)
            .map_err(|e| Error::Io(Arc::new(e)))?;

        Ok(serde_json::from_str(&json).map_err(|e| Error::Json(Arc::new(e)))?)
    }
}

struct MyApplication {
    server_infos: Vec<ServerInfo>,
    error_message: Option<String>,
    settings: UserSettings,
    state: States,
    edit_favorites: bool,
    palette: Box<Palette>,
}

struct Palette {
    pub background: iced::Color,
    pub foreground: iced::Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self { background: parse_color("#F2F2F2"), foreground: parse_color("#0D0D0D") }
    }
}

struct MainContainerStyle<'l> {
    palette: &'l Palette,
}

impl<'l> iced::container::StyleSheet for MainContainerStyle<'l> {
    fn style(&self) -> iced::container::Style {
        iced::container::Style {
            text_color: Some(self.palette.foreground),
            background: Some(Background::Color(self.palette.background)),
            ..Default::default()
        }
    }
}

impl<'l> MainContainerStyle<'l> {
    pub fn new(palette: &'l Palette) -> Self {
        Self { palette: palette }
    }
}

#[derive(Debug, Clone)]
enum Messages {
    UpdateServers,
    ServersInfoResponse(Result<Vec<ServerInfo>, Error>),
    FilterChanged(String),
    ClearFilter,
    Connect(std::net::Ipv4Addr, u16),
    EditFavorites(bool),
    FavoriteClicked(bool, usize),
    CopyClicked(usize),
}

struct ServerCardStyleSheet;

impl iced_pure::widget::button::StyleSheet for ServerCardStyleSheet {
    fn active(&self) -> iced::button::Style {
        use iced_color_helpers::parse_color;
        let mut style = iced::button::Style::default();
        style.background = Some(Background::Color(parse_color("#8C3232")));
        style.text_color = parse_color("#F2F2F2");
        style.border_radius = 6f32;
        style
    }

    fn hovered(&self) -> iced::button::Style {
        use iced_color_helpers::parse_color;
        let mut active = self.active();
        active.background = Some(Background::Color(parse_color("#BF7449")));

        active
    }

    fn pressed(&self) -> iced::button::Style {
        iced::button::Style {
            shadow_offset: iced::Vector::default(),
            ..self.active()
        }
    }
}

impl MyApplication {
    async fn request_servers_infos(address: &str) -> Result<Vec<ServerInfo>, Error> {
        let html = reqwest::get(address)
            .await
            .map_err(|e| Error::Http(Arc::new(e)))?
            .text()
            .await
            .map_err(|e| Error::Http(Arc::new(e)))?;

        Ok(parse_server_infos(&html))
    }

    fn server_view<'l>(
        &self,
        server_info: &ServerInfo,
        index: usize,
    ) -> iced::pure::Element<'l, Messages> {
        let informations = Column::new()
            .push(text(&server_info.name))
            .push(text(format!(
                "{} / {} players",
                server_info.current_players_count, server_info.max_players_count
            )));
        let mut buttons = Row::new().push(Space::with_width(Length::Fill));
        if self.edit_favorites {
            buttons = buttons.push(toggler(
                "Add to favorites".to_string(),
                self.settings.favorites.contains(&server_info.name),
                move |toggled| Messages::FavoriteClicked(toggled, index),
            ));
        } else {
            buttons = buttons.push(button("Copy").on_press(Messages::CopyClicked(index)));
        }
        let content = Row::new().push(informations).push(buttons);

        button(content)
            .style(ServerCardStyleSheet {})
            .on_press(Messages::Connect(server_info.ip, server_info.port))
            .padding(12)
            .into()
    }

    fn servers_view<'a>(&self) -> iced::pure::Element<'a, Messages> {
        let mut column: Column<Messages> = Column::new().spacing(12);

        for server_element in self
            .server_infos
            .iter()
            .enumerate()
            .filter(|(_, server_info)| self.accept_server(*server_info))
            .map(|(index, server_info)| self.server_view(server_info, index))
        {
            column = column.push(server_element);
        }

        scrollable(column).into()
    }

    fn accept_server(&self, server_info: &ServerInfo) -> bool {
        self.accept_filter(server_info) && self.accept_favorite(server_info)
    }

    fn accept_filter(&self, server_info: &ServerInfo) -> bool {
        self.settings.filter.is_empty()
            || server_info
                .name
                .as_str()
                .to_lowercase()
                .contains(&self.settings.filter)
    }

    fn accept_favorite(&self, server_info: &ServerInfo) -> bool {
        self.edit_favorites
            || self.settings.favorites.is_empty()
            || self.settings.favorites.contains(&server_info.name)
    }

    fn reload_view<'l>(&'l self) -> iced::pure::Element<'l, Messages> {
        text("Reloading...")
            .width(Length::Fill)
            .height(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .vertical_alignment(iced::alignment::Vertical::Center)
            .into()
    }

    fn filter_view<'l>(&'l self) -> iced::pure::Element<'l, Messages> {
        let filter = container(text_input(
            "Filter...",
            &self.settings.filter,
            Messages::FilterChanged,
        ))
        .center_y()
        .height(Length::Units(25))
        .width(Length::Fill);
        let row: Row<Messages> = Row::new()
            .push(filter)
            .push(button("X").on_press(Messages::ClearFilter))
            .push(button("Refresh").on_press(Messages::UpdateServers))
            .spacing(6);

        row.into()
    }

    fn display_servers_view<'l>(&'l self) -> iced::pure::Element<'l, Messages> {
        let filter = self.filter_view();
        let favorite_settings = toggler(
            "Edit Favorites".to_string(),
            self.edit_favorites,
            Messages::EditFavorites,
        );
        let column: Column<Messages> = Column::new()
            .push(filter)
            .push(favorite_settings)
            .push(self.servers_view())
            .push(text(self.error_message.as_ref().unwrap_or(&String::new())))
            .width(Length::Fill)
            .padding(3)
            .spacing(12);

        column.into()
    }

    fn error_view<'l>(&'l self) -> iced::pure::Element<'l, Messages> {
        let column: Column<Messages> = Column::new()
            .push(text(self.error_message.as_ref().unwrap()))
            .push(button("Retry").on_press(Messages::UpdateServers))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(iced::Alignment::Center)
            .padding(3);

        column.into()
    }

    fn launch_game(&self, ip: &std::net::Ipv4Addr, port: u16) {
        use std::process::Command;

        Command::new(r"C:\Program Files (x86)\Steam\Steam.exe")
            .args(["-applaunch", "440", "+connect", &format!("{}:{}", ip, port)])
            .output()
            .expect("failed to execute process");
    }
}

const SKIAL_URL: &str = "https://www.skial.com/api/servers.php";

impl Drop for MyApplication {
    fn drop(&mut self) {
        UserSettings::save_settings(&self.settings).expect("Write settings");
    }
}

impl Application for MyApplication {
    type Message = Messages;
    type Executor = iced::executor::Default;
    type Flags = UserSettings;

    fn new(settings: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                server_infos: Vec::new(),
                error_message: None,
                settings,
                state: States::Reload,
                edit_favorites: false,
                palette: Box::new(Palette::default()),
            },
            Command::perform(
                MyApplication::request_servers_infos(SKIAL_URL),
                Messages::ServersInfoResponse,
            ),
        )
    }

    fn title(&self) -> String {
        "TF2 launcher".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Messages::ServersInfoResponse(response) => {
                match response {
                    Ok(servers_info) => {
                        self.server_infos = servers_info;
                        self.error_message = None;
                        self.state = States::DisplayServers;
                    }
                    Err(error_message) => {
                        self.error_message = Some(error_message.to_string());
                        self.state = States::Error;
                    }
                };
            }
            Messages::UpdateServers => {
                return Command::perform(
                    MyApplication::request_servers_infos(SKIAL_URL),
                    Messages::ServersInfoResponse,
                )
            }
            Messages::FilterChanged(filter) => self.settings.filter = filter.to_lowercase(),
            Messages::ClearFilter => self.settings.filter.clear(),
            Messages::Connect(ip, port) => self.launch_game(&ip, port),
            Messages::FavoriteClicked(toggled, server_index) => {
                match toggled {
                    true => self
                        .settings
                        .favorites
                        .insert(self.server_infos[server_index].name.clone()),
                    false => self
                        .settings
                        .favorites
                        .remove(&self.server_infos[server_index].name),
                };
            }
            Messages::CopyClicked(server_index) => {
                println!("Copy {}", server_index);
            }
            Messages::EditFavorites(toggled) => {
                self.edit_favorites = toggled;
            }
        }

        Command::none()
    }

    fn view(&self) -> iced::pure::Element<Messages> {
        let content = match self.state {
            States::Reload => self.reload_view(),
            States::DisplayServers => self.display_servers_view(),
            States::Error => self.error_view(),
        };

        container(content).style(MainContainerStyle::new(&self.palette)).height(Length::Fill).padding(12).into()
    }
}

fn main() -> Result<(), Error> {
    MyApplication::run(Settings::with_flags(UserSettings::load_settings()?))
        .map_err(|e| Error::Ui(Arc::new(e)))
}
