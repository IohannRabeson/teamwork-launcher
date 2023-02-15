use iced::{Application, Settings};

mod application;
mod fonts;
mod icons;
mod ui;

fn main() -> iced::Result {
    application::TeamworkLauncher::run(Settings::default())
}
