use application::Application;
use iced::{Application as IcedApplication, Settings};
use settings::UserSettings;

mod application;
mod fonts;
mod icons;
mod launcher;
mod servers;
mod settings;
mod setup;
mod skial_source;
mod views;
mod states;

fn main() -> anyhow::Result<()> {
    setup::setup_logger()?;

    Application::run(Settings::with_flags(UserSettings::load_settings()?))?;

    Ok(())
}
