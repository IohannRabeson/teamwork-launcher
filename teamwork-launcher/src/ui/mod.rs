use {
    crate::application::{Filter, MainView, Message, PromisedValue, Screens, Server},
    iced::{
        widget::{button, column, row, scrollable, text},
        Element,
    },
};

pub fn main_view(view: &MainView) -> Element<Message> {
    let servers = view.servers.iter().filter(|server| view.filter.accept(*server));
    let servers_list = scrollable(servers.fold(column![], |c, server| {
        let r = row![
            thumbnail::thumbnail(server),
            column![
                text(&server.name),
                text(&format!("{}:{}", server.ip_port.ip(), server.ip_port.port())),
                text(&server.map),
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
        .spacing(4);

        c.push(r)
    }));

    let country_filters = filter::country_filter(&view.filter);
    let textual_filters = filter::text_filter(&view.filter);

    column![
        row![textual_filters, button("Refresh").on_press(Message::RefreshServers)],
        row![servers_list, country_filters]
    ]
    .into()
}

mod filter {
    use {
        crate::application::{Filter, FilterMessage, Message},
        iced::{
            widget::{checkbox, column, text_input},
            Element,
        },
    };

    pub fn text_filter(filter: &Filter) -> Element<Message> {
        text_input("Filter", filter.text.text(), |text| {
            Message::Filter(FilterMessage::TextChanged(text))
        })
        .into()
    }

    pub fn country_filter(filter: &Filter) -> Element<Message> {
        let filter = &filter.country;
        filter
            .available_countries()
            .fold(column![], |column, country| {
                column.push(checkbox(country.name(), filter.is_checked(country), |checked| {
                    Message::Filter(FilterMessage::CountryChecked(country.clone(), checked))
                }))
            })
            .push(checkbox("No country", filter.accept_no_country(), |checked| {
                Message::Filter(FilterMessage::NoCountryChecked(checked))
            }))
            .into()
    }
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

    const THUMBNAIL_WIDTH: u16 = 250;
    const THUMBNAIL_HEIGHT: u16 = 125;

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
