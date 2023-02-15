use {
    super::{buttons::svg_button, widgets::tooltip},
    crate::{
        application::{Message, Screens},
        icons,
    },
    iced::{
        alignment::Horizontal,
        widget::{container, horizontal_space, row, text},
        Alignment, Element, Length,
    },
};

const TITLE_FONT_SIZE: u16 = 44;
const SUBTITLE_FONT_SIZE: u16 = 32;
const BIG_BUTTON_SIZE: u16 = 26;
const VERSION_FONT_SIZE: u16 = 16;
const VISUAL_SPACING_SMALL: u16 = 4;

const APPLICATION_VERSION: &str = "1.2.3.4";
const GIT_SHA_SHORT: &str = "abcdef";

pub fn header_view<'a>(title: &str, view: &Screens) -> Element<'a, Message> {
    let title_widget = title_widget(title);

    match view {
        Screens::Main(_) => {
            row![
                title_widget,
                horizontal_space(iced::Length::Fill),
                settings_button(),
                refresh_button(Message::RefreshServers),
            ]
        }
        Screens::Settings => {
            row![title_widget, horizontal_space(iced::Length::Fill), back_button(),]
        }
        Screens::Settings => {
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
    .align_items(Alignment::Center)
    .padding([8, 8, 0, 8])
    .spacing(VISUAL_SPACING_SMALL)
    .into()
}

fn title_widget<'a>(title: &str) -> Element<'a, Message> {
    row![
        text(title).font(crate::fonts::TF2_BUILD).size(TITLE_FONT_SIZE),
        text(format!("{}-{}", APPLICATION_VERSION, GIT_SHA_SHORT)).size(VERSION_FONT_SIZE)
    ]
    .into()
}

fn subtitle_widget<'a>(title: &str) -> Element<'a, Message> {
    container(text(title).font(crate::fonts::TF2_SECONDARY).size(SUBTITLE_FONT_SIZE))
        .padding([0, 0, 0, 16])
        .center_y()
        .align_x(Horizontal::Left)
        .height(Length::Units(TITLE_FONT_SIZE))
        .width(Length::Fill)
        .into()
}

fn back_button<'a>() -> Element<'a, Message> {
    tooltip(
        svg_button(icons::BACK_ICON.clone(), BIG_BUTTON_SIZE).on_press(Message::Back),
        "Go back",
        iced::widget::tooltip::Position::Bottom,
    )
}

fn settings_button<'a>() -> Element<'a, Message> {
    tooltip(
        svg_button(icons::SETTINGS_ICON.clone(), BIG_BUTTON_SIZE).on_press(Message::ShowSettings),
        "Open settings editor",
        iced::widget::tooltip::Position::Bottom,
    )
}

fn refresh_button<'a>(message: Message) -> Element<'a, Message> {
    tooltip(
        svg_button(icons::REFRESH_ICON.clone(), BIG_BUTTON_SIZE).on_press(message),
        "Refresh the servers information",
        iced::widget::tooltip::Position::Bottom,
    )
}
