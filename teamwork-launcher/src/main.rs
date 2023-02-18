// Prevent a console to pop on Windows
#![windows_subsystem = "windows"]

use {
    crate::{
        application::{servers_source::ServersSource, user_settings::WindowSettings, Bookmarks, Filter, UserSettings},
        common_settings::{get_configuration_directory, read_file},
    },
    iced::{window::Position, Application, Settings},
};

mod application;
mod common_settings;
mod fonts;
mod icons;
mod ui;

const APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");
const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_SHA_SHORT: &str = env!("VERGEN_GIT_SHA_SHORT");

fn main() -> iced::Result {
    application::TeamworkLauncher::run(load_settings())
}

#[derive(Default)]
pub struct ApplicationFlags {
    pub bookmarks: Bookmarks,
    pub user_settings: UserSettings,
    pub filter: Filter,
    pub servers_sources: Vec<ServersSource>,
}

fn load_settings() -> Settings<ApplicationFlags> {
    let configuration_directory = get_configuration_directory();

    println!("Configuration directory: {}", configuration_directory.display());

    let bookmarks: Bookmarks = read_file(configuration_directory.join("bookmarks.json")).unwrap_or_default();
    let mut user_settings: UserSettings = read_file(configuration_directory.join("settings.json")).unwrap_or_default();
    let filter: Filter = read_file(configuration_directory.join("filters.json")).unwrap_or_default();
    let servers_sources: Vec<ServersSource> =
        read_file(configuration_directory.join("sources.json")).unwrap_or_else(|error| {
            eprintln!("Failed to read sources.json: {}", error);

            vec![
                ServersSource::new("Payload", "https://teamwork.tf/api/v1/quickplay/payload/servers"),
                ServersSource::new("Payload Race", "https://teamwork.tf/api/v1/quickplay/payload-race/servers"),
                ServersSource::new("King Of The Hill", "https://teamwork.tf/api/v1/quickplay/koth/servers"),
                ServersSource::new("Capture The Flag", "https://teamwork.tf/api/v1/quickplay/ctf/servers"),
                ServersSource::new("Attack/Defend", "https://teamwork.tf/api/v1/quickplay/attack-defend/servers"),
                ServersSource::new("Control Point", "https://teamwork.tf/api/v1/quickplay/control-point/servers"),
                ServersSource::new("Medieval Mode", "https://teamwork.tf/api/v1/quickplay/medieval-mode/servers"),
            ]
        });

    if let Some(window_settings) = user_settings.window.clone() {
        let mut settings = Settings::with_flags(ApplicationFlags {
            bookmarks,
            user_settings,
            filter,
            servers_sources,
        });

        settings.window.position = Position::Specific(window_settings.window_x, window_settings.window_y);
        settings.window.size.0 = window_settings.window_width;
        settings.window.size.1 = window_settings.window_height;

        settings
    } else {
        user_settings.window = Some(WindowSettings::default());

        Settings::with_flags(ApplicationFlags {
            bookmarks,
            user_settings,
            filter,
            servers_sources,
        })
    }
}
