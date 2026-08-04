use std::path::Path;

use crate::command::{CommandError, CommandOutput, CommandRunner, CommandSpec};

pub struct ProcessCommandRunner;

impl CommandRunner for ProcessCommandRunner {
    fn run(
        &self,
        command: &CommandSpec,
        directory: &Path,
    ) -> Result<CommandOutput, CommandError> {
        let output = std::process::Command::new(&command.program)
            .args(&command.args)
            .current_dir(directory)
            .output()?;

        Ok(CommandOutput {
            status: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}