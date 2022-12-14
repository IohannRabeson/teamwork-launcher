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

const GAMEMODE_IDS: [(&str, &str); 9] = [
    ("Payload", "payload"),
    ("Attack / Defend", "attack-defend"),
    ("Capture The Flag", "ctf"),
    ("Control point", "control-point"),
    ("Payload race", "payload-race"),
    ("Control Point Orange", "cp-orange"),
    ("King Of The Hill", "koth"),
    ("Medieval mode", "medieval-mode"),
    ("Mann Versus Machine", "mvm"),
];

impl Default for ServersProvider {
    fn default() -> Self {
        let mut sources: Vec<Box<dyn Source>> = vec![];

        for source in GAMEMODE_IDS
            .into_iter()
            .map(|(name, id)| Box::new(TeamworkSource::new(id, name)))
        {
            sources.push(source)
        }

        Self { sources }
    }
}

impl ServersProvider {
    pub fn get_sources<'a>(&'a self) -> impl Iterator<Item = (String, SourceKey)> + 'a {
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
