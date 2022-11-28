const APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");

pub fn setup_logger() -> anyhow::Result<()> {
    let builder = fern::Dispatch::new()
        .level(log::LevelFilter::Error)
        .level_for(APPLICATION_NAME, log::LevelFilter::Trace);

    #[cfg(debug_assertions)]
    let builder = builder.level_for("teamwork", log::LevelFilter::Trace);

    Ok(builder
        .chain(std::io::stdout())
        .apply()?)
}