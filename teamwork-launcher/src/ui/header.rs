use crate::{
    fonts::{SUBTITLE_FONT_SIZE, TITLE_FONT_SIZE, VERSION_FONT_SIZE},
    APPLICATION_VERSION, GIT_SHA_SHORT,
};
use super::widgets::tooltip;
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
    ]
    .into()
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

pub fn header_view<'a>(title: &str, icons: &'a Icons, state: &States) -> Element<'a, Messages> {
    let title_widget = title_widget(title);

    match state {
        States::ShowServers => {
            row![
                title_widget,
                horizontal_space(iced::Length::Fill),
                settings_button(icons),
                refresh_button(icons),
                favorites_button(icons),
            ]
        }
        States::EditFavoriteServers => {
            row![
                title_widget,
                horizontal_space(iced::Length::Fill),
                settings_button(icons),
                refresh_button(icons),
                back_button(icons),
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
                back_button(icons),
            ]
        }
        _ => {
            row![title_widget, horizontal_space(iced::Length::Fill), back_button(icons),]
        }
    }
    .spacing(VISUAL_SPACING_SMALL)
    .into()
}

fn back_button(icons: &Icons) -> Element<Messages> {
    tooltip(
        svg_button(icons.back(), BIG_BUTTON_SIZE).on_press(Messages::Back),
        "Go back",
        iced::widget::tooltip::Position::Bottom
    )
    .into()
}

fn settings_button(icons: &Icons) -> Element<Messages> {
    tooltip(
        svg_button(icons.settings(), BIG_BUTTON_SIZE).on_press(Messages::EditSettings),
        "Open settings editor",
        iced::widget::tooltip::Position::Bottom
    )
    .into()
}

fn refresh_button(icons: &Icons) -> Element<Messages> {
    tooltip(
        svg_button(icons.refresh(), BIG_BUTTON_SIZE).on_press(Messages::RefreshFavoriteServers),
        "Refresh the servers information",
        iced::widget::tooltip::Position::Bottom
    )
    .into()
}

fn favorites_button(icons: &Icons) -> Element<Messages> {
    tooltip(
        svg_button(icons.favorite_border(), BIG_BUTTON_SIZE).on_press(Messages::EditFavorites),
        "Open favorites servers editor",
        iced::widget::tooltip::Position::Bottom
    )
    .into()
}