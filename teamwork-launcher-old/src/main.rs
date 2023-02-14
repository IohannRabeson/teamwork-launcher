// Prevent a console to pop on Windows
#![windows_subsystem = "windows"]

use {
    application::{Application, Flags},
    clap::Parser,
    iced::{window, Application as IcedApplication, Settings},
    launcher::ExecutableLauncher,
    log::{error, info, warn},
    settings::UserSettings,
};

mod advanced_filter;
mod application;
mod directories;
mod fonts;
mod geolocation;
mod icons;
mod launcher;
mod models;
mod ping_service;
mod process_detection;
mod promised_value;
mod servers_provider;
mod settings;
mod sources;
mod states;
mod text_filter;
mod ui;

const APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");
const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_SHA_SHORT: &str = env!("VERGEN_GIT_SHA_SHORT");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliParameters {
    /// This flag enable the integration test. In this mode
    /// the application starts normally (servers are refreshed and pinged as usual) then the application quits after
    /// an hardcoded amount of time (5 secondes).
    #[arg(long)]
    pub integration_test: bool,
}

fn main() -> anyhow::Result<()> {
    let cli_params = CliParameters::parse();

    directories::create_configuration_directory_if_needed();

    setup::setup_logger()?;

    info!(
        "Configuration directory: {}",
        directories::get_configuration_directory().display()
    );

    if cli_params.integration_test {
        warn!("Integration test mode enabled!");
    }

    Application::run(create_application_settings(cli_params))?;

    Ok(())
}

fn create_application_settings(cli_params: CliParameters) -> Settings<Flags> {
    let mut settings = Settings::with_flags(Flags {
        cli_params,
        settings: load_user_settings(),
        launcher: ExecutableLauncher::new(false),
    });
    let window_settings = settings.flags.settings.get_window_settings();

    settings.exit_on_close_request = false;
    settings.window.size.0 = window_settings.width;
    settings.window.size.1 = window_settings.height;
    settings.window.position = window::Position::Specific(window_settings.x, window_settings.y);

    settings
}

fn load_user_settings() -> UserSettings {
    match UserSettings::load_settings() {
        Ok(settings) => settings,
        Err(error) => {
            error!("Unable to load user settings: {}", error);
            UserSettings::default()
        }
    }
}

mod setup {
    use std::{
        fs::{File, OpenOptions},
        path::Path,
    };

    use crate::directories::get_log_output_file_path;

    pub fn setup_logger() -> anyhow::Result<()> {
        let builder = fern::Dispatch::new()
            .level(log::LevelFilter::Error)
            .level_for("teamwork-launcher-old", log::LevelFilter::Trace)
            .level_for("teamwork", log::LevelFilter::Trace)
            .chain(
                fern::Dispatch::new()
                    .format(|out, message, record| {
                        out.finish(format_args!(
                            "[{}][{}][{}] {}",
                            record.level(),
                            record.target(),
                            chrono::Local::now().format("%H:%M:%S"),
                            message,
                        ))
                    })
                    .level(log::LevelFilter::Trace)
                    .chain(open_log_file(get_log_output_file_path())?)
                    .chain(std::io::stdout()),
            );

        Ok(builder.apply()?)
    }

    fn open_log_file(path: impl AsRef<Path>) -> std::io::Result<File> {
        OpenOptions::new().write(true).create(true).append(false).open(path.as_ref())
    }
}
