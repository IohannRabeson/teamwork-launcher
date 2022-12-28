pub use {
    self::buttons::{favorite_button, svg_button, text_button},
    header::header_view,
    servers::{servers_view, servers_view_edit_favorites},
    settings::settings_view,
};
use {
    crate::{application::Messages, fonts, icons::SvgHandle},
    iced::{
        alignment::{Horizontal, Vertical},
        widget::{button, column, text},
        Element, Length,
    },
};

mod advanced_filter;
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
pub const BIG_BUTTON_SIZE: u16 = 26;
pub const SMALL_BUTTON_SIZE: u16 = 20;

pub use announces::announce_view;
use iced::{widget::container, Alignment};

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

pub fn no_favorite_servers_view<'a>() -> Element<'a, Messages> {
    container(
        column![
            text("No favorite servers!").font(fonts::TF2_SECONDARY).size(36),
            text("You can edit the list of your favorite servers by clicking on the star button on the top right of the window."),
            button("Edit favorite servers").on_press(Messages::EditFavorites),
        ]
        .align_items(Alignment::Center)
        .spacing(12),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .into()
}
