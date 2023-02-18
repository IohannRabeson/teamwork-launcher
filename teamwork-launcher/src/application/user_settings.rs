use iced::Theme;
use {
    serde::{Deserialize, Serialize},
    std::fmt::{Display, Formatter},
};
use crate::application::palettes;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct WindowSettings {
    pub window_x: i32,
    pub window_y: i32,
    pub window_width: u32,
    pub window_height: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum LauncherTheme {
    Red,
    Blue,
}

impl Into<Theme> for LauncherTheme {
    fn into(self) -> Theme {
        match self {
            LauncherTheme::Red => Theme::Custom(Box::new(palettes::create_red_palette())),
            LauncherTheme::Blue => Theme::Custom(Box::new(palettes::create_blue_palette())),
        }
    }
}

impl Display for LauncherTheme {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            LauncherTheme::Red => write!(f, "Red"),
            LauncherTheme::Blue => write!(f, "Blue"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserSettings {
    pub teamwork_api_key: String,
    pub steam_executable_path: String,
    pub servers_filter_pane_ratio: f32,
    pub quit_on_launch: bool,
    pub quit_on_copy: bool,
    pub theme: LauncherTheme,
    #[serde(default)]
    pub window: Option<WindowSettings>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            servers_filter_pane_ratio: 0.65f32,
            teamwork_api_key: String::new(),
            steam_executable_path: String::new(),
            quit_on_launch: false,
            quit_on_copy: false,
            window: None,
            theme: LauncherTheme::Red,
        }
    }
}
