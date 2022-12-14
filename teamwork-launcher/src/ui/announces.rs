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

pub fn announce_view<'a>(_icons: &Icons, announce: &Announce) -> Element<'a, Messages> {
    const SPACING: u16 = 8;

    button(column![text(&announce.title).size(24), text(&announce.message)].spacing(SPACING))
        .padding(SPACING)
        .width(Length::Fill)
        .style(theme::Button::Custom(Box::new(AnnounceStyle::default())))
        .on_press(Messages::DiscardCurrentAnnounce)
        .into()
}
