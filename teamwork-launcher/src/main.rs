// Prevent a console to pop on Windows
#![windows_subsystem = "windows"]

use {
    crate::{
        application::{
            filter::filter_servers::Filter, servers_source::ServersSource, user_settings::WindowSettings, Bookmarks,
            UserSettings,
        },
        common_settings::{get_configuration_directory, read_file},
    },
    iced::{window::Position, Application, Settings},
    log::{error, info},
    std::fs::OpenOptions,
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
    create_configuration_directory().expect("create configuration directory");
    setup_logger().expect("setup logger");
    info!("Teamwork Launcher v{}", APPLICATION_VERSION);
    application::TeamworkLauncher::run(load_settings())
}

#[derive(Default)]
pub struct ApplicationFlags {
    pub bookmarks: Bookmarks,
    pub user_settings: UserSettings,
    pub filter: Filter,
    pub servers_sources: Vec<ServersSource>,
}

fn create_configuration_directory() -> Result<(), std::io::Error> {
    std::fs::create_dir_all(&get_configuration_directory())
}

fn load_settings() -> Settings<ApplicationFlags> {
    let configuration_directory = get_configuration_directory();

    info!("Configuration directory: {}", configuration_directory.display());

    let bookmarks: Bookmarks = read_file(configuration_directory.join("bookmarks.json")).unwrap_or_default();
    let mut user_settings: UserSettings = read_file(configuration_directory.join("settings.json")).unwrap_or_default();
    let filter: Filter = read_file(configuration_directory.join("filters.json")).unwrap_or_default();
    let servers_sources: Vec<ServersSource> =
        read_file(configuration_directory.join("sources.json")).unwrap_or_else(|error| {
            error!("Failed to read sources.json: {}", error);

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

fn setup_logger() -> Result<(), fern::InitError> {
    let output_log_file_path = get_configuration_directory().join("output.log");

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Off)
        .level_for("teamwork_launcher", log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .chain(
            OpenOptions::new()
                .write(true)
                .create(true)
                .append(false)
                .open(output_log_file_path)?,
        )
        .apply()?;
    Ok(())
}
