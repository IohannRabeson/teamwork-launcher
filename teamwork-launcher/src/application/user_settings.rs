use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserSettings {
    pub teamwork_api_key: String,
    pub steam_executable_path: String,
    pub servers_filter_pane_ratio: f32,
    #[serde(default)]
    pub quit_on_launch: bool,
    #[serde(default)]
    pub quit_on_copy: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            servers_filter_pane_ratio: 0.8f32,
            teamwork_api_key: String::new(),
            steam_executable_path: String::new(),
            quit_on_launch: false,
            quit_on_copy: false,
        }
    }
}
