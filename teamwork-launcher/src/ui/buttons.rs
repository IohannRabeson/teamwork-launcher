use {
    crate::{
        icons::{self, SvgHandle},
        ui::styles::SvgButtonIconStyle,
    },
    iced::{
        theme,
        widget::{button, Button, Svg},
    },
};

pub fn svg_button<'a, M: Clone + 'a>(svg: SvgHandle, size: u16) -> Button<'a, M> {
    button(
        Svg::new(svg)
            .style(theme::Svg::Custom(Box::<SvgButtonIconStyle>::default()))
            .width(size)
            .height(size),
    )
}

pub fn favorite_button<'a, M: Clone + 'a>(is_favorite: bool, size: u16) -> Button<'a, M> {
    let icon = match is_favorite {
        true => icons::FAVORITE_CHECKED_ICON.clone(),
        false => icons::FAVORITE_UNCHECKED_ICON.clone(),
    };

    svg_button(icon, size)
}
