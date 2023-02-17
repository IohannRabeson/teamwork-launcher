use {
    serde::{Deserialize, Serialize},
    std::sync::Arc,
};

#[derive(Serialize, Deserialize, Debug, Hash, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SourceKey(Arc<String>);

impl SourceKey {
    pub fn new(key: impl ToString) -> Self {
        Self(Arc::new(key.to_string()))
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServersSource {
    display_name: String,
    url: String,
    key: SourceKey,
    enabled: bool,
}

impl ServersSource {
    pub fn new(display_name: impl ToString, url: impl ToString) -> Self {
        Self {
            display_name: display_name.to_string(),
            url: url.to_string(),
            key: SourceKey(Arc::new(url.to_string())),
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn key(&self) -> &SourceKey {
        &self.key
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
