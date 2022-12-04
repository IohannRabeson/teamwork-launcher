use serde::{Deserialize, Serialize};

mod skial_source;
mod teamwork_source;

pub use {skial_source::SkialSource, teamwork_source::TeamworkSource};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct SourceKey(String);

impl SourceKey {
    pub fn new<S: ToString>(key: S) -> Self {
        Self(key.to_string())
    }
}
