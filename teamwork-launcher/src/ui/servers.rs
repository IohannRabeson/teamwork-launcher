use {
    super::{favorite_button, svg_button, text_button, VISUAL_SPACING_MEDIUM, VISUAL_SPACING_SMALL},
    crate::{
        application::Messages, icons::Icons, models::Server, promised_value::PromisedValue, settings::UserSettings,
        ui::widgets, ui::advanced_filter::advanced_filter_view, fonts,
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
    icons: &'a Icons,
    settings: &'a UserSettings,
) -> Element<'a, Messages> {
    row![column![
        servers_text_filter_view(&settings.servers_filter_text(), icons),
        vertical_space(Length::Units(VISUAL_SPACING_SMALL)),
        servers_view_generic(servers_iterator, server_view_edit_favorites, settings, icons),
    ]]
    .spacing(VISUAL_SPACING_SMALL)
    .into()
}

pub fn servers_view<'a, I: Iterator<Item = &'a Server>>(
    servers_iterator: I,
    icons: &'a Icons,
    settings: &'a UserSettings,
) -> Element<'a, Messages> {
    let server_view_fn = |server, _, icons| server_view(server, icons);

    column![
        servers_text_filter_view(&settings.servers_filter_text(), icons),
        vertical_space(Length::Units(VISUAL_SPACING_SMALL)),
        servers_view_generic(servers_iterator, server_view_fn, settings, icons),
    ]
    .into()
}

/// Draw the widget to display a list of servers.
/// The parameter server_view_fn is a function that create the view for one server in the list.
fn servers_view_generic<'a, I, F>(
    servers_iterator: I,
    server_view_fn: F,
    settings: &'a UserSettings,
    icons: &'a Icons,
) -> Element<'a, Messages>
where
    I: Iterator<Item = &'a Server>,
    F: Fn(&'a Server, bool, &'a Icons) -> Element<'a, Messages>,
{
    row![
        container(
            scrollable(
                servers_iterator
                    .unique_by(|server| &server.ip_port)
                    .fold(Column::new().spacing(VISUAL_SPACING_SMALL), |column, server| {
                        column.push(server_view_fn(server, settings.filter_servers_favorite(server), icons))
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

fn servers_text_filter_view<'a>(text: &str, icons: &Icons) -> Element<'a, Messages> {
    let mut button = svg_button(icons.clear(), 28);

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

fn server_view_edit_favorites<'a>(server: &Server, is_favorite: bool, icons: &Icons) -> Element<'a, Messages> {
    container(row![
        widgets::thumbnail(server, icons),
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
                widgets::region(server, icons, SMALL_FLAG_SIZE, 0),
            ]
            .spacing(VISUAL_SPACING_SMALL),
            column![
                favorite_button(is_favorite, icons, 28)
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

fn server_view<'a>(server: &Server, icons: &Icons) -> Element<'a, Messages> {
    let server_name_row = match &server.country {
        PromisedValue::Ready(country) => row![
            widgets::country_icon(icons, country, TITLE_FONT_SIZE, VISUAL_SPACING_SMALL),
            horizontal_space(Length::Units(VISUAL_SPACING_SMALL)),
            text(&server.name).size(TITLE_FONT_SIZE)
        ],
        _ => row![text(&server.name).size(TITLE_FONT_SIZE)],
    };

    container(row![
        widgets::thumbnail(server, icons),
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
            svg_button(icons.copy(), 28).on_press(Messages::CopyToClipboard(server.ip_port.steam_connection_string())),
            text_button("Play").on_press(Messages::StartGame(server.ip_port.clone())),
        ]
        .align_items(Alignment::End)
        .spacing(VISUAL_SPACING_SMALL)
    ])
    .padding(6)
    .into()
}
