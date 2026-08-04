use std::path::Path;

pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub enum CommandError {
    IoError(std::io::Error)
}

impl From<std::io::Error> for CommandError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

pub struct CommandOutput {
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner {
    fn run(&self, command: &CommandSpec, directory: &Path)
    -> Result<CommandOutput, CommandError>;
}
