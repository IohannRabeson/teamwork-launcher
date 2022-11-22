use std::time::Instant;

use async_trait::async_trait;

use crate::skial_source::SkialSource;
use log::debug;

#[derive(Default)]
pub struct ServersProvider {
    skial_source: SkialSource,
}

/// Store information about a server.
///
/// Currently it's clonable but it could be better to make it "privately clonable" only.
#[derive(Debug, Hash, Clone)]
pub struct Server {
    pub name: String,
    pub max_players_count: u8,
    pub current_players_count: u8,
    pub map: String,
    pub ip: std::net::Ipv4Addr,
    pub port: u16,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            name: Default::default(),
            max_players_count: Default::default(),
            current_players_count: Default::default(),
            map: Default::default(),
            ip: std::net::Ipv4Addr::UNSPECIFIED,
            port: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SourceId(usize);

#[async_trait]
pub trait Source: Send + Sync + 'static {
    fn display_name(&self) -> String;
    async fn get_servers_infos(&self) -> Result<Vec<Server>, GetServersInfosError>;
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
    pub async fn refresh(&self) -> Result<Vec<(Server, SourceId)>, Error> {
        let started = Instant::now();
        let mut servers = Vec::with_capacity(16);

        servers.extend(
            self.skial_source
                .get_servers_infos()
                .await?
                .into_iter()
                .map(|info| (info, SourceId(1usize))),
        );

        debug!("Refresh servers: {}ms", (Instant::now() - started).as_millis());
        Ok(servers)
    }
}
