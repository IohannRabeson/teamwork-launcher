use super::styles;

use {
    crate::{announces::Announce, application::Messages, icons::Icons, ui::VISUAL_SPACING_MEDIUM},
    iced::{
        theme,
        widget::{button, column, text},
        Element, Length,
    },
};

/// Show an announce.
///
/// An announce display a title and a message.
/// When you click anywhere on it it's discarded.
pub fn announce_view<'a>(_icons: &Icons, announce: &Announce) -> Element<'a, Messages> {
    button(column![text(&announce.title).size(24), text(&announce.message)].spacing(VISUAL_SPACING_MEDIUM))
        .padding(VISUAL_SPACING_MEDIUM)
        .width(Length::Fill)
        .style(theme::Button::Custom(Box::<styles::Announce>::default()))
        .on_press(Messages::DiscardCurrentAnnounce)
        .into()
}
