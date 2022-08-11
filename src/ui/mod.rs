use iced::{svg, Svg, Length, pure::widget::Row};
use iced_pure::{button, text};

pub mod favorite_button;

use crate::{Messages, styles::{CardButtonStyleSheet, Palette}};

pub fn svg_card_button<'l>(svg: svg::Handle, message: Messages, palette: &'l Palette) -> iced::pure::Element<'l, Messages> 
{
    button(Svg::new(svg.clone()))
            .width(Length::Units(24))
            .height(Length::Units(24))    
            .style(CardButtonStyleSheet::new(&palette))
            .on_press(message)
            .into()
}

pub fn svg_default_button<'l>(svg: svg::Handle, message: Messages, size: u16) -> iced::pure::Element<'l, Messages> 
{
    button(Svg::new(svg.clone()))
            .width(Length::Units(size))
            .height(Length::Units(size))    
            .on_press(message)
            .into()
}

pub fn text_icon_button<'l>(label: &str, icon: svg::Handle, message: Messages, palette: &'l Palette) -> iced::pure::Element<'l, Messages> 
{
    let content = Row::new()
        .push(text(label))
        .push(Svg::new(icon))
        ;

    button(content)
        .width(Length::Units(24))
        .height(Length::Units(24))    
        .on_press(message)
        .into()
}
