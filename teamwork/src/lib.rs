use {
    self::models::GameModes,
    async_mutex::Mutex,
    log::{error, trace},
    serde::{de::DeserializeOwned, Deserialize},
    std::{collections::BTreeMap, sync::Arc, time::Duration},
};
pub use {
    models::{GameMode, Server},
    url_with_key::UrlWithKey,
};

mod parsing;
mod url_with_key;

#[derive(thiserror::Error, Debug, enum_as_inner::EnumAsInner)]
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
    #[error("Too Many Attempts")]
    TooManyAttempts,
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
    pub url: Option<String>,
}

const TEAMWORK_TF_QUICKPLAY_API: &str = "https://teamwork.tf/api/v1/quickplay";
const TEAMWORK_TF_MAP_THUMBNAIL_API: &str = "https://teamwork.tf/api/v1/map-stats/mapthumbnail";
const TEAMWORK_TF_MAP_STATS_API: &str = "https://teamwork.tf/api/v1/map-stats/map";
const TEAMWORK_TOO_MANY_ATTEMPTS: &str = "Too Many Attempts.";

mod map_details_response_json {
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Clone)]
    pub struct MapContext {
        pub screenshots: Vec<String>,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Response {
        #[serde(rename = "map")]
        pub name: String,
        #[serde(rename = "thumbnail")]
        pub thumbnail_url: String,
        pub context: Option<MapContext>,
        #[serde(rename = "official_map")]
        pub is_official: bool,
    }
}


impl Client {
    async fn get_map_details(&self,
                       api_key: &str,
                       map_name: &str) -> Result<map_details_response_json::Response, Error>
    {
        let query_url = UrlWithKey::new(format!("{}/{}", TEAMWORK_TF_MAP_STATS_API, map_name), api_key);
        let raw_text = self.get_raw_text(&query_url).await?;

        if raw_text == TEAMWORK_TOO_MANY_ATTEMPTS {
            return Err(Error::TooManyAttempts)
        }

        Self::try_parse_response::<map_details_response_json::Response>(&raw_text, &query_url)
    }

    pub async fn get_map_screenshots<I: Send + Sync, F: Fn(Vec<u8>) -> I>(
        &self,
        api_key: &str,
        map_name: &str,
        convert_to_image: F) -> Result<Vec<I>, Error>
    {
        let details = self.get_map_details(api_key, map_name).await?;
        let mut images = Vec::new();

        if let Some(context) = details.context {
            for screenshot_url in context.screenshots {
                images.push(self.get_image(&screenshot_url, &convert_to_image).await?);
            }
        }

        Ok(images)
    }

    /// Get the thumbnail for a map.
    pub async fn get_map_thumbnail<I: Send + Sync, F: Fn(Vec<u8>) -> I>(
        &self,
        api_key: &str,
        map_name: &str,
        convert_to_image: F,
    ) -> Result<Option<I>, Error> {
        if api_key.is_empty() {
            return Err(Error::NoTeamworkApiKey);
        }

        let image_url = self.get_map_thumbnail_url(api_key, map_name).await?;

        Ok(match image_url {
            Some(image_url) => {
                Some(self.get_image(&image_url, &convert_to_image).await?)
            }
            None => {
                None
            }
        })
    }

    async fn get_image<I: Send + Sync, F: Fn(Vec<u8>) -> I>(&self, url: &str, convert_to_image: &F) -> Result<I, Error> {
        let bytes = self.reqwest.get(url).send().await?.bytes().await?;

        Ok(convert_to_image(bytes.as_ref().to_vec()))
    }

    pub async fn get_map_thumbnail_url(&self, api_key: &str, map_name: &str) -> Result<Option<String>, Error> {
        if api_key.is_empty() {
            return Err(Error::NoTeamworkApiKey);
        }
        let map_name = map_name.to_string();
        let image_url = match self.thumbnail_urls_cache.lock().await.get(&map_name) {
            Some(thumbnail_url) => Some(thumbnail_url.clone()),
            None => {
                let query_url = UrlWithKey::new(format!("{}/{}", TEAMWORK_TF_MAP_THUMBNAIL_API, map_name), api_key);

                self.get_thumbnail_response(&query_url).await?.url
            }
        };

        Ok(image_url)
    }

    pub async fn get_game_modes(&self, api_key: &str) -> Result<Vec<GameMode>, Error> {
        let url = UrlWithKey::new(TEAMWORK_TF_QUICKPLAY_API, api_key);
        let modes: GameModes = self.get(&url).await?;
        let mut game_modes: Vec<GameMode> = Vec::new();

        game_modes.extend(modes.official);
        game_modes.extend(modes.community);

        Ok(game_modes)
    }

    pub async fn get_servers(&self, url: UrlWithKey) -> Result<Vec<Server>, Error> {
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

    async fn get_raw_text(&self, url: &UrlWithKey) -> Result<String, Error> {
        trace!("GET '{}'", url);

        Ok(self
            .reqwest
            .get(url.make_url())
            .send()
            .await
            .map_err(Error::HttpRequest)?
            .text()
            .await?)
    }

    async fn get<'a, T: DeserializeOwned + Send + Sync + Sized>(&self, url: &UrlWithKey) -> Result<T, Error> {
        let raw_text = self.get_raw_text(url).await?;

        Self::try_parse_response::<T>(&raw_text, url)
    }

    async fn get_thumbnail_response(&self, url: &UrlWithKey) -> Result<ThumbnailResponse, Error> {
        let raw_text = self.get_raw_text(url).await?;

        if raw_text == TEAMWORK_TOO_MANY_ATTEMPTS {
            return Err(Error::TooManyAttempts)
        }

        Self::try_parse_response::<ThumbnailResponse>(&raw_text, url)
    }

    /// Try to parse the value T from JSON.
    /// In case of failure, try to parse the same text but assuming the JSON contains an error (TeamworkErrorResponse).
    /// If this also fails, just return the original error as JSON error.
    fn try_parse_response<'a, T: Deserialize<'a>>(text: &'a str, url: &UrlWithKey) -> Result<T, Error> {
        match serde_json::from_str::<'a, T>(text) {
            Ok(value) => Ok(value),
            Err(json_error) => {
                match serde_json::from_str::<TeamworkErrorResponse>(text) {
                    Ok(error) => Err(Error::TeamworkError {
                        address: url.to_string(),
                        error: error.message,
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
        #[serde(rename = "gamemodes")]
        pub game_modes: Vec<String>,
        #[serde(rename = "gametype")]
        pub game_type: String,
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
