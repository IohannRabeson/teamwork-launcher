use iced::{
    widget::{button, container},
    Background, Theme, Vector,
};

#[derive(Default)]
pub struct ToolTip;

impl container::StyleSheet for ToolTip {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let mut background_color = style.extended_palette().background.weak.color;

        background_color.a = 0.9;

        container::Appearance {
            background: Some(Background::Color(background_color)),
            border_radius: 4f32,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Announce;

impl button::StyleSheet for Announce {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::new(0f32, 0f32),
            background: Some(Background::Color(style.palette().danger)),
            border_radius: 3.0,
            text_color: style.palette().text,
            ..Default::default()
        }
    }
}
