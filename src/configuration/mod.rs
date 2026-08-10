use std::path::PathBuf;

use serde::Deserialize;

#[cfg(test)]
pub mod tests;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub repositories: Vec<RepositoryConfiguration>,
}
#[derive(Debug, Deserialize)]
pub struct CommandConfiguration {
    pub program: String,

    #[serde(default)]
    pub args: Vec<String>,
}
#[derive(Debug, Deserialize)]
pub struct RepositoryConfiguration {
    pub directory: PathBuf,

    #[serde(default)]
    pub compose_down: bool,

    #[serde(default)]
    pub after_pull: Vec<CommandConfiguration>,

    #[serde(default)]
    pub before_up: Vec<CommandConfiguration>,

    #[serde(default)]
    pub after_up: Vec<CommandConfiguration>,
}