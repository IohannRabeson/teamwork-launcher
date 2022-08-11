use iced::{svg, Length, Svg};
use iced_pure::button;
use crate::{
    styles::{CardButtonStyleSheet, Palette},
    Messages,
};

pub mod favorite_button;

pub fn svg_card_button<'l>(
    svg: svg::Handle,
    message: Messages,
    palette: &'l Palette,
) -> iced::pure::Element<'l, Messages> {
    button(Svg::new(svg.clone()))
        .width(Length::Units(24))
        .height(Length::Units(24))
        .style(CardButtonStyleSheet::new(&palette))
        .on_press(message)
        .into()
}

pub fn svg_default_button<'l>(
    svg: svg::Handle,
    message: Messages,
    size: u16,
) -> iced::pure::Element<'l, Messages> {
    button(Svg::new(svg.clone()))
        .width(Length::Units(size))
        .height(Length::Units(size))
        .on_press(message)
        .into()
}
