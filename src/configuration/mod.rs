use std::path::PathBuf;

use serde::Deserialize;

use crate::{command::CommandSpec, execution::DeploymentPlan, repository::Repository};

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
        self.repositories
            .into_iter()
            .map(Into::into)
            .collect()
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

                after_up: configuration
                    .after_up
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            },
        }
    }
}