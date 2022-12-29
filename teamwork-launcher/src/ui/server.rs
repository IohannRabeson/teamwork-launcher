/// This is the implementation of the widget that displays
/// information about one server, with the map thumbnail and
/// the buttons to start the game.
use {
    super::{favorite_button, svg_button, SMALL_BUTTON_SIZE},
    crate::{
        application::Messages,
        fonts, icons,
        models::Server,
        promised_value::PromisedValue,
        settings::UserSettings,
        ui::{widgets, VISUAL_SPACING_SMALL},
    },
    iced::{
        widget::{column, container, horizontal_space, row, text},
        Alignment, Element, Length,
    },
};
use {
    crate::servers_provider::ServersProvider,
    iced::widget::{Row, Text},
};

const TITLE_FONT_SIZE: u16 = 24;
const SMALL_FLAG_SIZE: u16 = 20;

pub fn server_view<'a>(
    server: &Server,
    settings: &UserSettings,
    servers_provider: &ServersProvider,
) -> Element<'a, Messages> {
    let server_name_row = match &server.country {
        PromisedValue::Ready(country) => row![
            widgets::country_icon(country, TITLE_FONT_SIZE, 1),
            horizontal_space(Length::Units(VISUAL_SPACING_SMALL)),
            text(&server.name).size(TITLE_FONT_SIZE)
        ],
        _ => row![text(&server.name).size(TITLE_FONT_SIZE)],
    };

    container(row![
        widgets::thumbnail(server),
        column![
            server_name_row,
            text(format!(
                "Players: {} / {}",
                server.current_players_count, server.max_players_count
            ))
            .size(fonts::TEXT_FONT_SIZE),
            text(format!("Map: {}", server.map)).size(fonts::TEXT_FONT_SIZE),
            widgets::ping(server),
            source_text(server, servers_provider).size(fonts::TEXT_FONT_SIZE)
        ]
        .spacing(VISUAL_SPACING_SMALL),
        horizontal_space(Length::Fill),
        buttons_row(server, settings)
            .align_items(Alignment::End)
            .spacing(VISUAL_SPACING_SMALL)
    ])
    .padding(6)
    .into()
}

fn source_text<'a>(server: &Server, servers_provider: &ServersProvider) -> Text<'a> {
    text(format!(
        "Source: {}",
        server
            .source
            .as_ref()
            .and_then(|key| { servers_provider.get_source_name(key) })
            .unwrap_or_else(|| " - ".into())
    ))
}

fn buttons_row<'a>(server: &Server, settings: &UserSettings) -> Row<'a, Messages> {
    let mut copy_tooltip_text = "Copy to clipboard the connection string.".to_string();

    if settings.quit_on_copy() {
        copy_tooltip_text += "\nThe launcher will quit."
    }

    let mut start_tooltip_text = String::from("Start Team Fortress 2 and connect to this server.");

    if settings.quit_on_launch() {
        start_tooltip_text += "\nThe launcher will quit."
    }

    row![
        widgets::tooltip(
            svg_button(icons::CLEAR_ICON.clone(), SMALL_BUTTON_SIZE)
                .on_press(Messages::FavoriteClicked(server.ip_port.clone(), None)),
            "Remove this server from favorites",
            iced::widget::tooltip::Position::Bottom,
        ),
        widgets::tooltip(
            svg_button(icons::COPY_ICON.clone(), SMALL_BUTTON_SIZE)
                .on_press(Messages::CopyToClipboard(server.ip_port.steam_connection_string())),
            copy_tooltip_text,
            iced::widget::tooltip::Position::Bottom,
        ),
        widgets::tooltip(
            svg_button(icons::PLAY_ICON.clone(), SMALL_BUTTON_SIZE).on_press(Messages::StartGame(server.ip_port.clone())),
            start_tooltip_text,
            iced::widget::tooltip::Position::Bottom,
        ),
    ]
}

pub fn server_view_edit_favorites<'a>(
    server: &Server,
    servers_provider: &'a ServersProvider,
    is_favorite: bool,
) -> Element<'a, Messages> {
    container(row![
        widgets::thumbnail(server),
        horizontal_space(Length::Units(VISUAL_SPACING_SMALL)),
        column![row![
            column![
                text(&server.name).size(TITLE_FONT_SIZE),
                text(format!(
                    "Players: {} / {}",
                    server.current_players_count, server.max_players_count
                ))
                .size(fonts::TEXT_FONT_SIZE),
                text(format!("Map: {}", server.map)).size(fonts::TEXT_FONT_SIZE),
                widgets::region(server, SMALL_FLAG_SIZE, 0),
                source_text(server, servers_provider).size(fonts::TEXT_FONT_SIZE)
            ]
            .spacing(VISUAL_SPACING_SMALL),
            column![
                favorite_button(is_favorite, SMALL_BUTTON_SIZE)
                    .on_press(Messages::FavoriteClicked(server.ip_port.clone(), server.source.clone())),
                widgets::ping(server)
            ]
            .spacing(VISUAL_SPACING_SMALL)
            .width(Length::Fill)
            .align_items(Alignment::End),
        ]],
        horizontal_space(Length::Fill),
    ])
    .padding(6)
    .into()
}
