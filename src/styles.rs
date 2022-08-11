use iced::Background;

use crate::colors::parse_color;

pub struct Palette {
    pub background: iced::Color,
    pub foreground: iced::Color,
    pub card_background: iced::Color,
    pub card_foreground: iced::Color,
    pub card_background_hover: iced::Color,
    pub toggler_background: iced::Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            background: parse_color("#F2F2F2"),
            foreground: parse_color("#0D0D0D"),
            card_background: parse_color("#8C3232"),
            card_foreground: parse_color("#F2F2F2"),
            card_background_hover: parse_color("#BF7449"),
            toggler_background: parse_color("#457F8C"),
        }
    }
}

pub struct MainContainerStyle<'l> {
    palette: &'l Palette,
}

impl<'l> iced::container::StyleSheet for MainContainerStyle<'l> {
    fn style(&self) -> iced::container::Style {
        iced::container::Style {
            text_color: Some(self.palette.foreground),
            background: Some(Background::Color(self.palette.background)),
            ..Default::default()
        }
    }
}

impl<'l> MainContainerStyle<'l> {
    pub fn new(palette: &'l Palette) -> Self {
        Self { palette: palette }
    }
}

pub struct ServerCardStyleSheet<'l> {
    palette: &'l Palette,
}

impl<'l> ServerCardStyleSheet<'l> {
    pub fn new(palette: &'l Palette) -> Self {
        Self { palette }
    }
}

impl<'l> iced_pure::widget::button::StyleSheet for ServerCardStyleSheet<'l> {
    fn active(&self) -> iced::button::Style {
        let mut style = iced::button::Style::default();
        style.background = Some(Background::Color(self.palette.card_background.clone()));
        style.text_color = self.palette.card_foreground.clone();
        style.border_radius = 6f32;
        style
    }

    fn hovered(&self) -> iced::button::Style {
        let mut active = self.active();
        active.background = Some(Background::Color(self.palette.card_background_hover));

        active
    }

    fn pressed(&self) -> iced::button::Style {
        iced::button::Style {
            shadow_offset: iced::Vector::default(),
            ..self.active()
        }
    }
}

pub struct CardButtonStyleSheet<'l> {
    palette: &'l Palette,
}

impl<'l> CardButtonStyleSheet<'l> {
    pub fn new(palette: &'l Palette) -> Self {
        Self { palette }
    }
}

impl<'l> iced_pure::widget::button::StyleSheet for CardButtonStyleSheet<'l> {
    fn active(&self) -> iced::button::Style {
        let mut style = iced::button::Style::default();
        style.background = Some(Background::Color(self.palette.card_background.clone()));
        style.text_color = self.palette.card_foreground.clone();
        style.border_color = self.palette.foreground;
        style.border_radius = 6f32;
        style.border_width = 0f32;
        style
    }

    fn hovered(&self) -> iced::button::Style {
        let mut active = self.active();
        active.border_color = self.palette.background.clone();
        active.border_width = 1f32;
        active
    }
}

pub struct ToggleStyle<'l> {
    palette: &'l Palette
}

impl<'l> ToggleStyle<'l> {
    pub fn new(palette: &'l Palette) -> Self {
        Self { palette }
    }
}

impl<'l> iced::pure::widget::toggler::StyleSheet for ToggleStyle<'l> {
    fn active(&self, is_active: bool) -> iced::toggler::Style {
        iced::toggler::Style {
            background: if is_active {
                self.palette.toggler_background.clone()
            } else {
                iced::Color::from_rgb(0.7, 0.7, 0.7)
            },
            background_border: None,
            foreground: iced::Color::WHITE,
            foreground_border: None,
        }
    }

    fn hovered(&self, is_active: bool) -> iced::toggler::Style {
        iced::toggler::Style {
            foreground: iced::Color::from_rgb(0.95, 0.95, 0.95),
            ..self.active(is_active)
        }
    }
}