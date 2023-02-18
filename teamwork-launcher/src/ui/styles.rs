use iced::{
    Background,
    Color, Theme, widget::{container, svg},
};

#[derive(Default)]
pub struct ToolTip;

impl container::StyleSheet for ToolTip {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
            border_radius: 4f32,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct SvgButtonIconStyle;

impl svg::StyleSheet for SvgButtonIconStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> svg::Appearance {
        svg::Appearance {
            color: Some(Color::from_rgb(1.0, 1.0, 1.0)),
        }
    }
}

pub enum ColoredPingIndicatorStyle {
    Good,
    Bad,
    VeryBad,
}

impl svg::StyleSheet for ColoredPingIndicatorStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> svg::Appearance {
        svg::Appearance {
            color: Some(match self {
                ColoredPingIndicatorStyle::Good => Color::from_rgb8(0, 255, 0),
                ColoredPingIndicatorStyle::Bad => Color::from_rgb8(255, 255, 0),
                ColoredPingIndicatorStyle::VeryBad => Color::from_rgb8(255, 0, 0),
            }),
        }
    }
}

pub struct BoxContainerStyle;

impl container::StyleSheet for BoxContainerStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let background = style.palette().background;

        container::Appearance {
            background: background.into(),
            ..Default::default()
        }
    }
}
