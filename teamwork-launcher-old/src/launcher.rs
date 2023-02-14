use {
    crate::{models::IpPort, process_detection::ProcessDetection},
    std::error::Error,
};

#[derive(thiserror::Error, Debug)]

pub enum LaunchError {
    #[error("Failed to start executable: {executable_path}.\n{origin}")]
    CantStartProcess {
        executable_path: String,
        origin: Box<dyn Error>,
    },
    #[error("The game is already started. You can copy the connection string then paste it in the console in game.")]
    AlreadyStarted,
}

trait Launcher {
    fn launch_game(
        &self,
        executable_path: &str,
        params: &IpPort,
        arguments: &[ExecutableArgument],
    ) -> Result<(), LaunchError>;
}

#[derive(Default)]
struct GameLauncher {
    process_detection: ProcessDetection,
}

impl Launcher for GameLauncher {
    fn launch_game(
        &self,
        executable_path: &str,
        params: &IpPort,
        arguments: &[ExecutableArgument],
    ) -> Result<(), LaunchError> {
        use std::process::Command;

        if self.process_detection.is_game_detected() {
            return Err(LaunchError::AlreadyStarted);
        }

        Command::new(executable_path)
            .args(arguments.iter().map(|arg| arg.format_to_string(params)))
            .output()
            .map_err(|error| LaunchError::CantStartProcess {
                executable_path: executable_path.to_string(),
                origin: Box::new(error),
            })?;

        Ok(())
    }
}

#[derive(Default)]
struct DebugLauncher;

impl Launcher for DebugLauncher {
    fn launch_game(
        &self,
        executable_path: &str,
        params: &IpPort,
        arguments: &[ExecutableArgument],
    ) -> Result<(), LaunchError> {
        println!("Debug launcher launch: {:?}", params);
        println!("Executable: {:?}", executable_path);
        println!("Arguments: {:?}", arguments);

        Ok(())
    }
}

#[derive(Debug)]
enum ExecutableArgument {
    Argument(String),
    Server,
}

impl ExecutableArgument {
    pub fn format_to_string(&self, ip_port: &IpPort) -> String {
        match self {
            ExecutableArgument::Argument(argument) => argument.clone(),
            ExecutableArgument::Server => format!("{}:{}", ip_port.ip(), ip_port.port()),
        }
    }
}

impl From<&str> for ExecutableArgument {
    fn from(s: &str) -> Self {
        Self::Argument(s.to_string())
    }
}

pub struct ExecutableLauncher {
    arguments: Vec<ExecutableArgument>,
    launcher: Box<dyn Launcher>,
}

impl ExecutableLauncher {
    pub fn new(enable_debug_mode: bool) -> Self {
        Self {
            arguments: vec![
                "-applaunch".into(),
                "440".into(),
                "+connect".into(),
                ExecutableArgument::Server,
            ],
            launcher: match enable_debug_mode {
                true => Box::<DebugLauncher>::default(),
                false => Box::<GameLauncher>::default(),
            },
        }
    }

    pub fn launch(&self, executable_path: &str, params: &IpPort) -> Result<(), LaunchError> {
        self.launcher.launch_game(executable_path, params, &self.arguments)
    }
}
