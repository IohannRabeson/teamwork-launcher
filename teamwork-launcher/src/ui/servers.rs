use {
    super::{svg_button, VISUAL_SPACING_MEDIUM, VISUAL_SPACING_SMALL},
    crate::{
        application::Messages,
        icons,
        models::Server,
        servers_provider::ServersProvider,
        settings::UserSettings,
        ui::{
            advanced_filter::advanced_filter_view,
            server::{server_view, server_view_edit_favorites},
            SMALL_BUTTON_SIZE,
        },
    },
    iced::{
        widget::{column, container, row, scrollable, text_input, vertical_space, Column},
        Element, Length,
    },
    itertools::Itertools,
};

/// View displaying the editor for favorites servers.
/// It displays the list of sources and allows the users to choose which ones to use.
/// Note for myself: stop trying to factorize this view and the list of favorites servers because they are really different.
pub fn servers_view_edit_favorites<'a, I: Iterator<Item = &'a Server>>(
    servers_iterator: I,
    settings: &'a UserSettings,
    servers_provider: &'a ServersProvider,
) -> Element<'a, Messages> {
    row![column![
        servers_text_filter_view(&settings.servers_filter_text()),
        vertical_space(Length::Units(VISUAL_SPACING_SMALL)),
        servers_view_generic(servers_iterator, server_view_edit_favorites, settings, servers_provider),
    ]]
    .spacing(VISUAL_SPACING_SMALL)
    .into()
}

pub fn servers_view<'a, I: Iterator<Item = &'a Server>>(
    servers_iterator: I,
    settings: &'a UserSettings,
    servers_provider: &'a ServersProvider,
) -> Element<'a, Messages> {
    let server_view_fn = |server, _, _| server_view(server, settings, servers_provider);

    column![
        servers_text_filter_view(&settings.servers_filter_text()),
        vertical_space(Length::Units(VISUAL_SPACING_SMALL)),
        servers_view_generic(servers_iterator, server_view_fn, settings, servers_provider),
    ]
    .into()
}

/// Draw the widget to display a list of servers.
/// The parameter server_view_fn is a function that create the view for one server in the list.
fn servers_view_generic<'a, I, F>(
    servers_iterator: I,
    server_view_fn: F,
    settings: &'a UserSettings,
    servers_provider: &'a ServersProvider,
) -> Element<'a, Messages>
where
    I: Iterator<Item = &'a Server>,
    F: Fn(&'a Server, &'a ServersProvider, bool) -> Element<'a, Messages>,
{
    row![
        container(
            scrollable(
                servers_iterator
                    .unique_by(|server| &server.ip_port)
                    .fold(Column::new().spacing(VISUAL_SPACING_SMALL), |column, server| {
                        column.push(server_view_fn(
                            server,
                            servers_provider,
                            settings.filter_servers_favorite(server),
                        ))
                    })
                    .width(Length::Fill)
                    .padding([0, VISUAL_SPACING_MEDIUM, 0, 0]),
            )
            .vertical_scroll(scrollable::Properties::new().width(8).scroller_width(8))
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
