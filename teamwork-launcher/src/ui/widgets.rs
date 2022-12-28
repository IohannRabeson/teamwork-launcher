use {
    super::{styles, VISUAL_SPACING_MEDIUM},
    crate::{
        application::Messages,
        fonts::{self, TEXT_FONT_SIZE},
        icons,
        models::{Country, Server, Thumbnail},
        promised_value::PromisedValue,
        ui::VISUAL_SPACING_SMALL,
    },
    iced::{
        theme,
        widget::{container, horizontal_space, image, row, svg, text, tooltip::Position},
        Alignment, Color, Element, Length, Theme,
    },
    lazy_static::lazy_static,
    std::time::Duration,
};

const THUMBNAIL_WIDTH: u16 = 250;
const THUMBNAIL_HEIGHT: u16 = 125;

fn image_thumbnail_viewer<'a>(image: image::Handle) -> Element<'a, Messages> {
    image::viewer(image)
        .width(Length::Units(THUMBNAIL_WIDTH))
        .height(Length::Units(THUMBNAIL_HEIGHT))
        .scale_step(0.0)
        .into()
}

fn image_thumbnail_content<'a>(server: &Server) -> Element<'a, Messages> {
    match &server.map_thumbnail {
        Thumbnail::Ready(image) => image_thumbnail_viewer(image.clone()),
        Thumbnail::Loading => return text("Loading").into(),
        Thumbnail::None => image_thumbnail_viewer(icons::NO_IMAGE.clone()),
    }
}

pub fn thumbnail<'a>(server: &Server) -> Element<'a, Messages> {
    container(image_thumbnail_content(server))
        .width(Length::Units(THUMBNAIL_WIDTH))
        .height(Length::Units(THUMBNAIL_HEIGHT))
        .center_x()
        .center_y()
        .into()
}

pub fn region<'a>(server: &Server, size: u16, padding: u16) -> Element<'a, Messages> {
    match &server.country {
        PromisedValue::Ready(country) => row![
            text("Region:".to_string()).size(fonts::TEXT_FONT_SIZE),
            horizontal_space(Length::Units(VISUAL_SPACING_SMALL)),
            country_icon(country, size, padding)
        ]
        .into(),
        PromisedValue::Loading => text("Region: loading...").size(fonts::TEXT_FONT_SIZE).into(),
        PromisedValue::None => text("Region: unknown").size(fonts::TEXT_FONT_SIZE).into(),
    }
}

pub fn country_icon<'a>(country: &Country, size: u16, padding: u16) -> Element<'a, Messages> {
    let size = size - (padding * 2);

    match icons::flag(country.code()) {
        Some(icon) => tooltip(
            container(svg(icon).width(Length::Units(size)).height(Length::Units(size))).padding(padding),
            country,
            iced::widget::tooltip::Position::Right,
        ),
        None => text(format!("Region: {} ({})", country, country.code()))
            .size(fonts::TEXT_FONT_SIZE)
            .into(),
    }
}

struct ColoredSvgStyle(Color);

impl svg::StyleSheet for ColoredSvgStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> svg::Appearance {
        svg::Appearance { color: Some(self.0) }
    }
}

lazy_static! {
    pub static ref GOOD_PING_COLOR: Color = Color::from_rgb8(0, 255, 0);
    pub static ref BAD_PING_COLOR: Color = Color::from_rgb8(255, 255, 0);
    pub static ref VERY_BAD_PING_COLOR: Color = Color::from_rgb8(255, 0, 0);
}

pub fn ping<'a>(server: &Server) -> Element<'a, Messages> {
    match &server.ping {
        PromisedValue::Ready(duration) => {
            row![text("Ping:").size(fonts::TEXT_FONT_SIZE), ping_icon(duration, TEXT_FONT_SIZE),]
                .spacing(VISUAL_SPACING_SMALL)
                .align_items(Alignment::End)
                .into()
        }
        PromisedValue::Loading => text("Ping: loading...").size(fonts::TEXT_FONT_SIZE).into(),
        PromisedValue::None => text("Ping: timeout").size(fonts::TEXT_FONT_SIZE).into(),
    }
}

fn ping_icon<'a>(duration: &Duration, size: u16) -> Element<'a, Messages> {
    let icon = if duration < &Duration::from_millis(25) {
        svg(icons::RECEPTION_GOOD.clone()).style(theme::Svg::Custom(Box::new(ColoredSvgStyle(GOOD_PING_COLOR.clone()))))
    } else if duration < &Duration::from_millis(50) {
        svg(icons::RECEPTION_OK.clone()).style(theme::Svg::Custom(Box::new(ColoredSvgStyle(GOOD_PING_COLOR.clone()))))
    } else if duration < &Duration::from_millis(100) {
        svg(icons::RECEPTION_BAD.clone()).style(theme::Svg::Custom(Box::new(ColoredSvgStyle(BAD_PING_COLOR.clone()))))
    } else {
        svg(icons::RECEPTION_VERY_BAD.clone())
            .style(theme::Svg::Custom(Box::new(ColoredSvgStyle(VERY_BAD_PING_COLOR.clone()))))
    };

    tooltip(
        container(icon.width(Length::Units(size)).height(Length::Units(size))),
        format!("{}ms", duration.as_millis()),
        Position::Bottom,
    )
    .into()
}

pub fn tooltip<'a>(
    content: impl Into<Element<'a, Messages>>,
    tooltip: impl ToString,
    position: iced::widget::tooltip::Position,
) -> Element<'a, Messages> {
    iced::widget::tooltip(content, tooltip, position)
        .gap(VISUAL_SPACING_MEDIUM)
        .style(theme::Container::Custom(Box::<styles::ToolTip>::default()))
        .into()
}
