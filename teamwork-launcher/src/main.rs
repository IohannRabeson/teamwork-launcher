use iced::{Application, Settings};

mod application;
mod fonts;
mod icons;
mod ui;

const APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");
const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_SHA_SHORT: &str = env!("VERGEN_GIT_SHA_SHORT");

fn main() -> iced::Result {
    application::TeamworkLauncher::run(Settings::default())
}
