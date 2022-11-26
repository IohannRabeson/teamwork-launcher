use {
    application::{Application, Flags},
    clap::Parser,
    iced::{Application as IcedApplication, Settings},
    launcher::ExecutableLauncher,
    log::{error, warn},
    settings::UserSettings,
};

mod application;
mod fonts;
mod icons;
mod launcher;
mod servers_provider;
mod settings;
mod setup;
mod sources;
mod states;
mod views;
mod models;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliParameters {
    #[arg(short, long)]
    testing_mode: bool,
}

fn main() -> anyhow::Result<()> {
    let cli_params = CliParameters::parse();

    setup::setup_logger()?;

    if cli_params.testing_mode {
        warn!("Testing mode enabled");
    }

    Application::run(Settings::with_flags(Flags {
        settings: load_user_settings(),
        launcher: ExecutableLauncher::new(cli_params.testing_mode),
    }))?;

    Ok(())
}

fn load_user_settings() -> UserSettings {
    let settings = match UserSettings::load_settings() {
        Ok(settings) => settings,
        Err(error) => {
            error!("Unable to load user settings: {}", error);
            UserSettings::default()
        }
    };
    settings
}
