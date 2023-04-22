use {
    crate::application::Server,
    serde::{Deserialize, Serialize},
};

#[derive(Serialize, Deserialize, Default)]
pub struct Blacklist {
    terms: Vec<String>,
}

impl Blacklist {
    pub fn insert(&mut self, term: impl ToString) {
        let term = term.to_string();

        if !self.terms.contains(&term) {
            self.terms.push(term);
        }
    }

    pub fn remove(&mut self, index: usize) {
        self.terms.remove(index);
    }

    pub fn accept(&self, server: &Server) -> bool {
        let ip_port = format!("{}:{}", server.ip_port.ip(), server.ip_port.port());

        for term in &self.terms {
            if ip_port.contains(term) || server.name.contains(term) {
                return false;
            }
        }

        true
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.terms.iter().map(|term| term.as_str())
    }
}
