use iced::{
    widget::{button, container},
    Length,
};

use crate::{fonts, models::Server};

use {
    super::{favorite_button, svg_button, text_button, VISUAL_SPACING_SMALL},
    crate::{application::Messages, icons::Icons, settings::UserSettings},
    iced::{
        widget::{column, horizontal_space, row, scrollable, text, text_input, vertical_space, Column, Row},
        Alignment, Element,
    },
    itertools::Itertools,
};

pub fn servers_view<'a, I: Iterator<Item = &'a Server>>(
    servers_iterator: I,
    icons: &Icons,
    settings: &UserSettings,
    edit_favorites: bool,
) -> Element<'a, Messages> {
    let servers: Vec<&Server> = servers_iterator.collect();

    if servers.is_empty() && !edit_favorites {
        return no_favorite_servers_view();
    }

    column![
        servers_filter_view(&settings.servers_filter_text(), icons),
        vertical_space(Length::Units(VISUAL_SPACING_SMALL)),
        scrollable(
            servers
                .into_iter()
                .unique_by(|server| &server.ip_port)
                .fold(Column::new().spacing(VISUAL_SPACING_SMALL), |column, server| {
                    column.push(server_view(
                        server,
                        settings.filter_servers_favorite(&server),
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

fn no_favorite_servers_view<'a>() -> Element<'a, Messages> {
    container(
        column![
            text("No favorite servers!").font(fonts::TF2_SECONDARY).size(36),
            text("You can edit the list of your favorite servers by clicking on the star button on the top right of the window."),
            button("Edit favorite servers").on_press(Messages::EditFavorites),
        ]
        .align_items(Alignment::Center)
        .spacing(12),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .into()
}

fn servers_filter_view<'a>(text: &str, icons: &Icons) -> Element<'a, Messages> {
    row![
        text_input("Filter servers", text, Messages::FilterChanged),
        svg_button(icons.clear(), 28).on_press(Messages::FilterChanged(String::new())),
    ]
    .align_items(iced::Alignment::Center)
    .spacing(VISUAL_SPACING_SMALL)
    .padding([0, VISUAL_SPACING_SMALL])
    .into()
}

fn server_view_buttons<'a>(server: &Server, is_favorite: bool, icons: &Icons, edit_favorites: bool) -> Row<'a, Messages> {
    if edit_favorites {
        row![favorite_button(is_favorite, icons, 32)
            .on_press(Messages::FavoriteClicked(server.ip_port.clone(), server.source.clone())),]
    } else {
        row![
            svg_button(icons.copy(), 28).on_press(Messages::CopyToClipboard(server.ip_port.steam_connection_string())),
            text_button("Play").on_press(Messages::StartGame(server.ip_port.clone())),
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
