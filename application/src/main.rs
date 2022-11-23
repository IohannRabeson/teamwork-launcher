use {
    application::Application,
    iced::{Application as IcedApplication, Settings},
    settings::UserSettings,
};

mod application;
mod fonts;
mod icons;
mod launcher;
mod servers;
mod settings;
mod setup;
mod skial_source;
mod states;
mod views;

use {
    application::Flags,
    launcher::ExecutableLauncher,
    log::{debug, error},
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliParameters {
    #[arg(short, long)]
    testing_mode: bool,
}

fn main() -> anyhow::Result<()> {
    let cli_params = CliParameters::parse();

    setup::setup_logger()?;

    debug!("CLI parameters: {:?}", cli_params);

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
