use iced::{
    alignment::{Horizontal, Vertical},
    widget::{column, text},
    Element, Length,
};

use crate::{
    application::Messages,
    fonts,
    icons::{Icons, SvgHandle},
    launcher::LaunchParams,
    settings::UserSettings, models::Server,
};

pub use {
    self::buttons::{favorite_button, svg_button, text_button},
    header::header_view,
    servers::servers_view,
    settings::settings_view,
};

mod buttons;
mod header;
mod servers;
mod settings;

const VISUAL_SPACING_SMALL: u16 = 4;
const BIG_BUTTON_SIZE: u16 = 36;

impl From<&Server> for LaunchParams {
    fn from(server: &Server) -> Self {
        Self {
            server_ip: server.ip_port.ip().clone(),
            server_port: server.ip_port.port(),
        }
    }
}

pub fn edit_favorite_servers_view<'a, I: Iterator<Item = &'a Server>>(
    servers_iterator: I,
    icons: &Icons,
    settings: &UserSettings,
) -> Element<'a, Messages> {
    servers_view(servers_iterator, icons, settings, true)
}

pub fn refresh_view<'a>() -> Element<'a, Messages> {
    text("Reloading...")
        .width(Length::Fill)
        .height(Length::Fill)
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center)
        .font(fonts::TF2_SECONDARY)
        .size(40)
        .into()
}

pub fn error_view<'a>(message: &str) -> Element<'a, Messages> {
    column![text("Error").font(fonts::TF2_SECONDARY).size(32), text(message)]
        .padding(12)
        .into()
}
