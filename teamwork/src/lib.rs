pub use models::{GameMode, Server};
use {
    self::models::GameModes,
    async_mutex::Mutex,
    log::{error, trace},
    serde::{de::DeserializeOwned, Deserialize},
    std::{collections::BTreeMap, sync::Arc, time::Duration},
    url_with_key::UrlWithKey,
};

mod parsing;
mod url_with_key;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No Teamwork.tf API key. To request an API key, login to teamwork.tf then go to https://teamwork.tf/settings")]
    NoTeamworkApiKey,
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),
    #[error("Failed to get {address} with error: {error}")]
    TeamworkError { address: String, error: String },
}

#[derive(Clone)]
/// Notice the client is Send + Sync and it must stay as is (a unit test checks that).
pub struct Client {
    reqwest: reqwest::Client,
    thumbnail_urls_cache: Arc<Mutex<BTreeMap<String, String>>>,
}

impl Default for Client {
    fn default() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("build reqwest client");

        Self {
            reqwest: client,
            thumbnail_urls_cache: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

#[derive(Deserialize)]
struct TeamworkErrorResponse {
    #[serde(rename = "error")]
    pub message: String,
}

#[derive(Deserialize)]
struct ThumbnailResponse {
    #[serde(rename = "thumbnail")]
    pub url: String,
}

const TEAMWORK_TF_QUICKPLAY_API: &str = "https://teamwork.tf/api/v1/quickplay";
const TEAMWORK_TF_MAP_THUMBNAIL_API: &str = "https://teamwork.tf/api/v1/map-stats/mapthumbnail";

impl Client {
    /// Get the thumbnail for a map.
    pub async fn get_map_thumbnail<I: Send + Sync, F: Fn(Vec<u8>) -> I>(
        &self,
        api_key: &str,
        map_name: &str,
        convert_to_image: F,
    ) -> Result<I, Error> {
        if api_key.is_empty() {
            return Err(Error::NoTeamworkApiKey);
        }

        let image_url = self.get_map_thumbnail_url(api_key, map_name).await?;
        let bytes = self.reqwest.get(image_url).send().await?.bytes().await?;

        Ok(convert_to_image(bytes.as_ref().to_vec()))
    }

    pub async fn get_map_thumbnail_url(&self, api_key: &str, map_name: &str) -> Result<String, Error> {
        if api_key.is_empty() {
            return Err(Error::NoTeamworkApiKey);
        }
        let map_name = map_name.to_string();
        let image_url = match self.thumbnail_urls_cache.lock().await.get(&map_name) {
            Some(thumbnail_url) => thumbnail_url.clone(),
            None => {
                let query_url = UrlWithKey::new(format!("{}/{}", TEAMWORK_TF_MAP_THUMBNAIL_API, map_name), api_key);

                self.get::<ThumbnailResponse>(&query_url).await?.url
            }
        };

        Ok(image_url)
    }

    pub async fn get_gamemodes(&self, api_key: &str) -> Result<Vec<GameMode>, Error> {
        let url = UrlWithKey::new(TEAMWORK_TF_QUICKPLAY_API, api_key);
        let modes: GameModes = self.get(&url).await?;
        let mut game_modes: Vec<GameMode> = Vec::new();

        game_modes.extend(modes.official);
        game_modes.extend(modes.community);

        Ok(game_modes)
    }

    pub async fn get_servers(&self, api_key: &str, game_mode_id: &str) -> Result<Vec<Server>, Error> {
        if api_key.is_empty() {
            return Err(Error::NoTeamworkApiKey);
        }

        let url = UrlWithKey::new(format!("{}/{}/servers", TEAMWORK_TF_QUICKPLAY_API, game_mode_id), api_key);
        let mut servers: Vec<Server> = self.get(&url).await?;

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

    fn filter_char(c: &char) -> bool {
        c.is_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_punctuation() || c.is_ascii_whitespace()
    }

    async fn get<'a, T: DeserializeOwned + Send + Sync + Sized>(&self, url: &UrlWithKey) -> Result<T, Error> {
        trace!("GET '{}'", url);

        let raw_text = self
            .reqwest
            .get(url.make_url())
            .send()
            .await
            .map_err(Error::HttpRequest)?
            .text()
            .await?;

        Self::try_parse_response::<T>(&raw_text, url)
    }

    /// Try to parse the value T from JSON.
    /// In case of failure, try to parse the same text but assuming the JSON contains an error (TeamworkErrorResponse).
    /// If this also fails, just return the original error as JSON error.
    fn try_parse_response<'a, T: Deserialize<'a>>(text: &'a str, url: &UrlWithKey) -> Result<T, Error> {
        match serde_json::from_str::<'a, T>(&text) {
            Ok(value) => Ok(value),
            Err(json_error) => {
                match serde_json::from_str::<TeamworkErrorResponse>(&text) {
                    Ok(error) => Err(Error::TeamworkError {
                        address: url.to_string(),
                        error: error.message.clone(),
                    }),
                    Err(_error) => {
                        trace!("Failed to parse JSON: {}", text);

                        // Failed to parse the teamwork error, ignore the last error and return the original json error.
                        Err(Error::Json(json_error))
                    }
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Client>();
        assert_sync::<Client>();
    }
}
