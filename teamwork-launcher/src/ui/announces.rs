use {
    crate::{announces::Announce, application::Messages, icons::Icons, ui::svg_button},
    iced::{
        theme,
        widget::{button, column, container, horizontal_space, row, text, vertical_space},
        Background, Element, Length, Theme, Vector,
    },
};

#[derive(Default)]
struct AnnounceStyle;

impl button::StyleSheet for AnnounceStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let mut result = button::Appearance::default();

        result.background = Some(Background::Color(style.palette().danger));
        result.text_color = style.palette().text;
        result.shadow_offset = Vector::new(0f32, 0f32);
        result.border_radius = 3.0;
        result
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
