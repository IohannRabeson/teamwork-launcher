use {
    crate::{
        application::{
            Bookmarks, Filter, FilterMessage, MainView, Message, PaneId, PaneMessage, PaneView, PromisedValue, Server,
        },
        icons,
        ui::{
            self,
            buttons::{favorite_button, svg_button},
            widgets,
        },
    },
    iced::{
        widget::{
            button, column, container, horizontal_space,
            pane_grid::{self, Pane},
            row, scrollable, text, text_input, toggler, PaneGrid,
        },
        Alignment, Element, Length,
    },
    iced_lazy::responsive,
};

pub fn view<'l>(
    view: &'l MainView,
    servers: &'l [Server],
    bookmarks: &'l Bookmarks,
    filter: &'l Filter,
) -> Element<'l, Message> {
    let textual_filters = container(ui::filter::text_filter(filter)).padding([0, 8]);
    let pane_grid = PaneGrid::new(&view.panes, |id, pane, is_maximized| {
        pane_grid::Content::new(responsive(move |size| match &pane.id {
            PaneId::Servers => servers_view(servers, bookmarks, filter),
            PaneId::Filters => filter_view(view, filter),
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

fn server_view<'l>(server: &'l Server, bookmarks: &'l Bookmarks) -> Element<'l, Message> {
    let is_bookmarked = bookmarks.is_bookmarked(&server.ip_port);
    let ip_port_text = format!("{}:{}", server.ip_port.ip(), server.ip_port.port());

    container(
        row![
            thumbnail::thumbnail(server),
            column![
                row![
                    text(&server.name).size(28).width(Length::Fill),
                    svg_button(icons::PLAY_ICON.clone(), 20).on_press(Message::LaunchGame(server.ip_port.clone())),
                    svg_button(icons::COPY_ICON.clone(), 20)
                        .on_press(Message::CopyToClipboard(server.ip_port.steam_connection_string())),
                    favorite_button(is_bookmarked, 20).on_press(Message::Bookmarked(server.ip_port.clone(), !is_bookmarked)),
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
                    column![region(server, 20, 0), ping(server),]
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

fn servers_view<'l>(servers: &'l [Server], bookmarks: &'l Bookmarks, filter: &'l Filter) -> Element<'l, Message> {
    let servers = servers.iter().filter(|server| filter.accept(*server, bookmarks));
    let servers_list = container(scrollable(
        servers.fold(column![], |c, server| c.push(server_view(server, bookmarks))),
    ))
    .height(Length::Fill)
    .width(Length::Fill)
    .padding(4);

    servers_list.into()
}

fn filter_view<'l>(view: &'l MainView, filter: &'l Filter) -> Element<'l, Message> {
    let filter_panel = container(scrollable(
        column![
            toggler(Some(String::from("Bookmarks only")), filter.bookmarked_only, |checked| {
                Message::Filter(FilterMessage::BookmarkedOnlyChecked(checked))
            })
            .width(Length::Shrink),
            ui::filter::country_filter(filter)
        ]
        .spacing(4),
    ))
    .padding(4);

    filter_panel.into()
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
