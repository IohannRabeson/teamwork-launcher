use enum_as_inner::EnumAsInner;
use iced::{
    pure::{
        button, container, scrollable, text, text_input, toggler,
        widget::{Column, Row},
        Application,
    },
    svg, Command, Length, Settings, Space,
};

use server_info::{parse_server_infos, ServerInfo};
use settings::UserSettings;
use std::sync::Arc;
use styles::Palette;
use thiserror::Error;

mod colors;
mod icons;
mod server_info;
mod settings;
mod styles;
mod ui;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] Arc<reqwest::Error>),
    #[error("UI error: {0}")]
    Ui(#[from] Arc<iced::Error>),
    #[error("JSON error: {0}")]
    Json(#[from] Arc<serde_json::Error>),
    #[error("IO error: {0}")]
    Io(#[from] Arc<std::io::Error>),
    #[error("XML parse error: {0}")]
    Xml(#[from] Arc<xmltree::ParseError>),
    #[error("XML error: {0}")]
    XmlTree(#[from] Arc<xmltree::Error>),
}

#[derive(Debug, EnumAsInner)]
enum States {
    Reload,
    DisplayServers,
    Error,
}

#[derive(Debug, Clone)]
pub enum Messages {
    UpdateServers,
    ServersInfoResponse(Result<Vec<ServerInfo>, Error>),
    FilterChanged(String),
    ClearFilter,
    Connect(std::net::Ipv4Addr, u16),
    EditFavorites(bool),
    FavoriteClicked(bool, usize),
    CopyClicked(usize),
}
pub struct ApplicationIcons {
    copy_image_handle: svg::Handle,
    favorite_on: svg::Handle,
    favorite_off: svg::Handle,
    refresh: svg::Handle,
    clear: svg::Handle,
}

impl ApplicationIcons {
    pub fn load_application_icons(light_color: &iced::Color, dark_color: &iced::Color) -> ApplicationIcons {
        ApplicationIcons {
            copy_image_handle: crate::icons::load_svg(
                include_bytes!("../icons/copy.svg"),
                light_color,
                "copy.svg",
            ),
            favorite_on: crate::icons::load_svg(
                include_bytes!("../icons/favorite.svg"),
                light_color,
                "favorite.svg",
            ),
            favorite_off: crate::icons::load_svg(
                include_bytes!("../icons/favorite_border.svg"),
                light_color,
                "favorite_border.svg",
            ),
            refresh: crate::icons::load_svg(include_bytes!("../icons/refresh.svg"), dark_color, "refresh.svg"),
            clear: crate::icons::load_svg(include_bytes!("../icons/clear.svg"), dark_color, "clear.svg"),
        }
    }
}

struct MyApplication {
    server_infos: Vec<ServerInfo>,
    error_message: Option<String>,
    settings: UserSettings,
    state: States,
    edit_favorites: bool,
    palette: Box<Palette>,
    icons: ApplicationIcons,
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

    fn server_view<'l>(&'l self, server_info: &ServerInfo, index: usize) -> iced::pure::Element<'l, Messages> {
        let informations = Column::new()
            .push(text(&server_info.name))
            .push(text(format!("{} / {} players", server_info.current_players_count, server_info.max_players_count)))
            .push(text(&server_info.map));
        let mut buttons = Row::new().push(Space::with_width(Length::Fill));
        if self.edit_favorites {
            buttons = buttons.push(ui::favorite_button::FavoriteButton::new(
                &self.icons,
                &self.palette,
                self.settings.favorites.contains(&server_info.name),
                move |toggled| Messages::FavoriteClicked(toggled, index),
            ));
        } else {
            buttons = buttons.push(ui::svg_card_button(
                self.icons.copy_image_handle.clone(),
                Messages::CopyClicked(index),
                &self.palette,
            ));
        }
        let content = Row::new().push(informations).push(buttons);

        button(content)
            .style(styles::ServerCardStyleSheet::new(&self.palette))
            .on_press(Messages::Connect(server_info.ip, server_info.port))
            .padding(12)
            .into()
    }

    fn servers_view<'l>(&'l self) -> iced::pure::Element<'l, Messages> {
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
        self.settings.filter.is_empty() || server_info.name.as_str().to_lowercase().contains(&self.settings.filter)
    }

    fn accept_favorite(&self, server_info: &ServerInfo) -> bool {
        self.edit_favorites
            || self.settings.favorites.is_empty()
            || self.settings.favorites.contains(&server_info.name)
    }

    fn reload_view<'l>(&'l self) -> iced::pure::Element<'l, Messages> {
        container(text("Reloading..."))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn filter_view<'l>(&'l self) -> iced::pure::Element<'l, Messages> {
        let filter = container(text_input("Filter...", &self.settings.filter, Messages::FilterChanged).padding(6))
            .center_y()
            .width(Length::Fill);
        let row: Row<Messages> = Row::new()
            .push(filter)
            .push(ui::svg_default_button(
                self.icons.clear.clone(),
                Messages::ClearFilter,
                32u16,
            ))
            .push(ui::svg_default_button(
                self.icons.refresh.clone(),
                Messages::UpdateServers,
                32u16,
            ))
            .spacing(6);

        row.into()
    }

    fn display_servers_view<'l>(&'l self) -> iced::pure::Element<'l, Messages> {
        let filter = self.filter_view();
        let favorite_settings = toggler(
            "Edit Favorites: ".to_string(),
            self.edit_favorites,
            Messages::EditFavorites,
        )
        .text_alignment(iced::alignment::Horizontal::Right)
        .style(styles::ToggleStyle::new(&self.palette));
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
        let palette = Box::new(Palette::default());
        let card_foreground_color = palette.card_foreground.clone();
        let foreground_color = palette.foreground.clone();

        (
            Self {
                server_infos: Vec::new(),
                error_message: None,
                settings,
                state: States::Reload,
                edit_favorites: false,
                palette,
                icons: ApplicationIcons::load_application_icons(&card_foreground_color, &foreground_color),
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
                    false => self.settings.favorites.remove(&self.server_infos[server_index].name),
                };
            }
            Messages::CopyClicked(server_index) => {
                let server = &self.server_infos[server_index];

                return iced::clipboard::write(format!("connect {}:{}", server.ip, server.port));
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

        container(content)
            .style(styles::MainContainerStyle::new(&self.palette))
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(12)
            .into()
    }
}

fn main() -> Result<(), Error> {
    MyApplication::run(Settings::with_flags(UserSettings::load_settings()?)).map_err(|e| Error::Ui(Arc::new(e)))
}
