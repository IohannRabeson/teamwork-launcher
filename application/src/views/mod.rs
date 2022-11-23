use iced::{
    alignment::{Horizontal, Vertical},
    widget::{column, container, horizontal_space, row, scrollable, text, text_input, vertical_space, Column, Row},
    Alignment, Element, Length,
};

use crate::{
    application::Messages,
    fonts,
    icons::{Icons, SvgHandle},
    launcher::LaunchParams,
    servers::{Server, SourceId},
    settings::UserSettings,
    states::States,
};

use self::buttons::{favorite_button, svg_button, text_button};

const VISUAL_SPACING_SMALL: u16 = 4;
const BIG_BUTTON_SIZE: u16 = 36;

impl From<&Server> for LaunchParams {
    fn from(server: &Server) -> Self {
        Self {
            server_ip: server.ip,
            server_port: server.port,
        }
    }
}

pub fn header_view<'a>(title: &str, icons: &Icons, state: &States) -> Element<'a, Messages> {
    match state {
        States::Normal => {
            row![
                text(title).font(crate::fonts::TF2_BUILD).size(BIG_BUTTON_SIZE),
                horizontal_space(iced::Length::Fill),
                svg_button(icons.settings(), BIG_BUTTON_SIZE).on_press(Messages::EditSettings),
                svg_button(icons.refresh(), BIG_BUTTON_SIZE).on_press(Messages::RefreshServers),
                svg_button(icons.favorite_border(), BIG_BUTTON_SIZE).on_press(Messages::EditFavorites),
            ]
        }
        States::Reloading => {
            row![text(title).font(crate::fonts::TF2_BUILD).size(BIG_BUTTON_SIZE),]
        }
        _ => {
            row![
                text(title).font(crate::fonts::TF2_BUILD).size(BIG_BUTTON_SIZE),
                horizontal_space(iced::Length::Fill),
                svg_button(icons.back(), BIG_BUTTON_SIZE).on_press(Messages::Back),
            ]
        }
    }
    .spacing(VISUAL_SPACING_SMALL)
    .into()
}

pub fn edit_favorite_servers_view<'a, I: Iterator<Item = &'a (Server, SourceId)>>(
    servers_iterator: I,
    icons: &Icons,
    settings: &UserSettings,
) -> Element<'a, Messages> {
    servers_view(servers_iterator, icons, settings, true)
}

pub fn servers_view<'a, I: Iterator<Item = &'a (Server, SourceId)>>(
    servers_iterator: I,
    icons: &Icons,
    settings: &UserSettings,
    edit_favorites: bool,
) -> Element<'a, Messages> {
    column![
        servers_filter_view(&settings.filter, icons),
        vertical_space(Length::Units(4)),
        scrollable(
            servers_iterator
                .fold(Column::new().spacing(VISUAL_SPACING_SMALL), |column, (server, _source_id)| {
                    column.push(server_view(
                        server,
                        settings.favorites.contains(&server.name),
                        icons,
                        edit_favorites,
                    ))
                })
                .width(Length::Fill)
                .padding([0, 8, 0, 0]),
        )
        .scrollbar_width(8)
        .scroller_width(8)
    ]
    .into()
}

pub fn refreshing_view<'a>() -> Element<'a, Messages> {
    text("Reloading...")
        .width(Length::Fill)
        .height(Length::Fill)
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center)
        .font(fonts::TF2_SECONDARY)
        .size(40)
        .into()
}

mod buttons {
    use {
        crate::{icons::Icons, views::SvgHandle},
        iced::alignment::Vertical,
    };

    use iced::{
        widget::{button, text, Button, Svg},
        Length,
    };

    pub(crate) fn svg_button<'a, M: Clone + 'a>(svg: SvgHandle, size: u16) -> Button<'a, M> {
        button(Svg::new(svg)).width(Length::Units(size)).height(Length::Units(size))
    }

    pub(crate) fn text_button<'a, M: Clone + 'a>(content: &str) -> Button<'a, M> {
        button(
            text(content)
                .height(Length::Units(18))
                .vertical_alignment(Vertical::Center)
                .font(crate::fonts::TF2_SECONDARY)
                .size(16),
        )
    }

    pub(crate) fn favorite_button<'a, M: Clone + 'a>(is_favorite: bool, icons: &Icons, size: u16) -> Button<'a, M> {
        let icon = match is_favorite {
            true => icons.favorite(),
            false => icons.favorite_border(),
        };

        svg_button(icon, size)
    }
}

fn servers_filter_view<'a>(text: &str, icons: &Icons) -> Element<'a, Messages> {
    row![
        text_input("Filter", text, Messages::FilterChanged),
        svg_button(icons.clear(), 28).on_press(Messages::FilterChanged(String::new())),
    ]
    .align_items(iced::Alignment::Center)
    .spacing(VISUAL_SPACING_SMALL)
    .padding([0, 4])
    .into()
}

fn server_view_buttons<'a>(server: &Server, is_favorite: bool, icons: &Icons, edit_favorites: bool) -> Row<'a, Messages> {
    if edit_favorites {
        row![favorite_button(is_favorite, icons, 32).on_press(Messages::FavoriteClicked(server.name.clone())),]
    } else {
        row![
            svg_button(icons.copy(), 28)
                .on_press(Messages::CopyToClipboard(format!("connect {}:{}", server.ip, server.port))),
            text_button("Launch!").on_press(Messages::StartGame(server.into())),
        ]
    }
}

fn server_view<'a>(server: &Server, is_favorite: bool, icons: &Icons, edit_favorites: bool) -> Element<'a, Messages> {
    const BIG_FONT_SIZE: u16 = 32;

    container(row![
        column![
            text(&server.name).size(BIG_FONT_SIZE),
            text(format!(
                "Players: {} / {}",
                server.current_players_count, server.max_players_count
            )),
            text(format!("Map: {}", server.map))
        ],
        horizontal_space(Length::Fill),
        server_view_buttons(server, is_favorite, icons, edit_favorites)
            .align_items(Alignment::End)
            .spacing(VISUAL_SPACING_SMALL)
    ])
    .padding(6)
    .into()
}

pub fn settings_view<'a>() -> Element<'a, Messages> {
    column![text("Settings").font(fonts::TF2_SECONDARY).size(32),]
        .padding(12)
        .into()
}

pub fn error_view<'a>(message: &str) -> Element<'a, Messages> {
    column![text("Error").font(fonts::TF2_SECONDARY).size(32), text(message)]
        .padding(12)
        .into()
}
