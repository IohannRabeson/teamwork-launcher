use {
    crate::{
        icons,
        ui::{styles::SvgButtonIconStyle, SvgHandle},
    },
    iced::{
        alignment::Vertical,
        theme,
        widget::{button, svg, svg::Appearance, text, Button, Svg},
        Color, Length, Theme,
    },
};

pub fn svg_button<'a, M: Clone + 'a>(svg: SvgHandle, size: u16) -> Button<'a, M> {
    button(
        Svg::new(svg)
            .style(theme::Svg::Custom(Box::new(SvgButtonIconStyle::default())))
            .width(Length::Units(size))
            .height(Length::Units(size)),
    )
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

pub fn favorite_button<'a, M: Clone + 'a>(is_favorite: bool, size: u16) -> Button<'a, M> {
    let icon = match is_favorite {
        true => icons::FAVORITE_CHECKED_ICON.clone(),
        false => icons::FAVORITE_UNCHECKED_ICON.clone(),
    };

    svg_button(icon, size)
}
