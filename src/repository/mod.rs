use std::path::PathBuf;

#[cfg(test)]
pub mod tests;

use crate::{
    command::CommandRunner,
    execution::{
        DeploymentPlan, execution_error::ExecutionError,
    },
};

pub struct Repository {
    pub directory: PathBuf,
    pub deployment_plan: DeploymentPlan,
}

pub struct RepositoryManager<'a, R> {
    runner: &'a R,
}

impl<'a, R: CommandRunner> RepositoryManager<'a, R> {
    pub fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R: CommandRunner> RepositoryManager<'a, R> {
    pub fn run(&self, repositories: &[Repository]) -> Result<(), ExecutionError> {
        todo!()
    }
}
