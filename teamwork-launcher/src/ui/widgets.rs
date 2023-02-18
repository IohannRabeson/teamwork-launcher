use {
    crate::{
        application::{
            game_mode::{GameMode, GameModeId, GameModes},
            Country, Message, PromisedValue, Server,
        },
        icons,
        ui::{styles, styles::ColoredPingIndicatorStyle},
    },
    iced::{
        theme,
        widget::{container, row, svg, text, tooltip as iced_tooltip, Image}, Color, Element, Length,
        Theme::{self, Dark},
    },
    std::time::Duration,
};

pub fn country_icon<'a>(country: &Country, size: u16, padding: u16) -> Element<'a, Message> {
    let size = Length::Fixed((size - (padding * 2)) as f32);

    match icons::flag(country.code()) {
        Some(icon) => tooltip(
            container(svg(icon).width(size).height(size)).padding(padding),
            country,
            iced_tooltip::Position::Left,
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
    let size = Length::Fixed(size as f32);

    tooltip(
        container(icon.width(size).height(size)),
        format!("{}ms", duration.as_millis()),
        iced_tooltip::Position::Bottom,
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

pub fn ping<'a>(value: &PromisedValue<Duration>) -> Element<'a, Message> {
    match value {
        PromisedValue::Ready(duration) => ping_icon(duration, 20).into(),
        PromisedValue::Loading => text("Loading...").into(),
        PromisedValue::None => text("Timeout").into(),
    }
}

pub fn region<'a>(server: &Server, size: u16, padding: u16) -> Element<'a, Message> {
    match &server.country {
        PromisedValue::Ready(country) => row![text("Region:".to_string()), country_icon(country, size, padding)]
            .spacing(4)
            .into(),
        PromisedValue::Loading => text("Region: loading...").into(),
        PromisedValue::None => text("Region: unknown").into(),
    }
}

fn image_thumbnail_content<'a>(server: &Server) -> Element<'a, Message> {
    match &server.map_thumbnail {
        PromisedValue::Ready(image) => Image::new(image.clone()).into(),
        PromisedValue::Loading => return text("Loading").into(),
        PromisedValue::None => Image::new(icons::NO_IMAGE.clone()).into(),
    }
}

pub fn thumbnail<'a>(server: &Server, width: Length, height: Length) -> Element<'a, Message> {
    container(image_thumbnail_content(server))
        .width(width)
        .height(height)
        .center_x()
        .center_y()
        .into()
}

fn game_mode_view<'l>(game_mode_id: &'l GameModeId, game_modes: &'l GameModes) -> Element<'l, Message> {
    match game_modes.get(&game_mode_id) {
        None => text(game_mode_id.to_string()).into(),
        Some(game_mode) => {
            let inner = container(game_mode_view_inner(game_mode))
                .style(theme::Container::Custom(Box::new(GameModeStyle::new(game_mode.color))))
                .padding([2, 4]);

            match game_mode.description.is_empty() {
                false => tooltip(inner, &game_mode.description, iced_tooltip::Position::Bottom).into(),
                true => inner.into(),
            }
        }
    }
}

fn game_mode_view_inner(game_mode: &GameMode) -> Element<Message> {
    match game_mode.color {
        Some(_color) => text(&game_mode.title),
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

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            border_radius: 4.0,
            border_width: 1.0,
            border_color: self.color.clone(),
            ..Default::default()
        }
    }
}

pub fn game_modes<'l>(game_modes: &'l GameModes, modes_to_display: &'l [GameModeId]) -> Element<'l, Message> {
    modes_to_display
        .iter()
        .fold(row![].spacing(4), |row, game_mode_id| {
            row.push(game_mode_view(game_mode_id, game_modes))
        })
        .into()
}
