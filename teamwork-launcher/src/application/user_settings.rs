use {
    crate::application::{palettes, paths::get_default_steam_executable},
    iced::Theme,
    serde::{Deserialize, Serialize},
    std::fmt::{Display, Formatter},
};

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

impl From<LauncherTheme> for Theme {
    fn from(theme: LauncherTheme) -> Self {
        match theme {
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
    pub steam_executable_path: String,
    pub servers_filter_pane_ratio: f32,
    pub quit_on_launch: bool,
    pub quit_on_copy: bool,
    pub theme: LauncherTheme,
    #[serde(default = "default_max_thumbnails_cache_size")]
    pub max_thumbnails_cache_size_mb: u64,
    #[serde(default)]
    pub window: Option<WindowSettings>,
    teamwork_api_key: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            servers_filter_pane_ratio: 0.65f32,
            teamwork_api_key: String::new(),
            steam_executable_path: get_default_steam_executable()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_default(),
            quit_on_launch: false,
            quit_on_copy: false,
            window: None,
            theme: LauncherTheme::Red,
            /// The maximum size for the thumbnails cache.
            /// 20Mb by default, I observe a usage of ~10MB.
            /// If the maximum size is reached, then no more images are added
            /// to the cache. I want to keep the cache system simple to avoid the need to store and read additional
            /// information on disk (such as the date of add, or the order of add for each entry) and this behavior is
            /// the simplest I can think for now as it requires no additional data.
            max_thumbnails_cache_size_mb: default_max_thumbnails_cache_size(),
        }
    }
}

const fn default_max_thumbnails_cache_size() -> u64 {
    20
}

impl UserSettings {
    const TEAMWORK_API_KEY_ENV: &'static str = "TEAMWORK_API_KEY";

    fn get_key_from_env() -> Option<String> {
        std::env::var(Self::TEAMWORK_API_KEY_ENV).ok()
    }

    pub fn has_teamwork_api_key(&self) -> bool {
        !self.teamwork_api_key().trim().is_empty()
    }

    pub fn is_teamwork_api_key_from_env(&self) -> bool {
        Self::get_key_from_env().is_some()
    }

    pub fn teamwork_api_key(&self) -> String {
        match Self::get_key_from_env() {
            Some(api_key) => api_key,
            None => self.teamwork_api_key.clone(),
        }
    }

    pub fn set_teamwork_api_key(&mut self, key: String) {
        self.teamwork_api_key = key;
    }
}
