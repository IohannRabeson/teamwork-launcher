use std::{
    error::Error,
    ffi::{OsStr, OsString},
};

#[derive(Debug, Clone)]
pub struct LaunchParams {
    pub server_ip: std::net::Ipv4Addr,
    pub server_port: u16,
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to start executable: {message}")]
pub struct LaunchError {
    message: String,
    origin: Option<Box<dyn Error>>,
    params: LaunchParams,
}

pub trait Launcher {
    fn launch(&self, params: &LaunchParams) -> Result<(), LaunchError>;
}

#[derive(Default)]
pub struct DebugLauncher;

impl Launcher for DebugLauncher {
    fn launch(&self, params: &LaunchParams) -> Result<(), LaunchError> {
        println!("Debug launcher launch: {:?}", params);

        Ok(())
    }
}

pub enum ExecutableArgument {
    Argument(String),
    Server,
}

impl ExecutableArgument {
    pub fn format_to_string(&self, params: &LaunchParams) -> String {
        match self {
            ExecutableArgument::Argument(argument) => argument.clone(),
            ExecutableArgument::Server => format!("{}:{}", params.server_ip, params.server_port),
        }
    }
}

impl From<&str> for ExecutableArgument {
    fn from(s: &str) -> Self {
        Self::Argument(s.to_string())
    }
}

pub struct ExecutableLauncher {
    executable_path: OsString,
    arguments: Vec<ExecutableArgument>,
}

impl ExecutableLauncher {
    pub fn new<P: AsRef<OsStr>>(executable_path: P) -> Self {
        Self {
            arguments: vec![
                "-applaunch".into(),
                "440".into(),
                "+connect".into(),
                ExecutableArgument::Server,
            ],
            executable_path: executable_path.as_ref().to_os_string(),
        }
    }
}

impl Launcher for ExecutableLauncher {
    fn launch(&self, params: &LaunchParams) -> Result<(), LaunchError> {
        use std::process::Command;

        // r"C:\Program Files (x86)\Steam\Steam.exe"
        Command::new(&self.executable_path)
            .args(self.arguments.iter().map(|arg| arg.format_to_string(params)))
            .output()
            .map_err(|error| LaunchError {
                message: format!("Can start executable '{0}'", &self.executable_path.to_string_lossy()),
                origin: Some(Box::new(error)),
                params: params.clone(),
            })?;

        Ok(())
    }
}
