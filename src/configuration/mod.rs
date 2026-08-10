use std::path::PathBuf;

use serde::Deserialize;

#[cfg(test)]
pub mod tests;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub repositories: Vec<RepositoryConfiguration>,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryConfiguration {
    pub directory: PathBuf,
}