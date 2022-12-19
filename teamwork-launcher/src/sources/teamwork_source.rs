use {
    super::SourceKey,
    crate::{
        models::{IpPort, Server, Thumbnail},
        promised_value::PromisedValue,
        servers_provider::{GetServersInfosError, Source},
        settings::UserSettings,
    },
    async_trait::async_trait,
    std::str::FromStr,
    teamwork::{Client as TeamworkClient, UrlWithKey},
};

pub struct TeamworkSource {
    client: TeamworkClient,
    query_url_base: String,
    /// The name displayed
    source_name: String,
}

/// The rational is I do not want the entire application depends on the Teamwork.tf API.
/// So even if it's a bit tedious, I prefer to have a struct Server dedicated to the application
/// to avoid using teamwork::Server.
/// Also that opens the system to extension by adding more source of data.
///
impl From<teamwork::Server> for Server {
    fn from(server: teamwork::Server) -> Self {
        Server {
            name: server.name,
            map: server.map_name,
            map_thumbnail: Thumbnail::Loading,
            current_players_count: server.players,
            max_players_count: server.max_players,
            ip_port: IpPort::new(std::net::Ipv4Addr::from_str(&server.ip).expect("parse IP"), server.port),
            source: None,
            country: PromisedValue::Loading,
            ping: PromisedValue::Loading,
        }
    }
}

impl TeamworkSource {
    pub fn new(url: &str, source_name: &str) -> Self {
        Self {
            client: TeamworkClient::default(),
            query_url_base: url.to_string(),
            source_name: source_name.to_string(),
        }
    }
}

#[async_trait]
impl Source for TeamworkSource {
    fn display_name(&self) -> String {
        self.source_name.clone()
    }

    fn unique_key(&self) -> SourceKey {
        SourceKey::new(self.query_url_base.clone())
    }

    async fn get_servers_infos(&self, settings: &UserSettings) -> Result<Vec<Server>, GetServersInfosError> {
        self.client
            .get_servers(UrlWithKey::new(&self.query_url_base, &settings.teamwork_api_key()))
            .await
            .map(|servers: Vec<teamwork::Server>| -> Vec<Server> {
                servers.into_iter().map(|server: teamwork::Server| server.into()).collect()
            })
            .map_err(|error| GetServersInfosError {
                source_name: self.display_name(),
                message: format!("Failed to get servers data: {}", error),
            })
    }
}
