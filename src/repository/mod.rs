use std::path::PathBuf;

#[cfg(test)]
pub mod tests;

use crate::{
    command::CommandRunner, execution::{
        DeploymentPlan, Execution,
        PullOutcome::{self},
        execution_error::ExecutionError,
    },
};

pub struct Repository {
    pub directory: PathBuf,
    pub deployment_plan: DeploymentPlan,
}

#[derive(Debug)]
pub struct RepositoryFailure {
    pub directory : PathBuf,
    pub error: ExecutionError
}

impl RepositoryFailure {
    pub fn new(directory : PathBuf, error : ExecutionError) -> Self {
        Self { directory, error }
    }
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

    fn run_repository(&self, repository : &Repository) -> Result<(), ExecutionError> {
        let execution = Execution::new(&repository.directory)?;
        let outcome = execution.pull(self.runner)?;
        self.resolve_outcome(outcome, repository)?;
        Ok(())
    }
    
    pub fn run(&self, repositories: &[Repository]) -> Result<(), Vec<RepositoryFailure>> {
        let mut errors = Vec::new();
        for repository in repositories {
            if let Err(e) = self.run_repository(repository) {
                errors.push(
                    RepositoryFailure::new(repository.directory.clone(), e)
                );
            }
        }
        if errors.is_empty() {
            return Ok(())
        }
        Err(errors)
    }
}
