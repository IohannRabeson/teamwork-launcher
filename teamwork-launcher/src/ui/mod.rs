use iced::{
    alignment::{Horizontal, Vertical},
    theme,
    widget::{button, column, container, horizontal_space, row, text, vertical_space},
    Background, Element, Length, Theme, Vector,
};

use crate::{
    announces::Announce,
    application::Messages,
    fonts,
    icons::{Icons, SvgHandle},
};

pub use {
    self::buttons::{favorite_button, svg_button, text_button},
    header::header_view,
    servers::{no_favorite_servers_view, servers_view, servers_view_edit_favorites},
    settings::settings_view,
};

mod buttons;
mod header;
mod servers;
mod settings;

const VISUAL_SPACING_SMALL: u16 = 4;
const BIG_BUTTON_SIZE: u16 = 36;

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

#[derive(Default)]
struct AnnounceStyle;

impl button::StyleSheet for AnnounceStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let mut result = button::Appearance::default();

        result.background = Some(Background::Color(style.palette().danger));
        result.shadow_offset = Vector::new(0f32, 0f32);
        result
    }
}

impl container::StyleSheet for AnnounceStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let mut result = container::Appearance::default();

        result.border_width = 0f32;
        result.background = Some(Background::Color(style.palette().danger));
        result.text_color = Some(style.palette().text.clone());

        result
    }
}

pub fn announce_view<'a>(icons: &Icons, announce: &Announce) -> Element<'a, Messages> {
    const SPACING: u16 = 8;

    let discard_announce_button = svg_button(icons.clear().clone(), 24)
        .on_press(Messages::DiscardCurrentAnnounce)
        .style(theme::Button::Custom(Box::new(AnnounceStyle::default())));

    container(row![
        column![text(&announce.title).size(24), text(&announce.message)].spacing(SPACING),
        horizontal_space(Length::Fill),
        column![
            vertical_space(Length::Shrink),
            discard_announce_button,
            vertical_space(Length::Shrink),
        ],
    ])
    .padding(SPACING)
    .width(Length::Fill)
    .style(theme::Container::Custom(Box::new(AnnounceStyle::default())))
    .into()
}
