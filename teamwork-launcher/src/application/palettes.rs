use iced::{theme, Color};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GREEN: Color = Color::from_rgb8(75, 116, 28);
    pub static ref RED: Color = Color::from_rgb8(189, 59, 59);
}

pub fn create_blue_palette() -> theme::Custom {
    theme::Custom::new(theme::palette::Palette {
        background: Color::from_rgb8(38, 35, 33),
        text: Color::from([0.9, 0.9, 0.9]),
        primary: Color::from_rgb8(57, 92, 120),
        success: *GREEN,
        danger: *RED,
    })
}

pub fn create_red_palette() -> theme::Custom {
    theme::Custom::new(theme::palette::Palette {
        background: Color::from_rgb8(38, 35, 33),
        text: Color::from([0.9, 0.9, 0.9]),
        primary: Color::from_rgb8(159, 49, 47),
        success: *GREEN,
        danger: *RED,
    })
}
