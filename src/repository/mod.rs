use std::path::PathBuf;

#[cfg(test)]
pub mod tests;

use crate::{
    command::CommandRunner,
    execution::{
        DeploymentPlan, Execution,
        PullOutcome::{self},
        execution_error::ExecutionError,
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
    // compiler is going to inline it on release anyways
    fn resolve_outcome(
        &self,
        outcome: PullOutcome,
        repository: &Repository,
    ) -> Result<(), ExecutionError> {
        match outcome {
            PullOutcome::UpToDate => Ok(()),
            PullOutcome::Updated(execution) => {
                execution.deploy(self.runner, &repository.deployment_plan)
            }
        }
    }

    pub fn run(&self, repositories: &[Repository]) -> Result<(), ExecutionError> {
        for repository in repositories {
            let execution = Execution::new(&repository.directory)?;
            let outcome = execution.pull(self.runner)?;
            self.resolve_outcome(outcome, repository)?;
        }
        Ok(())
    }
}
