use std::collections::BTreeSet;

use {
    crate::{
        models::Server,
        settings::UserSettings,
        sources::{SourceKey, TeamworkSource},
    },
    async_trait::async_trait,
    log::{debug, error},
    std::time::Instant,
};

#[async_trait]
pub trait Source: Send + Sync {
    fn display_name(&self) -> String;
    fn unique_key(&self) -> SourceKey;
    async fn get_servers_infos(&self, settings: &UserSettings) -> Result<Vec<Server>, GetServersInfosError>;
}

#[derive(Debug, thiserror::Error, Clone)]
#[error("Failed to get servers information from source '{source_name}': {message}")]
pub struct GetServersInfosError {
    pub source_name: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[error(transparent)]
    FailedToGetServerInfo(#[from] GetServersInfosError),
}

pub struct ServersProvider {
    sources: Vec<Box<dyn Source>>,
}

const DEFAULT_TEAMWORK_PROVIDERS: [(&str, &str); 10] = [
    ("Skial", "https://teamwork.tf/api/v1/community/provider/skial/servers"),
    (
        "Blackwonder",
        "https://teamwork.tf/api/v1/community/provider/blackwonder/servers",
    ),
    (
        "Uncletopia",
        "https://teamwork.tf/api/v1/community/provider/uncletopia/servers",
    ),
    (
        "Panda-Community",
        "https://teamwork.tf/api/v1/community/provider/panda-community/servers",
    ),
    ("Otaku", "https://teamwork.tf/api/v1/community/provider/otakugamingtf/servers"),
    (
        "Fire Friendly",
        "https://teamwork.tf/api/v1/community/provider/fire_friendly/servers",
    ),
    (
        "Jump Academy",
        "https://teamwork.tf/api/v1/community/provider/jumpacademy/servers",
    ),
    (
        "Games For Life",
        "https://teamwork.tf/api/v1/community/provider/gamesforlifegfl/servers",
    ),
    (
        "Spaceship Servers",
        "https://teamwork.tf/api/v1/community/provider/spaceshipservers/servers",
    ),
    (
        "Leaders Of the Old School",
        "https://teamwork.tf/api/v1/community/provider/leadersoftheoldschool/servers",
    ),
];

impl Default for ServersProvider {
    fn default() -> Self {
        let mut sources: Vec<Box<dyn Source>> = vec![];

        for source in DEFAULT_TEAMWORK_PROVIDERS
            .into_iter()
            .map(|(name, base_url)| Box::new(TeamworkSource::new(base_url, name)))
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

async fn fetch_servers(source: &Box<dyn Source>, settings: &UserSettings, servers: &mut Vec<Server>) {
    let source_key = source.unique_key();
    match source.get_servers_infos(settings).await {
        Ok(new_servers) => servers.extend(new_servers.into_iter().map(|mut info| {
            info.source = Some(source_key.clone());
            info
        })),
        Err(error) => {
            error!("Get servers failed for source '{}': {}", source.display_name(), error)
        }
    };
}
