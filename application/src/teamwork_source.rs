use std::{str::FromStr, time::Duration};

use {async_trait::async_trait, teamwork::Client as TeamworkClient};

use crate::{
    servers::{GetServersInfosError, Server, Source},
    settings::UserSettings,
};

const GAMEMODE_IDS: &[&str] = &[
    "payload",
    "attack-defend",
    "ctf",
    "control-point",
    "payload-race",
    "cp-orange",
];

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
            current_players_count: server.players,
            max_players_count: server.max_players,
            port: server.port,
            ip: std::net::Ipv4Addr::from_str(&server.ip).expect("parse IP"),
        }
    }
}

impl TeamworkSource {
    async fn get_servers_info(
        &self,
        gamemode_id: &str,
        client: &TeamworkClient,
        settings: &UserSettings,
    ) -> Result<Vec<Server>, GetServersInfosError> {
        client
            .get_servers(&settings.teamwork_api_key(), gamemode_id)
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

const PAUSE_BETWEEN_REQUESTS: Duration = Duration::from_secs(1);

#[async_trait]
impl Source for TeamworkSource {
    fn display_name(&self) -> String {
        "Teamwork.tf".into()
    }

    async fn get_servers_infos(&self, settings: &UserSettings) -> Result<Vec<Server>, GetServersInfosError> {
        let mut servers = Vec::new();

        for (index, gamemode_id) in GAMEMODE_IDS.iter().enumerate() {
            servers.extend(self.get_servers_info(gamemode_id, &self.client, settings).await?);

            // Cooldown between two requests to teamwork.tf.
            // It seems the server does not like to be spammed and I have to space the requests.
            if index + 1 < GAMEMODE_IDS.len() {
                async_std::task::sleep(PAUSE_BETWEEN_REQUESTS).await;
            }
        }

        Ok(servers)
    }
}
