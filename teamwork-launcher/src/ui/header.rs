use crate::{APPLICATION_VERSION, fonts::{TITLE_FONT_SIZE, VERSION_FONT_SIZE, SUBTITLE_FONT_SIZE}, GIT_SHA_SHORT};

use {
    super::{svg_button, BIG_BUTTON_SIZE, VISUAL_SPACING_SMALL},
    crate::{
        application::{Messages, States},
        icons::Icons,
    },
    iced::{
        alignment::Horizontal,
        widget::{container, horizontal_space, row, text},
        Element, Length,
    },
};

fn title_widget<'a>(title: &str) -> Element<'a, Messages> {
    row![
        text(title).font(crate::fonts::TF2_BUILD).size(TITLE_FONT_SIZE),
        text(format!("{}-{}", APPLICATION_VERSION, GIT_SHA_SHORT)).size(VERSION_FONT_SIZE)
    ].into()
}

fn subtitle_widget<'a>(title: &str) -> Element<'a, Messages> {
    container(text(title).font(crate::fonts::TF2_SECONDARY).size(SUBTITLE_FONT_SIZE))
        .padding([0, 0, 0, 16])
        .center_y()
        .align_x(Horizontal::Left)
        .height(Length::Units(TITLE_FONT_SIZE))
        .width(Length::Fill)
        .into()
}

pub fn header_view<'a>(title: &str, icons: &Icons, state: &States) -> Element<'a, Messages> {
    let title_widget = title_widget(title);

    match state {
        States::ShowServers => {
            row![
                title_widget,
                horizontal_space(iced::Length::Fill),
                svg_button(icons.settings(), BIG_BUTTON_SIZE).on_press(Messages::EditSettings),
                svg_button(icons.refresh(), BIG_BUTTON_SIZE).on_press(Messages::RefreshFavoriteServers),
                svg_button(icons.favorite_border(), BIG_BUTTON_SIZE).on_press(Messages::EditFavorites),
            ]
        }
        States::EditFavoriteServers => {
            row![
                title_widget,
                subtitle_widget("Edit favorite servers"),
                svg_button(icons.settings(), BIG_BUTTON_SIZE).on_press(Messages::EditSettings),
                svg_button(icons.refresh(), BIG_BUTTON_SIZE).on_press(Messages::RefreshServers),
                svg_button(icons.back(), BIG_BUTTON_SIZE).on_press(Messages::Back),
            ]
        }
        States::Reloading => {
            row![title_widget,]
        }
        States::Settings => {
            row![
                horizontal_space(iced::Length::Units(VISUAL_SPACING_SMALL)),
                title_widget,
                subtitle_widget("Settings"),
                svg_button(icons.back(), BIG_BUTTON_SIZE).on_press(Messages::Back),
            ]
        }
        _ => {
            row![
                title_widget,
                horizontal_space(iced::Length::Fill),
                svg_button(icons.back(), BIG_BUTTON_SIZE).on_press(Messages::Back),
            ]
        }
    }
    .spacing(VISUAL_SPACING_SMALL)
    .into()
}
