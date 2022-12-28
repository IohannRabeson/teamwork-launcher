use std::fmt::Display;

/// An URL that does not leak your API key when you print it.
/// Use UrlWithKey.make_url() to get the final URL.
/// When displayed the API key is hidden.
pub struct UrlWithKey {
    url: String,
    api_key: String,
}

impl UrlWithKey {
    pub fn new(url: impl ToString, api_key: &str) -> UrlWithKey {
        Self {
            url: url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn make_url(&self) -> String {
        format!("{}?key={}", self.url, self.api_key)
    }
}

impl Display for UrlWithKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?key=***", self.url)
    }
}
