use std::{marker::PhantomData, path::{Path, PathBuf}};

use crate::{command::CommandRunner, execution::execution_error::ExecutionError};

pub mod execution_error;

pub struct NeedsPull;
pub struct NeedsDeploy;
pub struct Finished;

pub struct Execution<State> {
    state: PhantomData<State>,
    directory: PathBuf,
}

pub enum PullOutcome {
    UpToDate(Execution<Finished>),
    Updated(Execution<NeedsDeploy>),
}

impl<State> Execution<State> {
    fn transition<Next>(self) -> Execution<Next> {
        Execution {
            state: PhantomData,
            directory: self.directory,
        }
    }
}

impl Execution<NeedsPull> {
    pub fn new(
        path: impl AsRef<Path>,
    ) -> Result<Self, ExecutionError> {
        // validate
        todo!()
    }

    pub fn pull<R: CommandRunner>(
        self,
        runner: &R,
    ) -> Result<PullOutcome, ExecutionError> {
        todo!()
    }
}

impl Execution<NeedsDeploy> {
    pub fn deploy<R: CommandRunner>(
        self,
        runner: &R,
        plan: &DeploymentPlan,
    ) -> Result<Execution<Finished>, ExecutionError> {
        todo!()
    }
}