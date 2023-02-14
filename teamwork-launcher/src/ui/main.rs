use {
    crate::{
        application::{MainView, Message, PromisedValue, Server},
        ui,
    },
    iced::{
        Alignment,
        widget::{button, column, toggler, container, horizontal_space, row, scrollable, text, text_input},
        Element, Length,
    },
};
use crate::application::Bookmarks;
use crate::ui::buttons::{svg_button, favorite_button};
use crate::application::FilterMessage;
use crate::icons;

fn server_view<'l>(server: &'l Server, bookmarks: &'l Bookmarks) -> Element<'l, Message> {
    let is_bookmarked = bookmarks.is_bookmarked(&server.ip_port);

    container(row![
        thumbnail::thumbnail(server),
        column![
            row![
                text(&server.name).width(Length::Fill),
                svg_button(icons::PLAY_ICON.clone(), 20).on_press(Message::LaunchGame(server.ip_port.clone())),
                svg_button(icons::COPY_ICON.clone(), 20).on_press(Message::CopyToClipboard(server.ip_port.steam_connection_string())),
                favorite_button(is_bookmarked, 20).on_press(Message::Bookmarked(server.ip_port.clone(), !is_bookmarked)),
            ].spacing(4),
            text(&format!("{}:{}", server.ip_port.ip(), server.ip_port.port())),
            text(&server.map),
            text(&format!("{} / {}", server.current_players_count, server.max_players_count)),
            text(match &server.country {
                PromisedValue::None => String::from("-"),
                PromisedValue::Loading => String::from("Waiting country"),
                PromisedValue::Ready(country) => country.to_string(),
            }),
            text(match &server.ping {
                PromisedValue::None => String::from("-"),
                PromisedValue::Loading => String::from("Waiting ping"),
                PromisedValue::Ready(ping) => ping.as_millis().to_string(),
            })
        ],
    ]
    .spacing(4))
    .padding([4, 14])
    .into()
}

pub fn view<'l>(view: &'l MainView, bookmarks: &'l Bookmarks) -> Element<'l, Message> {
    let servers = view.servers.iter().filter(|server| view.filter.accept(*server, bookmarks));
    let servers_list = container(scrollable(servers.fold(column![], |c, server| {
        c.push(server_view(server, bookmarks))
    })))
    .height(Length::Fill)
    .width(Length::Fill);

    let filter_panel = container(scrollable(
        column![
            toggler(Some(String::from("Bookmarks only")), view.filter.bookmarked_only, |checked|Message::Filter(FilterMessage::BookmarkedOnlyChecked(checked))).width(Length::Shrink),
            ui::filter::country_filter(&view.filter)
        ].spacing(4))).padding(4);
    let textual_filters = container(ui::filter::text_filter(&view.filter))
        .padding([0, 8]);

    column![
        row![
            text("Teamwork launcher").size(48),
            horizontal_space(Length::Fill),
            svg_button(icons::SETTINGS_ICON.clone(), 24).on_press(Message::ShowSettings),
            svg_button(icons::REFRESH_ICON.clone(), 24).on_press(Message::RefreshServers)
        ]
        .align_items(Alignment::Center)
        .spacing(4)
        .padding([0, 8]),
        textual_filters,
        row![servers_list, filter_panel].spacing(4)
    ]
    .padding([8, 0])
    .spacing(4)
    .into()
}

mod thumbnail {
    use {
        crate::{
            application::{Message, PromisedValue, Server},
            icons,
        },
        iced::{
            widget::{container, image, text},
            Element, Length,
        },
    };

    const THUMBNAIL_WIDTH: u16 = 300;
    const THUMBNAIL_HEIGHT: u16 = 150;

    fn image_thumbnail_viewer<'a>(image: image::Handle) -> Element<'a, Message> {
        image::viewer(image)
            .width(Length::Units(THUMBNAIL_WIDTH))
            .height(Length::Units(THUMBNAIL_HEIGHT))
            .scale_step(0.0)
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
