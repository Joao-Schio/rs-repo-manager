use std::path::PathBuf;

use crate::command::CommandError;

#[derive(Debug)]
pub enum ExecutionError {
    Io(std::io::Error),

    InvalidDirectory(PathBuf),

    CommandFailed { status: Option<i32>, stderr: String },

    CommandError(CommandError),
}

impl From<std::io::Error> for ExecutionError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<CommandError> for ExecutionError {
    fn from(error: CommandError) -> Self {
        Self::CommandError(error)
    }
}
