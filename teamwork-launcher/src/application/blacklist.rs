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
        let ip_port = server.ip_port.to_string();

        for term in &self.terms {
            if ip_port.contains(term) || server.name.contains(term) {
                return false;
            }
        }

        true
    }

    pub fn index_of(&self, text: &String) -> Option<usize> {
        self.terms.iter().position(|term| term == text)
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.terms.iter().map(|term| term.as_str())
    }
}
