use {
    serde::{Deserialize, Serialize},
    std::fmt::{Display, Formatter},
};

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct MapName(String);

impl MapName {
    pub fn new(name: impl ToString) -> Self {
        Self(name.to_string())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for MapName {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
