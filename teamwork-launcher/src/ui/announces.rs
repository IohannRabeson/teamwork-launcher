use crate::ui::VISUAL_SPACING_MEDIUM;

use {
    crate::{announces::Announce, application::Messages, icons::Icons},
    iced::{
        theme,
        widget::{button, column, text},
        Background, Element, Length, Theme, Vector,
    },
};

#[derive(Default)]
struct AnnounceStyle;

impl button::StyleSheet for AnnounceStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::new(0f32, 0f32),
            background: Some(Background::Color(style.palette().danger)),
            border_radius: 3.0,
            text_color: style.palette().text,
            ..Default::default()
        }
    }
}

/// Show an announce.
/// 
/// An announce display a title and a message. 
/// When you click anywhere on it it's discarded.
pub fn announce_view<'a>(_icons: &Icons, announce: &Announce) -> Element<'a, Messages> {
    button(column![text(&announce.title).size(24), text(&announce.message)].spacing(VISUAL_SPACING_MEDIUM))
        .padding(VISUAL_SPACING_MEDIUM)
        .width(Length::Fill)
        .style(theme::Button::Custom(Box::new(AnnounceStyle::default())))
        .on_press(Messages::DiscardCurrentAnnounce)
        .into()
}
