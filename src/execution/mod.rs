use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::{
    command::{CommandOutput, CommandRunner, CommandSpec},
    execution::{
        PullOutcome::UpToDate,
        execution_error::ExecutionError,
    },
};

pub mod execution_error;

pub struct NeedsPull;
pub struct NeedsDeploy;
pub enum PullOutcome {
    UpToDate,
    Updated(Execution<NeedsDeploy>),
}

pub struct Execution<State> {
    state: PhantomData<State>,
    directory: PathBuf,
}

impl<State> Execution<State> {
    fn check_command(command: CommandOutput) -> Result<CommandOutput, ExecutionError> {
        if command.status != Some(0) {
            return Err(ExecutionError::CommandFailed {
                status: command.status,
                stderr: command.stderr,
            });
        }
        return Ok(command);
    }
}

impl Execution<NeedsPull> {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, ExecutionError> {
        let directory = path.as_ref().to_path_buf();

        if !directory.is_dir() {
            return Err(ExecutionError::InvalidDirectory(directory));
        }

        Ok(Self {
            state: PhantomData,
            directory,
        })
    }
    pub fn pull<R: CommandRunner>(self, runner: &R) -> Result<PullOutcome, ExecutionError> {
        let head_command = CommandSpec {
            program: "git".into(),
            args: vec!["rev-parse".into(), "HEAD".into()],
        };

        let head_before_output = Self::check_command(runner.run(&head_command, &self.directory)?)?;

        let pull_command = CommandSpec {
            program: "git".into(),
            args: vec!["pull".into()],
        };

        let _ = Self::check_command(runner.run(&pull_command, &self.directory)?)?;

        let head_after_output = Self::check_command(runner.run(&head_command, &self.directory)?)?;

        if head_before_output.stdout.trim() == head_after_output.stdout.trim() {
            return Ok(UpToDate);
        }
        return Ok(PullOutcome::Updated(Execution {
            state: PhantomData,
            directory: self.directory,
        }));
    }
}

pub struct DeploymentPlan {
    pub after_pull: Vec<CommandSpec>,
    pub compose_down: bool,
    pub before_up: Vec<CommandSpec>,
    pub after_up: Vec<CommandSpec>,
}
impl Default for DeploymentPlan {
    fn default() -> Self {
        Self {
            after_pull: Vec::new(),
            compose_down: false,
            before_up: Vec::new(),
            after_up: Vec::new(),
        }
    }
}
impl Execution<NeedsDeploy> {
    fn run_and_check_commands<R: CommandRunner>(
        commands: &[CommandSpec], 
        runner: &R,
        directory : &Path) -> Result<(), ExecutionError> {
        for command in commands {
            Self::check_command(
                runner.run(command, directory)?
            )?;
        }
        Ok(())
    }
    
    pub fn deploy<R: CommandRunner>(
        self,
        runner: &R,
        plan: &DeploymentPlan,
    ) -> Result<(), ExecutionError> {
        Self::run_and_check_commands(
            &plan.after_pull, 
            runner, 
            &self.directory
        )?;

        if plan.compose_down {
            let compose_down_command = CommandSpec {
                program: "docker".into(),
                args: vec!["compose".into(), "down".into()],
            };

            Self::check_command(runner.run(&compose_down_command, &self.directory)?)?;
        }

        Self::run_and_check_commands(
            &plan.before_up, 
            runner, 
            &self.directory
        )?;

        let compose_up_command = CommandSpec {
            program: "docker".into(),
            args: vec!["compose".into(), "up".into(), "-d".into(), "--build".into()],
        };

        Self::check_command(runner.run(&compose_up_command, &self.directory)?)?;
        
        Self::run_and_check_commands(
            &plan.after_up, 
            runner, 
            &self.directory
        )?;

        Ok(())
    }
}
#[cfg(test)]
mod tests;
