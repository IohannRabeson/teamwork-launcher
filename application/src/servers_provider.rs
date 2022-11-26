use {async_trait::async_trait, log::debug, std::time::Instant};

use crate::{
    settings::UserSettings,
    sources::{SkialSource, TeamworkSource}, models::{Server, SourceId},
};
pub struct ServersProvider {
    sources: Vec<Box<dyn Source>>,
}

impl Default for ServersProvider {
    fn default() -> Self {
        Self {
            sources: vec![Box::new(SkialSource::default()), Box::new(TeamworkSource::default())],
        }
    }
}

#[async_trait]
pub trait Source: Send + Sync + 'static {
    fn display_name(&self) -> String;
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

impl ServersProvider {
    pub async fn refresh(&self, settings: &UserSettings) -> Result<Vec<Server>, Error> {
        let started = Instant::now();
        let mut servers = Vec::with_capacity(16);

        for (index, source) in self.sources.iter().enumerate() {
            servers.extend(
                source
                    .get_servers_infos(&settings)
                    .await?
                    .into_iter()
                    .map(|mut info| {
                        info.source = Some(SourceId(index));
                        info
                     }),
            );
        }

        debug!("Refresh servers: {}ms", (Instant::now() - started).as_millis());

        Ok(servers)
    }
}
