use {
    crate::{
        application::Messages,
        icons::Icons,
        models::{Country, Server, Thumbnail},
        promised_value::PromisedValue,
        ui::VISUAL_SPACING_SMALL,
    },
    iced::{
        widget::{container, horizontal_space, image, row, svg, text},
        Element, Length,
    },
};
use iced::{theme, widget::Text};
use super::{VISUAL_SPACING_MEDIUM, styles};

const THUMBNAIL_WIDTH: u16 = 250;
const THUMBNAIL_HEIGHT: u16 = 125;

fn image_thumbnail_viewer<'a>(image: image::Handle) -> Element<'a, Messages> {
    image::viewer(image)
        .width(Length::Units(THUMBNAIL_WIDTH))
        .height(Length::Units(THUMBNAIL_HEIGHT))
        .scale_step(0.0)
        .into()
}

fn image_thumbnail_content<'a>(server: &Server, icons: &Icons) -> Element<'a, Messages> {
    match &server.map_thumbnail {
        Thumbnail::Ready(image) => image_thumbnail_viewer(image.clone()),
        Thumbnail::Loading => return text("Loading").into(),
        Thumbnail::None => image_thumbnail_viewer(icons.no_image()),
    }
}

pub fn thumbnail<'a>(server: &Server, icons: &Icons) -> Element<'a, Messages> {
    container(image_thumbnail_content(server, icons))
        .width(Length::Units(THUMBNAIL_WIDTH))
        .height(Length::Units(THUMBNAIL_HEIGHT))
        .center_x()
        .center_y()
        .into()
}

pub fn region<'a>(server: &Server, icons: &Icons, size: u16, padding: u16) -> Element<'a, Messages> {
    match &server.country {
        PromisedValue::Ready(country) => row![
            text(format!("Region:")),
            horizontal_space(Length::Units(VISUAL_SPACING_SMALL)),
            country_icon(icons, country, size, padding)
        ]
        .into(),
        PromisedValue::Loading => text("Region: loading...").into(),
        PromisedValue::None => text("Region: unknown").into(),
    }
}

pub fn country_icon<'a>(icons: &Icons, country: &Country, size: u16, padding: u16) -> Element<'a, Messages> {
    let size = size - (padding * 2);

    match icons.flag(country.code()) {
        Some(icon) => {
            tooltip(container(svg(icon).width(Length::Units(size)).height(Length::Units(size)))
                    .padding(padding), &country, iced::widget::tooltip::Position::Right)
                    .into()
        },
        None => text(format!("Region: {} ({})", country, country.code())).into(),
    }
}

pub fn ping<'a>(server: &Server) -> Text<'a> {
    match &server.ping {
        PromisedValue::Ready(duration) => text(format!("Ping: {} ms", duration.as_millis())),
        PromisedValue::Loading => text("Ping: loading..."),
        PromisedValue::None => text("Ping: timeout"),
    }
    .into()
}

pub fn tooltip<'a>(
    content: impl Into<Element<'a, Messages>>,
    tooltip: impl ToString,
    position: iced::widget::tooltip::Position) -> Element<'a, Messages>
{
    iced::widget::tooltip(
        content,
        tooltip,
        position,
    )
    .gap(VISUAL_SPACING_MEDIUM)
    .style(theme::Container::Custom(Box::new(styles::ToolTip::default())))
    .into()
}