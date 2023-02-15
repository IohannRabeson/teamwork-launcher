use {
    serde::{Deserialize, Serialize},
    std::{fmt::Display, str::FromStr},
};

#[derive(Debug, Hash, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Country {
    code: String,
}

impl Country {
    pub fn new(code: &impl ToString) -> Self {
        Self { code: code.to_string() }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> String {
        match iso_country::Country::from_str(&self.code) {
            Ok(country) => country.name().to_string(),
            Err(_error) => self.code.clone(),
        }
    }
}

impl Display for Country {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
