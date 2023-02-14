use iced::{
    widget::{button, container, svg},
    Background, Color, Theme, Vector,
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

pub struct Announce {
    background: Color,
    text: Color,
}

impl Announce {
    pub fn new(text: Color, background: Color) -> Self {
        Self { text, background }
    }
}

impl button::StyleSheet for Announce {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::new(0f32, 0f32),
            background: Some(Background::Color(self.background.clone())),
            border_radius: 3.0,
            text_color: self.text.clone(),
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
