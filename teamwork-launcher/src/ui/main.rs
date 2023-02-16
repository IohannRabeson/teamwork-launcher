use {
    crate::{
        application::{
            game_mode::{GameMode, GameModeId, GameModes},
            Bookmarks, Filter, FilterMessage, MainView, Message, PaneId, PaneMessage, PaneView, PromisedValue, Server,
        },
        icons,
        ui::{
            self,
            buttons::{favorite_button, svg_button},
            widgets::{self, tooltip},
        },
    },
    iced::{
        theme,
        widget::{
            button, column, container, horizontal_space,
            pane_grid::{self, Pane},
            row, scrollable, text, text_input, toggler,
            tooltip::Position,
            Container, PaneGrid, Text,
        },
        Alignment, Color, Element, Length,
        Theme::{self, Dark},
    },
    iced_lazy::responsive,
};

pub fn view<'l>(
    view: &'l MainView,
    servers: &'l [Server],
    bookmarks: &'l Bookmarks,
    filter: &'l Filter,
    game_modes: &'l GameModes,
) -> Element<'l, Message> {
    let textual_filters = container(ui::filter::text_filter(filter)).padding([0, 8]);
    let pane_grid = PaneGrid::new(&view.panes, |id, pane, is_maximized| {
        pane_grid::Content::new(responsive(move |size| match &pane.id {
            PaneId::Servers => servers_view(servers, bookmarks, filter, game_modes),
            PaneId::Filters => filter_view(view, filter, game_modes),
        }))
    })
    .on_resize(10, |e| Message::Pane(PaneMessage::Resized(e)));

    column![textual_filters, pane_grid,].padding([8, 0]).spacing(4).into()
}

fn region<'a>(server: &Server, size: u16, padding: u16) -> Element<'a, Message> {
    match &server.country {
        PromisedValue::Ready(country) => row![
            text("Region:".to_string()),
            horizontal_space(Length::Units(4)),
            widgets::country_icon(country, size, padding)
        ]
        .into(),
        PromisedValue::Loading => text("Region: loading...").into(),
        PromisedValue::None => text("Region: unknown").into(),
    }
}

fn ping<'a>(server: &Server) -> Element<'a, Message> {
    match &server.ping {
        PromisedValue::Ready(duration) => row![text("Ping:"), widgets::ping_icon(duration, 20),]
            .spacing(4)
            .align_items(Alignment::End)
            .into(),
        PromisedValue::Loading => text("Ping: loading...").into(),
        PromisedValue::None => text("Ping: timeout").into(),
    }
}

fn game_mode_view_inner(game_mode: &GameMode) -> Element<Message> {
    match game_mode.color {
        Some(color) => text(&game_mode.title),
        None => text(&game_mode.title),
    }
    .size(16)
    .into()
}

struct GameModeStyle {
    color: Color,
}

impl GameModeStyle {
    pub fn new(color: Option<Color>) -> Self {
        Self {
            color: color.unwrap_or(Dark.palette().text),
        }
    }
}

impl container::StyleSheet for GameModeStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        container::Appearance {
            border_radius: 4.0,
            border_width: 1.0,
            border_color: self.color.clone(),
            ..Default::default()
        }
    }
}

fn game_mode_view<'l>(game_mode_id: &'l GameModeId, game_modes: &'l GameModes) -> Element<'l, Message> {
    match game_modes.get(&game_mode_id) {
        None => text(game_mode_id.to_string()).into(),
        Some(game_mode) => {
            let inner = container(game_mode_view_inner(game_mode))
                .style(theme::Container::Custom(Box::new(GameModeStyle::new(game_mode.color))))
                .padding([2, 4]);

            match game_mode.description.is_empty() {
                false => tooltip(inner, &game_mode.description, Position::Bottom).into(),
                true => inner.into(),
            }
        }
    }
}

fn server_view<'l>(server: &'l Server, bookmarks: &'l Bookmarks, game_modes: &'l GameModes) -> Element<'l, Message> {
    let is_bookmarked = bookmarks.is_bookmarked(&server.ip_port);
    let ip_port_text = format!("{}:{}", server.ip_port.ip(), server.ip_port.port());
    let game_modes = server.game_modes.iter().fold(row![].spacing(4), |row, game_mode_id| {
        row.push(game_mode_view(game_mode_id, game_modes))
    });

    container(
        row![
            thumbnail::thumbnail(server),
            column![
                row![
                    text(&server.name).size(28).width(Length::Fill),
                    favorite_button(is_bookmarked, 20).on_press(Message::Bookmarked(server.ip_port.clone(), !is_bookmarked)),
                    svg_button(icons::COPY_ICON.clone(), 20)
                        .on_press(Message::CopyToClipboard(server.ip_port.steam_connection_string())),
                    svg_button(icons::PLAY_ICON.clone(), 20).on_press(Message::LaunchGame(server.ip_port.clone())),
                ]
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
                    column![region(server, 20, 0), ping(server), game_modes]
                        .padding(4)
                        .spacing(4)
                        .align_items(Alignment::End)
                ]
            ],
        ]
        .spacing(4),
    )
    .padding([4, 14])
    .into()
}

fn servers_view<'l>(
    servers: &'l [Server],
    bookmarks: &'l Bookmarks,
    filter: &'l Filter,
    game_modes: &'l GameModes,
) -> Element<'l, Message> {
    let servers = servers.iter().filter(|server| filter.accept(*server, bookmarks));
    let servers_list = container(scrollable(
        servers.fold(column![], |c, server| c.push(server_view(server, bookmarks, game_modes))),
    ))
    .height(Length::Fill)
    .width(Length::Fill)
    .padding(4);

    servers_list.into()
}

fn filter_view<'l>(view: &'l MainView, filter: &'l Filter, game_modes: &'l GameModes) -> Element<'l, Message> {
    let filter_panel = container(scrollable(
        column![
            filter_section(None, ui::filter::bookmark_filter(filter)),
            filter_section(Some("Max ping"), ui::filter::ping_filter(filter)),
            filter_section(Some("Text filter"), ui::filter::advanced_text_filter(filter)),
            filter_section_with_switch(
                Some("Game modes"),
                ui::filter::game_modes_filter(filter, game_modes),
                filter.game_modes.is_enabled(),
                |checked| Message::Filter(FilterMessage::GameModeFilterEnabled(checked))
            ),
            filter_section_with_switch(
                Some("Countries"),
                ui::filter::country_filter(filter),
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

#[derive(Default)]
struct FilterSectionContainer;

impl container::StyleSheet for FilterSectionContainer {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Theme::Dark.palette().text.clone().into(),
            background: Theme::Dark.palette().background.into(),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: Color::from([0.5, 0.5, 0.5]),
        }
    }
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
    .style(theme::Container::Custom(Box::<FilterSectionContainer>::default()))
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
        .style(theme::Container::Custom(Box::<FilterSectionContainer>::default()))
        .padding(8)
}

mod thumbnail {
    use {
        crate::{
            application::{Message, PromisedValue, Server},
            icons,
        },
        iced::{
            widget::{container, image, text, Image},
            Element, Length,
        },
    };

    const THUMBNAIL_WIDTH: u16 = 250;
    const THUMBNAIL_HEIGHT: u16 = 125;

    fn image_thumbnail_viewer<'a>(image: image::Handle) -> Element<'a, Message> {
        Image::new(image)
            .width(Length::Units(THUMBNAIL_WIDTH))
            .height(Length::Units(THUMBNAIL_HEIGHT))
            .into()
    }

    fn image_thumbnail_content<'a>(server: &Server) -> Element<'a, Message> {
        match &server.map_thumbnail {
            PromisedValue::Ready(image) => image_thumbnail_viewer(image.clone()),
            PromisedValue::Loading => return text("Loading").into(),
            PromisedValue::None => image_thumbnail_viewer(icons::NO_IMAGE.clone()),
        }
    }

    pub fn thumbnail<'a>(server: &Server) -> Element<'a, Message> {
        container(image_thumbnail_content(server))
            .width(Length::Units(THUMBNAIL_WIDTH))
            .height(Length::Units(THUMBNAIL_HEIGHT))
            .center_x()
            .center_y()
            .into()
    }
}