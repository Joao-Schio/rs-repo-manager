use std::{
    cell::RefCell, path::{Path, PathBuf},
};

use crate::{command::{
    CommandOutput, CommandRunner, CommandSpec, RecordedCommand,
}, execution::execution_error::ExecutionError};

struct FakeCommandRunner {
    commands: RefCell<Vec<RecordedCommand>>,
}

impl FakeCommandRunner {
    fn new() -> Self {
        Self {
            commands: RefCell::new(Vec::new()),
        }
    }
}

impl CommandRunner for FakeCommandRunner {
    fn run(
        &self,
        command: &CommandSpec,
        directory: &Path,
    ) -> Result<CommandOutput, ExecutionError> {
        self.commands.borrow_mut().push(
            RecordedCommand {
                program: command.program.clone(),
                args: command.args.clone(),
                directory: directory.to_path_buf(),
            },
        );

        Ok(CommandOutput {
            status: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

