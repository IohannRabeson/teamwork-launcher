pub use {
    self::buttons::{favorite_button, svg_button, text_button},
    header::header_view,
    servers::{no_favorite_servers_view, servers_view, servers_view_edit_favorites},
    settings::settings_view,
};
use {
    crate::{application::Messages, fonts, icons::SvgHandle},
    iced::{
        alignment::{Horizontal, Vertical},
        widget::{column, text},
        Element, Length,
    },
};

mod announces;
mod buttons;
mod header;
mod servers;
mod settings;
mod styles;
mod widgets;

pub const VISUAL_SPACING_SMALL: u16 = 4;
pub const VISUAL_SPACING_MEDIUM: u16 = 8;
pub const VISUAL_SPACING_BIG: u16 = 12;
pub const BIG_BUTTON_SIZE: u16 = 36;

pub use announces::announce_view;

pub fn refresh_view<'a>() -> Element<'a, Messages> {
    text("Reloading...")
        .width(Length::Fill)
        .height(Length::Fill)
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center)
        .font(fonts::TF2_SECONDARY)
        .size(fonts::TITLE_FONT_SIZE)
        .into()
}

pub fn error_view<'a>(message: &str) -> Element<'a, Messages> {
    column![
        text("Error").font(fonts::TF2_SECONDARY).size(fonts::SUBTITLE_FONT_SIZE),
        text(message)
    ]
    .padding(12)
    .into()
}
