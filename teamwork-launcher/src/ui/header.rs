use {
    super::{svg_button, widgets::tooltip, BIG_BUTTON_SIZE, VISUAL_SPACING_SMALL},
    crate::{
        application::{Messages, States},
        fonts::{SUBTITLE_FONT_SIZE, TITLE_FONT_SIZE, VERSION_FONT_SIZE},
        icons, APPLICATION_VERSION, GIT_SHA_SHORT,
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

pub fn header_view<'a>(title: &str, state: &States) -> Element<'a, Messages> {
    let title_widget = title_widget(title);

    match state {
        States::ShowServers => {
            row![
                title_widget,
                horizontal_space(iced::Length::Fill),
                settings_button(),
                refresh_button(Messages::RefreshFavoriteServers),
                favorites_button(),
            ]
        }
        States::EditFavoriteServers => {
            row![
                title_widget,
                horizontal_space(iced::Length::Fill),
                settings_button(),
                refresh_button(Messages::RefreshServers),
                back_button(),
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
                back_button(),
            ]
        }
        _ => {
            row![title_widget, horizontal_space(iced::Length::Fill), back_button(),]
        }
    }
    .spacing(VISUAL_SPACING_SMALL)
    .into()
}

fn back_button<'a>() -> Element<'a, Messages> {
    tooltip(
        svg_button(icons::BACK_ICON.clone(), BIG_BUTTON_SIZE).on_press(Messages::Back),
        "Go back",
        iced::widget::tooltip::Position::Bottom,
    )
}

fn settings_button<'a>() -> Element<'a, Messages> {
    tooltip(
        svg_button(icons::SETTINGS_ICON.clone(), BIG_BUTTON_SIZE).on_press(Messages::EditSettings),
        "Open settings editor",
        iced::widget::tooltip::Position::Bottom,
    )
}

fn refresh_button<'a>(message: Messages) -> Element<'a, Messages> {
    tooltip(
        svg_button(icons::REFRESH_ICON.clone(), BIG_BUTTON_SIZE).on_press(message),
        "Refresh the servers information",
        iced::widget::tooltip::Position::Bottom,
    )
}

fn favorites_button<'a>() -> Element<'a, Messages> {
    tooltip(
        svg_button(icons::FAVORITE_UNCHECKED_ICON.clone(), BIG_BUTTON_SIZE).on_press(Messages::EditFavorites),
        "Open favorites servers editor",
        iced::widget::tooltip::Position::Bottom,
    )
}
