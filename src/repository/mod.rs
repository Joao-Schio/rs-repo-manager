use std::path::PathBuf;

#[cfg(test)]
pub mod tests;

use crate::{
    command::CommandRunner, execution::{
        DeploymentPlan, Execution, execution_error::ExecutionError,
    }, repository,
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
        for repository in repositories {
            let execution = Execution::new(
                &repository.directory
            )?;
            let _ = execution.pull(self.runner)?;
        }
        Ok(())
    }
}
