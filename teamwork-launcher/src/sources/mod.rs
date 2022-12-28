use serde::{Deserialize, Serialize};

pub mod servers_sources;
mod teamwork_source;

pub use teamwork_source::TeamworkSource;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct SourceKey(String);

impl SourceKey {
    pub fn new<S: ToString>(key: S) -> Self {
        Self(key.to_string())
    }
}
