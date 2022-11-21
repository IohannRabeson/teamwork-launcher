use std::collections::BTreeSet;

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, horizontal_space, row, scrollable, text, text_input, Button, Column, Row, Svg,
    },
    Alignment, Element, Length,
};

use crate::{application::Messages, fonts, icons::Icons, servers::Server};
use crate::{icons::SvgHandle, launcher::LaunchParams};

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

fn svg_button<'a, M: Clone + 'a>(svg: SvgHandle, size: u16) -> Button<'a, M> {
    button(Svg::new(svg))
        .width(Length::Units(size))
        .height(Length::Units(size))
}

fn text_button<'a, M: Clone + 'a>(content: &str) -> Button<'a, M> {
    button(
        text(content)
            .height(Length::Units(18))
            .vertical_alignment(Vertical::Center)
            .font(crate::fonts::TF2_SECONDARY)
            .size(16),
    )
}

fn favorite_button<'a, M: Clone + 'a>(is_favorite: bool, icons: &Icons, size: u16) -> Button<'a, M> {
    let icon = match is_favorite {
        true => icons.favorite(),
        false => icons.favorite_border(),
    };

    svg_button(icon, size)
}

pub fn header_view<'a>(title: &str, icons: &Icons, edit_favorites: bool) -> Element<'a, Messages> {
    row![
        text(title).font(crate::fonts::TF2_BUILD).size(BIG_BUTTON_SIZE),
        horizontal_space(iced::Length::Fill),
        svg_button(icons.refresh(), BIG_BUTTON_SIZE).on_press(Messages::RefreshServers),
        favorite_button(edit_favorites, icons, BIG_BUTTON_SIZE).on_press(Messages::EditFavorites(!edit_favorites))
    ]
    .spacing(VISUAL_SPACING_SMALL)
    .into()
}

pub fn filter_view<'a>(text: &str, icons: &Icons) -> Element<'a, Messages> {
    row![
        text_input("Filter", text, Messages::FilterChanged),
        svg_button(icons.clear(), 28).on_press(Messages::FilterChanged(String::new())),
    ]
    .align_items(iced::Alignment::Center)
    .spacing(VISUAL_SPACING_SMALL)
    .into()
}

pub fn servers_view<'a, I: Iterator<Item = &'a Server>>(
    servers_iterator: I,
    icons: &Icons,
    favorite_servers: &BTreeSet<String>,
    edit_favorites: bool,
) -> Element<'a, Messages> {
    scrollable(
        servers_iterator
            .fold(Column::new().spacing(VISUAL_SPACING_SMALL), |column, server| {
                column.push(server_view(
                    server,
                    favorite_servers.contains(&server.name),
                    icons,
                    edit_favorites,
                ))
            })
            .width(Length::Fill)
            .padding([0, 8, 0, 0]),
    )
    .scrollbar_width(8)
    .scroller_width(8)
    .into()
}

fn server_view_buttons<'a>(
    server: &Server,
    is_favorite: bool,
    icons: &Icons,
    edit_favorites: bool,
) -> Row<'a, Messages> {
    if edit_favorites {
        row![favorite_button(is_favorite, icons, 32).on_press(Messages::FavoriteClicked(server.name.clone())),]
    } else {
        row![
            svg_button(icons.copy(), 28).on_press(Messages::CopyToClipboard(format!(
                "connect {}:{}",
                server.ip, server.port
            ))),
            text_button("Launch!").on_press(Messages::StartGame(server.into())),
        ]
    }
}

fn server_view<'a>(
    server: &Server,
    is_favorite: bool,
    icons: &Icons,
    edit_favorites: bool,
) -> Element<'a, Messages> {
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

pub fn refreshing_view<'a>() -> Element<'a, Messages> {
    text("Reloading")
        .width(Length::Fill)
        .height(Length::Fill)
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center)
        .font(fonts::TF2_SECONDARY)
        .size(40)
        .into()
}
