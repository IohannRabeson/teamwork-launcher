use {
    super::{favorite_button, svg_button, VISUAL_SPACING_MEDIUM, VISUAL_SPACING_SMALL},
    crate::{
        application::Messages,
        fonts, icons,
        models::Server,
        promised_value::PromisedValue,
        settings::UserSettings,
        ui::{advanced_filter::advanced_filter_view, widgets, SMALL_BUTTON_SIZE},
    },
    iced::{
        widget::{column, container, horizontal_space, row, scrollable, text, text_input, vertical_space, Column},
        Alignment, Element, Length,
    },
    itertools::Itertools,
};

/// View displaying the editor for favorites servers.
/// It displays the list of sources and allows the users to choose which ones to use.
/// Note for myself: stop trying to factorize this view and the list of favorites servers because they are really different.
pub fn servers_view_edit_favorites<'a, I: Iterator<Item = &'a Server>>(
    servers_iterator: I,
    settings: &'a UserSettings,
) -> Element<'a, Messages> {
    row![column![
        servers_text_filter_view(&settings.servers_filter_text()),
        vertical_space(Length::Units(VISUAL_SPACING_SMALL)),
        servers_view_generic(servers_iterator, server_view_edit_favorites, settings),
    ]]
    .spacing(VISUAL_SPACING_SMALL)
    .into()
}

pub fn servers_view<'a, I: Iterator<Item = &'a Server>>(
    servers_iterator: I,
    settings: &'a UserSettings,
) -> Element<'a, Messages> {
    let server_view_fn = |server, _| server_view(server);

    column![
        servers_text_filter_view(&settings.servers_filter_text()),
        vertical_space(Length::Units(VISUAL_SPACING_SMALL)),
        servers_view_generic(servers_iterator, server_view_fn, settings),
    ]
    .into()
}

/// Draw the widget to display a list of servers.
/// The parameter server_view_fn is a function that create the view for one server in the list.
fn servers_view_generic<'a, I, F>(
    servers_iterator: I,
    server_view_fn: F,
    settings: &'a UserSettings,
) -> Element<'a, Messages>
where
    I: Iterator<Item = &'a Server>,
    F: Fn(&'a Server, bool) -> Element<'a, Messages>,
{
    row![
        container(
            scrollable(
                servers_iterator
                    .unique_by(|server| &server.ip_port)
                    .fold(Column::new().spacing(VISUAL_SPACING_SMALL), |column, server| {
                        column.push(server_view_fn(server, settings.filter_servers_favorite(server)))
                    })
                    .width(Length::Fill)
                    .padding([0, VISUAL_SPACING_MEDIUM, 0, 0]),
            )
            .scrollbar_width(8)
            .scroller_width(8)
        )
        .width(Length::FillPortion(4)),
        advanced_filter_view(settings).width(Length::FillPortion(1))
    ]
    .into()
}

fn servers_text_filter_view<'a>(text: &str) -> Element<'a, Messages> {
    let mut button = svg_button(icons::CLEAR_ICON.clone(), SMALL_BUTTON_SIZE);

    // Enable the clear button only if the field contains text.
    if !text.is_empty() {
        button = button.on_press(Messages::FilterChanged(String::new()));
    }

    row![text_input("Filter servers", text, Messages::FilterChanged), button,]
        .align_items(iced::Alignment::Center)
        .spacing(VISUAL_SPACING_SMALL)
        .padding([0, VISUAL_SPACING_SMALL])
        .into()
}

const TITLE_FONT_SIZE: u16 = 24;
const SMALL_FLAG_SIZE: u16 = 20;

fn server_view_edit_favorites<'a>(server: &Server, is_favorite: bool) -> Element<'a, Messages> {
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
            ]
            .spacing(VISUAL_SPACING_SMALL),
            column![
                favorite_button(is_favorite, SMALL_BUTTON_SIZE)
                    .on_press(Messages::FavoriteClicked(server.ip_port.clone(), server.source.clone())),
                widgets::ping(server).size(fonts::TEXT_FONT_SIZE)
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

fn server_view<'a>(server: &Server) -> Element<'a, Messages> {
    let server_name_row = match &server.country {
        PromisedValue::Ready(country) => row![
            widgets::country_icon(country, TITLE_FONT_SIZE, VISUAL_SPACING_SMALL),
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
            widgets::ping(server).size(fonts::TEXT_FONT_SIZE),
        ]
        .spacing(VISUAL_SPACING_SMALL),
        horizontal_space(Length::Fill),
        row![
            widgets::tooltip(
                svg_button(icons::COPY_ICON.clone(), SMALL_BUTTON_SIZE).on_press(Messages::CopyToClipboard(server.ip_port.steam_connection_string())),
                &format!("Copy to clipboard the connection string \"{}\"", server.ip_port.steam_connection_string()),
                iced::widget::tooltip::Position::Bottom,
            ),
            widgets::tooltip(
                svg_button(icons::PLAY_ICON.clone(), SMALL_BUTTON_SIZE).on_press(Messages::StartGame(server.ip_port.clone())),
                "Start Team Fortress 2 and connect to this server",
                iced::widget::tooltip::Position::Bottom,
            ),
        ]
        .align_items(Alignment::End)
        .spacing(VISUAL_SPACING_SMALL)
    ])
    .padding(6)
    .into()
}
