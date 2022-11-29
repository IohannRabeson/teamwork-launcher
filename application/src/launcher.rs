use {crate::models::IpPort, std::error::Error};

#[derive(thiserror::Error, Debug)]
#[error("Failed to start executable: {message}")]
pub struct LaunchError {
    pub message: String,
    pub origin: Option<Box<dyn Error>>,
    pub params: IpPort,
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
struct GameLauncher;

impl Launcher for GameLauncher {
    fn launch_game(
        &self,
        executable_path: &str,
        params: &IpPort,
        arguments: &[ExecutableArgument],
    ) -> Result<(), LaunchError> {
        use std::process::Command;

        Command::new(executable_path)
            .args(arguments.iter().map(|arg| arg.format_to_string(params)))
            .output()
            .map_err(|error| LaunchError {
                message: format!("Cannot start executable '{0}'", executable_path),
                origin: Some(Box::new(error)),
                params: params.clone(),
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
    pub fn new(testing: bool) -> Self {
        Self {
            arguments: vec![
                "-applaunch".into(),
                "440".into(),
                "+connect".into(),
                ExecutableArgument::Server,
            ],
            launcher: match testing {
                true => Box::new(DebugLauncher::default()),
                false => Box::new(GameLauncher::default()),
            },
        }
    }

    pub fn launch(&self, executable_path: &str, params: &IpPort) -> Result<(), LaunchError> {
        self.launcher.launch_game(executable_path, params, &self.arguments)
    }
}
