use {
    crate::{
        directories,
        models::Server,
        settings::UserSettings,
        sources::{SourceKey, TeamworkSource},
    },
    async_trait::async_trait,
    log::{debug, error},
    serde::{Deserialize, Serialize},
    std::{collections::BTreeSet, path::PathBuf, time::Instant},
};

#[async_trait]
pub trait Source: Send + Sync {
    fn display_name(&self) -> String;
    fn unique_key(&self) -> SourceKey;
    async fn get_servers_info(&self, settings: &UserSettings) -> Result<Vec<Server>, GetServersInfoError>;
}

#[derive(Debug, thiserror::Error, Clone)]
#[error("Failed to get servers information from source '{source_name}': {message}")]
pub struct GetServersInfoError {
    pub source_name: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[error(transparent)]
    FailedToGetServerInfo(#[from] GetServersInfoError),
}

pub struct ServersProvider {
    sources: Vec<Box<dyn Source>>,
}

const DEFAULT_TEAMWORK_PROVIDERS: [(&str, &str); 7] = [
    ("https://teamwork.tf/api/v1/quickplay/attack-defend/servers", "Attack/Defend"),
    ("https://teamwork.tf/api/v1/quickplay/ctf/servers", "Capture the Flag"),
    ("https://teamwork.tf/api/v1/quickplay/control-point/servers", "Control Points"),
    ("https://teamwork.tf/api/v1/quickplay/koth/servers", "King of the Hill"),
    ("https://teamwork.tf/api/v1/quickplay/payload/servers", "Payload"),
    ("https://teamwork.tf/api/v1/quickplay/payload-race/servers", "Payload Race"),
    ("https://teamwork.tf/api/v1/quickplay/medieval-mode/servers", "Medieval Mode"),
];

#[derive(thiserror::Error, Debug)]
enum ConfigurationError {
    #[error("Unable to read providers configuration '{0}': {1}")]
    ReadFile(PathBuf, std::io::Error),
    #[error("Unable to parse providers configuration '{0}': {1}")]
    ParseJson(PathBuf, serde_json::Error),
    #[error("Unable to write providers configuration '{0}': {1}")]
    WriteFile(PathBuf, std::io::Error),
    #[error("Unable to format JSON for providers configuration: {0}")]
    FormatJson(serde_json::Error),
}

#[derive(Serialize, Deserialize)]
struct ProviderEntry {
    pub url: String,
    pub display_name: String,
}

#[derive(Serialize, Deserialize)]
struct Configuration {
    pub entries: Vec<ProviderEntry>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            entries: DEFAULT_TEAMWORK_PROVIDERS
                .iter()
                .map(|(url, name)| ProviderEntry {
                    display_name: name.to_string(),
                    url: url.to_string(),
                })
                .collect(),
        }
    }
}

impl Configuration {
    pub fn load_file() -> Result<Configuration, ConfigurationError> {
        let global_config_file_path = directories::get_providers_file_path();
        let content = std::fs::read_to_string(&global_config_file_path)
            .map_err(|e| ConfigurationError::ReadFile(global_config_file_path.clone(), e))?;

        serde_json::from_str(&content).map_err(|e| ConfigurationError::ParseJson(global_config_file_path, e))
    }

    pub fn write_file(&self) -> Result<(), ConfigurationError> {
        let global_config_file_path = directories::get_providers_file_path();
        let content = serde_json::to_string_pretty(self).map_err(ConfigurationError::FormatJson)?;

        std::fs::write(&global_config_file_path, content)
            .map_err(|e| ConfigurationError::WriteFile(global_config_file_path.clone(), e))
    }
}

impl Default for ServersProvider {
    fn default() -> Self {
        // Try to load a configuration file.
        // If it succeed, all good, otherwise, it create a default configuration
        // then write the file at the expected location.
        let configuration = match Configuration::load_file() {
            Ok(configuration) => configuration,
            Err(error) => {
                error!("{}", error);

                Configuration::default()
            }
        };

        if let Err(error) = configuration.write_file() {
            error!("Failed to write providers configuration: {}", error);
        }

        let mut sources: Vec<Box<dyn Source>> = Vec::new();

        for source in configuration
            .entries
            .iter()
            .map(|entry| Box::new(TeamworkSource::new(&entry.url, &entry.display_name)))
        {
            sources.push(source)
        }

        Self { sources }
    }
}

impl ServersProvider {
    pub fn get_sources(&self) -> impl Iterator<Item = (String, SourceKey)> + '_ {
        self.sources.iter().map(|source| (source.display_name(), source.unique_key()))
    }

    pub async fn refresh_some(
        &self,
        settings: &UserSettings,
        source_keys: &BTreeSet<SourceKey>,
    ) -> Result<Vec<Server>, Error> {
        let started = Instant::now();
        let mut servers = Vec::with_capacity(16);

        for source in self
            .sources
            .iter()
            .filter(|source| source_keys.contains(&source.unique_key()))
        {
            fetch_servers(source, settings, &mut servers).await;
        }

        debug!("Refresh servers: {}ms", (Instant::now() - started).as_millis());

        Ok(servers)
    }
}

async fn fetch_servers(source: impl AsRef<dyn Source>, settings: &UserSettings, servers: &mut Vec<Server>) {
    let source = source.as_ref();
    let source_key = source.unique_key();
    match source.get_servers_info(settings).await {
        Ok(new_servers) => servers.extend(new_servers.into_iter().map(|mut info| {
            info.source = Some(source_key.clone());
            info
        })),
        Err(error) => {
            error!("Get servers failed for source '{}': {}", source.display_name(), error)
        }
    };
}
