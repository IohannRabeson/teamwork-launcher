use std::str::FromStr;

use {
    async_trait::async_trait,
    teamwork::{Client as TeamworkClient},
};

use crate::{
    servers::{GetServersInfosError, Server, Source},
    settings::UserSettings,
};

pub struct TeamworkSource {
    client: TeamworkClient,
}

impl Default for TeamworkSource {
    fn default() -> Self {
        Self {
            client: TeamworkClient::default(),
        }
    }
}

#[async_trait]
impl Source for TeamworkSource {
    fn display_name(&self) -> String {
        "Teamwork.tf".into()
    }

    async fn get_servers_infos(&self, settings: &UserSettings) -> Result<Vec<Server>, GetServersInfosError> {
        self.client.get_servers(&settings.teamwork_api_key(), "payload").await
        .map(
            |servers: Vec<teamwork::Server>| -> Vec<Server> {
                servers
                    .into_iter()
                    .map(|server: teamwork::Server| Server {
                        name: server.name,
                        map: server.map_name,
                        current_players_count: server.players,
                        max_players_count: server.max_players,
                        port: server.port,
                        ip: std::net::Ipv4Addr::from_str(&server.ip).expect("parse IP"),
                    })
                    .collect()
            },
        )
        .map_err(|error| GetServersInfosError {
            source_name: self.display_name(),
            message: format!("Failed to get servers data: {}", error),
        })
    }
}
