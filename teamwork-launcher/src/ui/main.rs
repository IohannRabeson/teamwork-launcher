use {
    crate::{
        application::{
            filter::filter_servers::Filter,
            game_mode::GameModes,
            progress::Progress,
            screens::{PaneId, PaneView},
            servers_counts::ServersCounts,
            Bookmarks, FilterMessage, Message, PaneMessage, PromisedValue, Server,
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
        theme,
        widget::{self, column, container, horizontal_space, pane_grid, row, text, toggler, Container, Image, PaneGrid},
        Alignment, Element, Length,
    },
    iced_aw::Spinner,
    iced_lazy::responsive,
    iced_native::widget::{
        progress_bar,
        scrollable::{self, RelativeOffset},
    },
};
use crate::application::ViewMode;
use crate::ui::THUMBNAIL_CONTENT_FIT;

pub struct ViewContext<'l> {
    pub panes: &'l pane_grid::State<PaneView>,
    pub panes_split: &'l pane_grid::Split,
    pub servers: &'l [Server],
    pub bookmarks: &'l Bookmarks,
    pub filter: &'l Filter,
    pub game_modes: &'l GameModes,
    pub counts: &'l ServersCounts,
    pub servers_list: &'l ServersList,
    pub progress: &'l Progress,
    pub is_loading: bool,
    pub servers_list_view_mode: ViewMode,
}

pub fn view(context: ViewContext) -> Element<Message> {
    let textual_filters = container(ui::filter::text_filter(context.filter)).padding([0, 8]);
    let pane_grid = PaneGrid::new(context.panes, |_id, pane, _is_maximized| {
        pane_grid::Content::new(responsive(move |_size| match &pane.id {
            PaneId::Servers => match context.is_loading {
                false => servers_view(
                    context.servers,
                    context.bookmarks,
                    context.filter,
                    context.game_modes,
                    context.servers_list,
                    context.servers_list_view_mode,
                ),
                true => container(Spinner::new().width(Length::Fixed(20.0)).height(Length::Fixed(20.0)))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into(),
            },
            PaneId::Filters => filter_view(context.filter, context.game_modes, context.counts),
        }))
    })
    .on_resize(10, |e| Message::Pane(PaneMessage::Resized(e)));

    let mut main_column = column![textual_filters, pane_grid];

    if !context.progress.is_finished() {
        let progress_bar = progress_bar(0.0f32..=1.0f32, context.progress.current_progress()).height(Length::Fixed(4.0));

        main_column = main_column.push(progress_bar);
    }

    main_column.spacing(4).into()
}

/// The wide view that displays server information.
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
                    svg_button(icons::INFO_ICON.clone(), BUTTON_SIZE)
                        .on_press(Message::ShowServer(server.ip_port.clone(), server.map.clone())),
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
                        row![text("Region:"), region(&server.country, BUTTON_SIZE, 0)].spacing(4),
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
    .style(theme::Container::Custom(Box::new(BoxContainerStyle)))
    .into()
}

/// This is the compact view that displays server information.
fn compact_server_view<'l>(server: &'l Server, bookmarks: &'l Bookmarks, game_modes: &'l GameModes) -> Element<'l, Message> {
    let is_bookmarked = bookmarks.is_bookmarked(&server.ip_port);
    let ip_port_text = format!("{}:{}", server.ip_port.ip(), server.ip_port.port());
    let game_modes = widgets::game_modes(game_modes, &server.game_modes);

    const BUTTON_SIZE: u16 = 20;

    let thumbnail: Element<'l, Message> = match &server.map_thumbnail {
        PromisedValue::Ready(image) => Image::new(image.clone()).width(Length::Fill).content_fit(THUMBNAIL_CONTENT_FIT).into(),
        PromisedValue::Loading => container(Spinner::new().width(Length::Fixed(32.0)).height(Length::Fixed(32.0)))
            .width(Length::Fill)
            .height(Length::Fixed(250.0))
            .center_x()
            .center_y()
            .into(),
        PromisedValue::None => container(Image::new(icons::NO_IMAGE.clone()))
            .width(Length::Fill)
            .height(Length::Fixed(250.0))
            .center_x()
            .center_y()
            .into(),
    };

    container(
        column![
            row![
                thumbnail,
                column![
                    svg_button(icons::INFO_ICON.clone(), BUTTON_SIZE)
                        .on_press(Message::ShowServer(server.ip_port.clone(), server.map.clone())),
                    favorite_button(is_bookmarked, BUTTON_SIZE)
                        .on_press(Message::Bookmarked(server.ip_port.clone(), !is_bookmarked)),
                    svg_button(icons::COPY_ICON.clone(), BUTTON_SIZE)
                        .on_press(Message::CopyConnectionString(server.ip_port.clone())),
                    svg_button(icons::PLAY_ICON.clone(), BUTTON_SIZE).on_press(Message::LaunchGame(server.ip_port.clone())),
                ]
                .spacing(4)
            ]
            .spacing(4),
            text(&server.name).size(28).width(Length::Fill),
            row![
                text(&ip_port_text),
                svg_button(icons::COPY_ICON.clone(), 10).on_press(Message::CopyToClipboard(ip_port_text)),
                horizontal_space(Length::Fill),
                text("Region:"),
                region(&server.country, BUTTON_SIZE, 0)
            ]
            .spacing(4),
            row![
                text(&server.map),
                horizontal_space(Length::Fill),
                text(&format!(
                    "Players: {} / {}",
                    server.current_players_count, server.max_players_count
                ))
            ]
            .spacing(8),
            row![game_modes, horizontal_space(Length::Fill), text("Ping:"), ping(&server.ping),].spacing(8)
        ]
        .spacing(4),
    )
    .padding(8)
    .style(theme::Container::Custom(Box::new(BoxContainerStyle)))
    .into()
}

fn servers_view<'l>(
    servers: &'l [Server],
    bookmarks: &'l Bookmarks,
    filter: &'l Filter,
    game_modes: &'l GameModes,
    servers_list: &'l ServersList,
    servers_list_view_mode: ViewMode,
) -> Element<'l, Message> {
    let server_view_fn = match servers_list_view_mode {
        ViewMode::Normal => server_view,
        ViewMode::Compact => compact_server_view,
    };
    let servers = servers.iter().filter(|server| filter.accept(server, bookmarks));
    let servers_list = container(
        widget::scrollable(servers.fold(column![], |c, server| {
            c.push(
                container((server_view_fn)(server, bookmarks, game_modes))
                    /* <- THIS IS TO PREVENT THE SCROLLBAR TO OVERLAP THE VIEW */
                    .padding([4, 16, 4, 8]),
            )
        }))
        .on_scroll(Message::ServerListScroll)
        .id(servers_list.id.clone()),
    )
    .height(Length::Fill)
    .width(Length::Fill);

    servers_list.into()
}

fn filter_view<'l>(filter: &'l Filter, game_modes: &'l GameModes, counts: &'l ServersCounts) -> Element<'l, Message> {
    let filter_panel = container(widget::scrollable(
        column![
            filter_section(Some("Sort"), ui::filter::server_sort(filter)),
            filter_section(None, ui::filter::bookmark_filter(filter, counts)),
            filter_section_with_switch(
                Some("Ping filter"),
                ui::filter::ping_filter(filter, counts),
                filter.ping.enabled,
                |checked| Message::Filter(FilterMessage::PingFilterEnabled(checked))
            ),
            filter_section_with_switch(
                Some("Players filter"),
                ui::filter::players_filter(filter),
                filter.players.enabled,
                |checked| Message::Filter(FilterMessage::PlayerFilterEnabled(checked)),
            ),
            filter_section(Some("Text filter"), ui::filter::text_filter_options(filter)),
            filter_section(None, ui::filter::server_properties_filter(filter, counts)),
            filter_section_with_switch(
                Some("Maps filter"),
                ui::filter::maps_filter(filter, counts),
                filter.maps.enabled,
                |checked| Message::Filter(FilterMessage::MapFilterEnabled(checked))
            ),
            filter_section_with_switch(
                Some("Game modes filter"),
                ui::filter::game_modes_filter(filter, game_modes, counts),
                filter.game_modes.enabled,
                |checked| Message::Filter(FilterMessage::GameModeFilterEnabled(checked))
            ),
            filter_section_with_switch(
                Some("Countries filter"),
                ui::filter::country_filter(filter, counts),
                filter.country.enabled,
                |checked| Message::Filter(FilterMessage::CountryFilterEnabled(checked))
            ),
            filter_section_with_switch(
                Some("Providers filter"),
                ui::filter::providers_filter(filter, counts),
                filter.providers.enabled,
                |checked| Message::Filter(FilterMessage::ProviderFilterEnabled(checked))
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
    .style(theme::Container::Custom(Box::new(BoxContainerStyle)))
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
        .style(theme::Container::Custom(Box::new(BoxContainerStyle)))
        .padding(8)
}

pub struct ServersList {
    pub scroll_position: RelativeOffset,
    pub id: scrollable::Id,
}

impl ServersList {
    pub fn new() -> Self {
        Self {
            scroll_position: RelativeOffset::START,
            id: scrollable::Id::unique(),
        }
    }
}
