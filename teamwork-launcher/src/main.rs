// Prevent a console to pop on Windows
#![windows_subsystem = "windows"]

use {
    application::{Application, Flags},
    clap::Parser,
    iced::{Application as IcedApplication, Settings},
    launcher::ExecutableLauncher,
    log::{error, info, warn},
    settings::UserSettings,
};

mod announces;
mod application;
mod fonts;
mod geolocation;
mod icons;
mod launcher;
mod models;
mod ping_service;
mod promised_value;
mod advanced_filter;
mod servers_provider;
mod servers_sources;
mod settings;
mod sources;
mod states;
mod text_filter;
mod ui;

const APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");

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

    directories::create_if_needed();
    setup::setup_logger()?;

    if cli_params.integration_test {
        warn!("Integration test mode enabled!");
    }

    Application::run(Settings::with_flags(Flags {
        cli_params,
        settings: load_user_settings(),
        launcher: ExecutableLauncher::new(false),
    }))?;

    Ok(())
}

mod directories {
    use std::path::PathBuf;

    use super::*;

    pub fn create_if_needed() {
        let application_directory_path = get_path();

        if !application_directory_path.exists() {
            info!("Create directory '{}'", application_directory_path.to_string_lossy());

            if let Err(error) = std::fs::create_dir_all(&application_directory_path) {
                error!(
                    "Unable to create application directory '{}': {}",
                    application_directory_path.to_string_lossy(),
                    error
                );
            }
        }
    }

    pub fn get_path() -> PathBuf {
        platform_dirs::AppDirs::new(APPLICATION_NAME.into(), false)
            .map(|dirs| dirs.config_dir)
            .expect("config directory path")
    }

    pub fn get_log_output_file_path() -> PathBuf {
        let mut log_output_path = directories::get_path();

        log_output_path.push(format!("{}.log", APPLICATION_NAME));
        log_output_path
    }

    pub fn get_settings_file_path() -> PathBuf {
        let mut settings_file_path = directories::get_path();

        settings_file_path.push("user_settings.json");
        settings_file_path
    }
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
            .level_for("teamwork-launcher", log::LevelFilter::Trace)
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
