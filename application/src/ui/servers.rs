use {
    super::{favorite_button, svg_button, text_button, VISUAL_SPACING_SMALL},
    crate::{application::Messages, fonts, icons::Icons, models::Server, settings::UserSettings},
    iced::{
        widget::{
            button, column, container, horizontal_space, row, scrollable, text, text_input, vertical_space, Column,
        },
        Alignment, Element, Length,
    },
    itertools::Itertools,
};

pub fn servers_view_edit_favorites<'a, I: Iterator<Item = &'a Server>>(
    servers_iterator: I,
    icons: &Icons,
    settings: &UserSettings,
) -> Element<'a, Messages> {
    column![
        servers_filter_view(&settings.servers_filter_text(), icons),
        vertical_space(Length::Units(VISUAL_SPACING_SMALL)),
        scrollable(
            servers_iterator
                .unique_by(|server| &server.ip_port)
                .fold(Column::new().spacing(VISUAL_SPACING_SMALL), |column, server| {
                    column.push(server_view_edit_favorites(
                        server,
                        settings.filter_servers_favorite(&server),
                        icons,
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

pub fn servers_view<'a, I: Iterator<Item = &'a Server>>(
    servers_iterator: I,
    icons: &Icons,
    settings: &UserSettings,
) -> Element<'a, Messages> {
    column![
        servers_filter_view(&settings.servers_filter_text(), icons),
        vertical_space(Length::Units(VISUAL_SPACING_SMALL)),
        scrollable(
            servers_iterator
                .unique_by(|server| &server.ip_port)
                .fold(Column::new().spacing(VISUAL_SPACING_SMALL), |column, server| {
                    column.push(server_view(server, icons))
                })
                .width(Length::Fill)
                .padding([0, 8, 0, 0]),
        )
        .scrollbar_width(8)
        .scroller_width(8)
    ]
    .into()
}

pub fn no_favorite_servers_view<'a>() -> Element<'a, Messages> {
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

fn server_view_edit_favorites<'a>(server: &Server, is_favorite: bool, icons: &Icons) -> Element<'a, Messages> {
    const BIG_FONT_SIZE: u16 = 32;

    container(row![
        widgets::thumbnail(server, icons),
        horizontal_space(Length::Units(VISUAL_SPACING_SMALL)),
        column![
            text(&server.name).size(BIG_FONT_SIZE),
            text(format!(
                "Players: {} / {}",
                server.current_players_count, server.max_players_count
            )),
            text(format!("Map: {}", server.map))
        ],
        horizontal_space(Length::Fill),
        row![favorite_button(is_favorite, icons, BIG_FONT_SIZE)
            .on_press(Messages::FavoriteClicked(server.ip_port.clone(), server.source.clone())),]
        .align_items(Alignment::End)
        .spacing(VISUAL_SPACING_SMALL)
    ])
    .padding(6)
    .into()
}

fn server_view<'a>(server: &Server, icons: &Icons) -> Element<'a, Messages> {
    const BIG_FONT_SIZE: u16 = 32;

    container(row![
        widgets::thumbnail(server, icons),
        horizontal_space(Length::Units(VISUAL_SPACING_SMALL)),
        column![
            text(&server.name).size(BIG_FONT_SIZE),
            text(format!(
                "Players: {} / {}",
                server.current_players_count, server.max_players_count
            )),
            text(format!("Map: {}", server.map))
        ],
        horizontal_space(Length::Fill),
        row![
            svg_button(icons.copy(), 28).on_press(Messages::CopyToClipboard(server.ip_port.steam_connection_string())),
            text_button("Play").on_press(Messages::StartGame(server.ip_port.clone())),
        ]
        .align_items(Alignment::End)
        .spacing(VISUAL_SPACING_SMALL)
    ])
    .padding(6)
    .into()
}

mod widgets {
    use iced::{widget::{container, image, text}, Element, Length};

    use crate::{models::{Thumbnail, Server}, icons::Icons, application::Messages};

    fn image_thumbnail_viewer<'a>(image: image::Handle) -> Element<'a, Messages> {
        image::viewer(image)
                .width(Length::Units(200))
                .height(Length::Units(100))
                .scale_step(0.0)
                .into()
    }
    
    // TODO: make a proper widget I guess?
    fn image_thumbnail_content<'a>(server: &Server, icons: &Icons) -> Element<'a, Messages> {
        match &server.map_thumbnail {
            Thumbnail::Ready(image) => image_thumbnail_viewer(image.clone()),
            Thumbnail::Loading => return text("Loading").into(),
            Thumbnail::None => image_thumbnail_viewer(icons.no_image()),
        }
    }
    
    pub fn thumbnail<'a>(server: &Server, icons: &Icons) -> Element<'a, Messages> {
        container(
            image_thumbnail_content(server, icons)
        )
        .width(Length::Units(200))
        .height(Length::Units(100))
        .center_x()
        .center_y()
        .into()
    }
}
