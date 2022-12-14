use {
    crate::{icons::Icons, ui::SvgHandle},
    iced::{
        alignment::Vertical,
        widget::{button, text, Button, Svg},
        Length,
    },
};

pub fn svg_button<'a, M: Clone + 'a>(svg: SvgHandle, size: u16) -> Button<'a, M> {
    button(Svg::new(svg)).width(Length::Units(size)).height(Length::Units(size))
}

pub fn text_button<'a, M: Clone + 'a>(content: &str) -> Button<'a, M> {
    button(
        text(content)
            .height(Length::Units(18))
            .vertical_alignment(Vertical::Center)
            .font(crate::fonts::TF2_SECONDARY)
            .size(16),
    )
}

pub fn favorite_button<'a, M: Clone + 'a>(is_favorite: bool, icons: &Icons, size: u16) -> Button<'a, M> {
    let icon = match is_favorite {
        true => icons.favorite(),
        false => icons.favorite_border(),
    };

    svg_button(icon, size)
}
