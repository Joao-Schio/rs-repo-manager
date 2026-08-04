use std::path::Path;

use crate::execution::execution_error::ExecutionError;

pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

pub struct CommandOutput {
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner {
    fn run(
        &self,
        command: &CommandSpec,
        directory: &Path,
    ) -> Result<CommandOutput, ExecutionError>;
}
