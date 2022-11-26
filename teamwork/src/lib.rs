use std::time::Duration;

pub use models::{GameMode, Server};
use {self::models::GameModes, serde::Deserialize};

mod parsing;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),
    #[error("Failed to get {address} with error: {error}")]
    TeamworkError { address: String, error: String },
}

/// Notice the client is Send + Sync and it must stay as is.
#[derive(Default)]
pub struct Client;

#[derive(Deserialize)]
struct TeamworkErrorResponse {
    #[serde(rename = "error")]
    pub message: String,
}

const TEAMWORK_TF_API: &str = "https://teamwork.tf/api/v1";

impl Client {
    pub async fn query_gamemodes(&self, api_key: &str) -> Result<Vec<GameMode>, Error> {
        let address = format!("{}/quickplay?key={}", TEAMWORK_TF_API, api_key);
        let response = reqwest::get(&address).await.map_err(Error::HttpRequest)?;
        let raw_text = response.text().await?;

        match serde_json::from_str::<GameModes>(&raw_text) {
            Ok(modes) => {
                let mut game_modes: Vec<GameMode> = Vec::new();

                game_modes.extend(modes.official);
                game_modes.extend(modes.community);

                Ok(game_modes)
            }
            Err(_error) => {
                let error: TeamworkErrorResponse = serde_json::from_str(&raw_text).map_err(Error::Json)?;

                Err(Error::TeamworkError {
                    address: address.clone(),
                    error: error.message,
                })
            }
        }
    }

    fn filter_char(c: &char) -> bool {
        c.is_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_punctuation() || c.is_ascii_whitespace()
    }

    pub async fn get_servers(&self, api_key: &str, game_mode_id: &str) -> Result<Vec<models::Server>, Error> {
        self.query_servers(api_key, game_mode_id).await
    }

    async fn query_servers(&self, api_key: &str, game_mode_id: &str) -> Result<Vec<models::Server>, Error> {
        let address = format!("{}/quickplay/{}/servers?key={}", TEAMWORK_TF_API, game_mode_id, api_key);
        println!("GET '{}'", address);

        let mut servers: Vec<Server> = reqwest::get(address)
            .await
            .map_err(Error::HttpRequest)?
            .json()
            .await
            .map_err(Error::HttpRequest)?;

        for server in &mut servers {
            server.name = server
                .name
                .chars()
                .filter(Self::filter_char)
                .collect::<String>()
                .trim()
                .to_string();
        }

        Ok(servers.into_iter().filter(Server::is_valid).collect())
    }
}

pub mod models {
    use {
        serde::{de, Deserialize, Deserializer, Serialize},
        std::{fmt::Display, str::FromStr},
    };

    #[derive(Serialize, Deserialize, Debug, Clone, Default)]
    pub struct Color {
        pub r: u8,
        pub g: u8,
        pub b: u8,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Default)]
    pub struct Server {
        #[serde(default)]
        pub ip: String,
        #[serde(deserialize_with = "from_str")]
        pub port: u16,
        pub name: String,
        pub reachable: bool,
        #[serde(deserialize_with = "empty_str_is_none")]
        pub provider: Option<String>,
        pub valve_secure: bool,
        pub map_name: String,
        #[serde(deserialize_with = "empty_str_is_none")]
        pub map_name_next: Option<String>,
        pub players: u8,
        pub max_players: u8,
        pub gamemodes: Vec<String>,
        pub gametype: String,
        pub has_password: Option<bool>,
        /// RTD means "Role The Dice", it's a command that gives a random ability to the player.
        pub has_rtd: Option<bool>,
        pub has_randomcrits: Option<bool>,
        pub has_norespawntime: Option<bool>,
        pub has_alltalk: Option<bool>,
    }

    fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        T::from_str(&s).map_err(de::Error::custom)
    }

    fn color_from_str<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;

        match crate::parsing::hex_color(&text) {
            Ok((_, color)) => Ok(Some(color)),
            // I silent the error, I do not care about illformed color
            Err(_) => Ok(None),
        }
    }

    fn empty_str_is_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = Option::<String>::deserialize(deserializer)?.map(|text| text.trim().to_string());

        Ok(match text.is_none() || text.as_ref().unwrap().trim().is_empty() {
            true => None,
            false => text,
        })
    }

    impl Server {
        pub fn is_valid(&self) -> bool {
            !self.ip.is_empty()
        }
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct GameMode {
        pub id: String,
        pub title: String,
        #[serde(rename = "desc")]
        pub description: String,
        #[serde(deserialize_with = "color_from_str")]
        pub color: Option<Color>,
    }

    #[derive(Deserialize, Debug)]
    pub struct GameModes {
        #[serde(rename = "gamemodes_official")]
        pub official: Vec<GameMode>,
        #[serde(rename = "gamemodes_community")]
        pub community: Vec<GameMode>,
    }
}
