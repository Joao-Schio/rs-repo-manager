use std::path::PathBuf;

#[derive(Debug)]
pub enum ExecutionError {
    Io(std::io::Error),

    InvalidDirectory(PathBuf),

    CommandFailed {
        status: Option<i32>,
        stderr: String,
    },
}

impl From<std::io::Error> for ExecutionError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}