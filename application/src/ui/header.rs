use crate::application::States;

use {
    super::{svg_button, BIG_BUTTON_SIZE, VISUAL_SPACING_SMALL},
    crate::{application::Messages, icons::Icons},
    iced::{
        widget::{horizontal_space, row, text},
        Element,
    },
};

fn title_widget<'a>(title: &str) -> Element<'a, Messages> {
    text(title).font(crate::fonts::TF2_BUILD).size(44).into()
}

pub fn header_view<'a>(title: &str, icons: &Icons, state: &States) -> Element<'a, Messages> {
    let title_widget = title_widget(title);

    match state {
        States::Normal => {
            row![
                title_widget,
                horizontal_space(iced::Length::Fill),
                svg_button(icons.settings(), BIG_BUTTON_SIZE).on_press(Messages::EditSettings),
                svg_button(icons.refresh(), BIG_BUTTON_SIZE).on_press(Messages::RefreshFavoriteServers),
                svg_button(icons.favorite_border(), BIG_BUTTON_SIZE).on_press(Messages::EditFavorites),
            ]
        }
        States::Favorites => {
            row![
                title_widget,
                horizontal_space(iced::Length::Fill),
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
                title_widget,
                horizontal_space(iced::Length::Fill),
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
