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
mod models;
mod text_filter;
mod servers_provider;
mod settings;
mod sources;
mod states;
mod ui;

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
        warn!("Testing mode enabled!");
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

mod setup {
    const APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");

    pub fn setup_logger() -> anyhow::Result<()> {
        let builder = fern::Dispatch::new()
            .level(log::LevelFilter::Error)
            .level_for(APPLICATION_NAME, log::LevelFilter::Trace);

        #[cfg(debug_assertions)]
        let builder = builder.level_for("teamwork", log::LevelFilter::Trace);

        Ok(builder.chain(std::io::stdout()).apply()?)
    }
}
