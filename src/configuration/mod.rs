use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{command::CommandSpec, execution::DeploymentPlan, repository::Repository};

#[derive(Debug)]
pub enum ConfigurationError {
    Io(std::io::Error),
    InvalidJson(serde_json::Error),
}

impl From<serde_json::Error> for ConfigurationError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidJson(value)
    }
}

impl From<std::io::Error> for ConfigurationError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
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

impl From<CommandConfiguration> for CommandSpec {
    fn from(configuration: CommandConfiguration) -> Self {
        Self {
            program: configuration.program,
            args: configuration.args,
        }
    }
}

impl Configuration {
    pub fn into_repositories(self) -> Vec<Repository> {
        self.repositories.into_iter().map(Into::into).collect()
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigurationError> {
        let json = std::fs::read_to_string(path)?;

        Ok(Self::parse(&json)?)
    }
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl From<RepositoryConfiguration> for Repository {
    fn from(configuration: RepositoryConfiguration) -> Self {
        Self {
            directory: configuration.directory,
            deployment_plan: DeploymentPlan {
                compose_down: configuration.compose_down,

                after_pull: configuration
                    .after_pull
                    .into_iter()
                    .map(Into::into)
                    .collect(),

                before_up: configuration
                    .before_up
                    .into_iter()
                    .map(Into::into)
                    .collect(),

                after_up: configuration.after_up.into_iter().map(Into::into).collect(),
            },
        }
    }
}
