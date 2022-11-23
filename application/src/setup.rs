// use crate::launcher::Launcher;

const APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");

pub fn setup_logger() -> anyhow::Result<()> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Error)
        .level_for(APPLICATION_NAME, log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

// pub fn setup_launcher() -> Box<dyn Launcher> {
//     use crate::launcher::DebugLauncher;

//     Box::new(DebugLauncher::default())
// }

// #[cfg(not(debug_assertions))]
// pub fn setup_launcher() -> Box<dyn Launcher> {
//     use crate::launcher::ExecutableLauncher;

//     Box::new(ExecutableLauncher::new(r"C:\Program Files (x86)\Steam\Steam.exe"))
// }
