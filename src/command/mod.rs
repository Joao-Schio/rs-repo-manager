use std::path::{Path, PathBuf};

use crate::execution::execution_error::ExecutionError;
#[cfg(test)]
pub mod fake_runner;
#[cfg(test)]
pub mod command_tests;

pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

pub struct CommandOutput {
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
struct RecordedCommand {
    program: String,
    args: Vec<String>,
    directory: PathBuf,
}


pub trait CommandRunner {
    fn run(
        &self,
        command: &CommandSpec,
        directory: &Path,
    ) -> Result<CommandOutput, ExecutionError>;
}