use {
    crate::application::SettingsError,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
};

#[derive(Serialize, Deserialize)]
pub struct UserSettings {
    pub teamwork_api_key: String,
    pub steam_executable_path: String,
    pub servers_filter_pane_ratio: f32,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            servers_filter_pane_ratio: 0.8f32,
            teamwork_api_key: String::new(),
            steam_executable_path: String::new(),
        }
    }
}
