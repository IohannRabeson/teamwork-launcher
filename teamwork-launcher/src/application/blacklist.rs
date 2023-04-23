use {
    crate::application::Server,
    rfd::AsyncFileDialog,
    serde::{Deserialize, Serialize},
    std::path::PathBuf,
    tokio::io::{AsyncBufReadExt, BufReader},
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
    pub fn clear(&mut self) {
        self.terms.clear();
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

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum ImportBlacklistError {
    #[error("Failed to import blacklist file {0}: {1}")]
    Io(PathBuf, String),
}

pub async fn import_blacklist() -> Result<Vec<String>, ImportBlacklistError> {
    let file_handle = AsyncFileDialog::new().set_directory("/").pick_file().await;
    let mut terms = Vec::new();

    if let Some(file_handle) = file_handle {
        let file_content = tokio::fs::read_to_string(file_handle.path())
            .await
            .map_err(|error| ImportBlacklistError::Io(file_handle.path().to_path_buf(), error.to_string()))?;
        let mut reader = BufReader::new(file_content.as_bytes()).lines();

        while let Some(line) = reader
            .next_line()
            .await
            .map_err(|error| ImportBlacklistError::Io(file_handle.path().to_path_buf(), error.to_string()))?
        {
            if !line.trim().is_empty() {
                terms.push(line);
            }
        }
    }

    Ok(terms)
}
