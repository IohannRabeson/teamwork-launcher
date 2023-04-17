// Prevent a console to pop on Windows
#![windows_subsystem = "windows"]

use {
    crate::{
        application::{
            filter::filter_servers::Filter,
            paths::{DefaultPathsProvider, PathsProvider, TestPathsProvider},
            servers_source::ServersSource,
            user_settings::WindowSettings,
            Bookmarks, UserSettings,
        },
        common_settings::{read_bin_file, read_file},
    },
    iced::{window::Position, Application, Settings},
    log::{error, info},
    mods_manager::Registry,
    std::{fs::OpenOptions, path::Path},
};

mod application;
mod common_settings;
mod fonts;
mod icons;
mod ui;

const APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");
const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_SHA_SHORT: &str = env!("VERGEN_GIT_SHA");

fn main() -> iced::Result {
    let testing_mode = std::env::args().any(|arg| arg == "--testing-mode");
    let settings = load_settings(testing_mode);
    let configuration_directory = settings.flags.paths.get_configuration_directory();

    std::fs::create_dir_all(&configuration_directory).expect("create configuration directory");
    setup_logger(&configuration_directory).expect("setup logger");
    info!("Teamwork Launcher v{}", APPLICATION_VERSION);
    application::TeamworkLauncher::run(settings)
}

pub struct ApplicationFlags {
    pub bookmarks: Bookmarks,
    pub user_settings: UserSettings,
    pub filter: Filter,
    pub servers_sources: Vec<ServersSource>,
    pub paths: Box<dyn PathsProvider>,
    pub mods: Registry,
    pub testing_mode_enabled: bool,
}

impl Default for ApplicationFlags {
    fn default() -> Self {
        Self {
            bookmarks: Bookmarks::default(),
            user_settings: UserSettings::default(),
            filter: Filter::default(),
            servers_sources: Vec::new(),
            testing_mode_enabled: false,
            paths: Box::new(DefaultPathsProvider::new()),
            mods: Registry::new(),
        }
    }
}

fn load_settings(testing_mode_enabled: bool) -> Settings<ApplicationFlags> {
    let paths: Box<dyn PathsProvider> = match testing_mode_enabled {
        true => Box::new(TestPathsProvider::new()),
        false => Box::new(DefaultPathsProvider::new()),
    };
    let configuration_directory = paths.get_configuration_directory();
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
    let mods = read_bin_file(configuration_directory.join("mods.registry")).unwrap_or_default();

    if let Some(window_settings) = user_settings.window.clone() {
        let mut settings = Settings::with_flags(ApplicationFlags {
            bookmarks,
            user_settings,
            filter,
            servers_sources,
            paths,
            testing_mode_enabled,
            mods,
        });

        settings.window.position = Position::Specific(std::cmp::max(window_settings.window_x, 0), std::cmp::max(window_settings.window_y, 0));
        settings.window.size.0 = std::cmp::max(window_settings.window_width, 800);
        settings.window.size.1 = std::cmp::max(window_settings.window_height, 600);

        settings
    } else {
        user_settings.window = Some(WindowSettings::default());

        Settings::with_flags(ApplicationFlags {
            bookmarks,
            user_settings,
            filter,
            servers_sources,
            paths,
            testing_mode_enabled,
            mods,
        })
    }
}

fn setup_logger(configuration_directory: &Path) -> Result<(), fern::InitError> {
    let output_log_file_path = configuration_directory.join("output.log");

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
