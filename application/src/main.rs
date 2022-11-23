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

use log::error;

fn main() -> anyhow::Result<()> {
    setup::setup_logger()?;

    let settings = match UserSettings::load_settings() {
        Ok(settings) => settings,
        Err(error) => {
            error!("Unable to load user settings: {}", error);
            UserSettings::default()
        }
    };

    Application::run(Settings::with_flags(settings))?;

    Ok(())
}
