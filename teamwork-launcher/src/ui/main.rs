use {
    crate::{
        application::{
            Bookmarks, Filter, FilterMessage, game_mode::GameModes, MainView, Message, PaneId, PaneMessage, Server,
        },
        icons,
        ui::{
            self,
            buttons::{favorite_button, svg_button},
            styles::BoxContainerStyle,
            widgets::{self, ping, region, thumbnail},
        },
    },
    iced::{
        Alignment,
        Element,
        Length, theme, widget::{column, container, Container, horizontal_space, pane_grid, PaneGrid, row, scrollable, text, toggler},
    },
    iced_lazy::responsive,
};
use crate::application::servers_counts::ServersCounts;

pub fn view<'l>(
    view: &'l MainView,
    servers: &'l [Server],
    bookmarks: &'l Bookmarks,
    filter: &'l Filter,
    game_modes: &'l GameModes,
    counts: &'l ServersCounts,
) -> Element<'l, Message> {
    let textual_filters = container(ui::filter::text_filter(filter)).padding([0, 8]);
    let pane_grid = PaneGrid::new(&view.panes, |_id, pane, _is_maximized| {
        pane_grid::Content::new(responsive(move |_size| match &pane.id {
            PaneId::Servers => servers_view(servers, bookmarks, filter, game_modes).into(),
            PaneId::Filters => filter_view(filter, game_modes, counts).into(),
        }))
    })
    .on_resize(10, |e| Message::Pane(PaneMessage::Resized(e)));

    column![textual_filters, pane_grid,].padding([8, 0]).spacing(4).into()
}

fn server_view<'l>(server: &'l Server, bookmarks: &'l Bookmarks, game_modes: &'l GameModes) -> Element<'l, Message> {
    let is_bookmarked = bookmarks.is_bookmarked(&server.ip_port);
    let ip_port_text = format!("{}:{}", server.ip_port.ip(), server.ip_port.port());
    let game_modes = widgets::game_modes(game_modes, &server.game_modes);

    const BUTTON_SIZE: u16 = 20;

    container(
        row![
            thumbnail(server, Length::Fixed(250.0), Length::Fixed(125.0)),
            column![
                row![
                    text(&server.name).size(28).width(Length::Fill),
                    svg_button(icons::INFO_ICON.clone(), BUTTON_SIZE).on_press(Message::ShowServer(server.ip_port.clone())),
                    favorite_button(is_bookmarked, BUTTON_SIZE)
                        .on_press(Message::Bookmarked(server.ip_port.clone(), !is_bookmarked)),
                    svg_button(icons::COPY_ICON.clone(), BUTTON_SIZE)
                        .on_press(Message::CopyConnectionString(server.ip_port.clone())),
                    svg_button(icons::PLAY_ICON.clone(), BUTTON_SIZE).on_press(Message::LaunchGame(server.ip_port.clone())),
                ]
                .padding(4)
                .spacing(4),
                row![
                    column![
                        row![
                            text(&ip_port_text),
                            svg_button(icons::COPY_ICON.clone(), 10).on_press(Message::CopyToClipboard(ip_port_text)),
                        ]
                        .spacing(4),
                        text(&server.map),
                        text(&format!("{} / {}", server.current_players_count, server.max_players_count)),
                    ]
                    .spacing(4),
                    horizontal_space(Length::Fill),
                    column![
                        region(server, BUTTON_SIZE, 0),
                        row![text("Ping:"), ping(&server.ping)].spacing(4),
                        game_modes
                    ]
                    .padding(4)
                    .spacing(4)
                    .align_items(Alignment::End)
                ]
            ],
        ]
        .spacing(4),
    )
    .padding(8)
    .style(theme::Container::Custom(Box::new(BoxContainerStyle {})))
    .into()
}

fn servers_view<'l>(
    servers: &'l [Server],
    bookmarks: &'l Bookmarks,
    filter: &'l Filter,
    game_modes: &'l GameModes,
) -> Element<'l, Message> {
    let servers = servers.iter().filter(|server| filter.accept(server, bookmarks));
    let servers_list = container(scrollable(servers.fold(column![], |c, server| {
        c.push(
            container(server_view(server, bookmarks, game_modes))
                .padding([4, 24 /* <- THIS IS TO PREVENT THE SCROLLBAR TO COVER THE VIEW */, 4, 8]),
        )
    })))
    .height(Length::Fill)
    .width(Length::Fill);

    servers_list.into()
}

fn filter_view<'l>(filter: &'l Filter, game_modes: &'l GameModes, counts: &'l ServersCounts) -> Element<'l, Message> {
    let filter_panel = container(scrollable(
        column![
            filter_section(Some("Sort"), ui::filter::server_sort(filter)),
            filter_section(None, ui::filter::bookmark_filter(filter, counts)),
            filter_section(Some("Ping filter"), ui::filter::ping_filter(filter, counts)),
            filter_section(Some("Players filter"), ui::filter::players_filter(filter)),
            filter_section(Some("Text filter"), ui::filter::text_filter_options(filter)),
            filter_section(None, ui::filter::server_properties_filter(filter, counts)),
            filter_section_with_switch(Some("Maps filter"),
                ui::filter::maps_filter(filter, counts),
                filter.maps.enabled,
                |checked| Message::Filter(FilterMessage::MapFilterEnabled(checked))
            ),
            filter_section_with_switch(
                Some("Game modes filter"),
                ui::filter::game_modes_filter(filter, game_modes, counts),
                filter.game_modes.is_enabled(),
                |checked| Message::Filter(FilterMessage::GameModeFilterEnabled(checked))
            ),
            filter_section_with_switch(
                Some("Countries filter"),
                ui::filter::country_filter(filter, counts),
                filter.country.is_enabled(),
                |checked| Message::Filter(FilterMessage::CountryFilterEnabled(checked))
            ),
        ]
        .padding([0, 14, 0, 0])
        .spacing(4),
    ))
    .padding(4);

    filter_panel.into()
}

fn filter_section<'l>(title: Option<&str>, content: impl Into<Element<'l, Message>>) -> Container<'l, Message> {
    container(
        match title {
            None => {
                column![content.into()]
            }
            Some(title) => {
                column![text(title), content.into()]
            }
        }
        .spacing(8),
    )
    .width(Length::Fill)
    .style(theme::Container::Custom(Box::new(BoxContainerStyle {})))
    .padding(8)
}

fn filter_section_with_switch<'l>(
    title: Option<&str>,
    content: impl Into<Element<'l, Message>>,
    enabled: bool,
    f: impl Fn(bool) -> Message + 'l,
) -> Container<'l, Message> {
    let switch = toggler(title.map(|s| s.to_string()), enabled, f);
    let mut main_column = column![switch];

    if enabled {
        main_column = main_column.push(content.into());
    }

    container(main_column.spacing(8))
        .width(Length::Fill)
        .style(theme::Container::Custom(Box::new(BoxContainerStyle {})))
        .padding(8)
}
