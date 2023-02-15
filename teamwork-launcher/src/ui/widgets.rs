use {
    crate::{
        application::{Country, Message, PromisedValue, Server},
        fonts, icons,
        ui::{styles, styles::ColoredPingIndicatorStyle},
    },
    iced::{
        theme,
        widget::{container, row, svg, text, tooltip::Position},
        Alignment, Element, Length,
    },
    std::time::Duration,
};

pub fn country_icon<'a>(country: &Country, size: u16, padding: u16) -> Element<'a, Message> {
    let size = size - (padding * 2);

    match icons::flag(country.code()) {
        Some(icon) => tooltip(
            container(svg(icon).width(Length::Units(size)).height(Length::Units(size))).padding(padding),
            country,
            iced::widget::tooltip::Position::Right,
        ),
        None => text(format!("Region: {} ({})", country, country.code())).into(),
    }
}

pub fn ping_icon<'a>(duration: &Duration, size: u16) -> Element<'a, Message> {
    let icon = if duration < &Duration::from_millis(25) {
        svg(icons::RECEPTION_GOOD.clone()).style(theme::Svg::Custom(Box::new(ColoredPingIndicatorStyle::Good)))
    } else if duration < &Duration::from_millis(50) {
        svg(icons::RECEPTION_OK.clone()).style(theme::Svg::Custom(Box::new(ColoredPingIndicatorStyle::Good)))
    } else if duration < &Duration::from_millis(100) {
        svg(icons::RECEPTION_BAD.clone()).style(theme::Svg::Custom(Box::new(ColoredPingIndicatorStyle::Bad)))
    } else {
        svg(icons::RECEPTION_VERY_BAD.clone()).style(theme::Svg::Custom(Box::new(ColoredPingIndicatorStyle::VeryBad)))
    };

    tooltip(
        container(icon.width(Length::Units(size)).height(Length::Units(size))),
        format!("{}ms", duration.as_millis()),
        Position::Bottom,
    )
}

pub fn tooltip<'a>(
    content: impl Into<Element<'a, Message>>,
    tooltip: impl ToString,
    position: iced::widget::tooltip::Position,
) -> Element<'a, Message> {
    iced::widget::tooltip(content, tooltip, position)
        .gap(8)
        .style(theme::Container::Custom(Box::<styles::ToolTip>::default()))
        .into()
}
