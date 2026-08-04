use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::{
    command::{CommandRunner, CommandSpec},
    execution::execution_error::ExecutionError,
};

pub mod execution_error;

pub struct NeedsPull;

pub struct Execution<State> {
    state: PhantomData<State>,
    directory: PathBuf,
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

    pub fn pull<R: CommandRunner>(self, runner: &R) -> Result<(), ExecutionError> {
        let command = CommandSpec {
            program: "git".into(),
            args: vec!["pull".into()],
        };

        let execution = runner.run(&command, &self.directory)?;

        if execution.status != Some(0) {
            return Err(ExecutionError::CommandFailed {
                status: execution.status,
                stderr: execution.stderr,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::RefCell,
        fs,
        path::{Path, PathBuf},
    };

    use crate::command::{CommandError, CommandOutput, CommandRunner, CommandSpec};

    #[derive(Debug)]
    struct RecordedCommand {
        program: String,
        args: Vec<String>,
        directory: PathBuf,
    }

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
        ) -> Result<CommandOutput, CommandError> {
            self.commands.borrow_mut().push(RecordedCommand {
                program: command.program.clone(),
                args: command.args.clone(),
                directory: directory.to_path_buf(),
            });

            Ok(CommandOutput {
                status: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    #[test]
    fn new_accepts_existing_directory() {
        let path = std::env::temp_dir().join("rs_repo_manager_new_accepts_directory");

        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        let result = Execution::<NeedsPull>::new(&path);

        assert!(result.is_ok());

        fs::remove_dir_all(&path).unwrap();
    }

    #[test]
    fn new_rejects_nonexistent_directory() {
        let path = std::env::temp_dir().join("rs_repo_manager_directory_that_does_not_exist");

        let _ = fs::remove_dir_all(&path);

        let result = Execution::<NeedsPull>::new(&path);

        assert!(matches!(result, Err(ExecutionError::InvalidDirectory(_))));
    }

    #[test]
    fn pull_runs_git_pull_in_repository_directory() {
        let path = std::env::temp_dir().join("rs_repo_manager_pull_runs_git_pull");

        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        let execution = Execution::<NeedsPull>::new(&path).unwrap();
        let runner = FakeCommandRunner::new();

        execution.pull(&runner).unwrap();

        let commands = runner.commands.borrow();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].program, "git");
        assert_eq!(commands[0].args, vec!["pull"]);
        assert_eq!(commands[0].directory, path);

        fs::remove_dir_all(&path).unwrap();
    }

    struct FailingCommandRunner;

    impl CommandRunner for FailingCommandRunner {
        fn run(
            &self,
            _command: &CommandSpec,
            _directory: &Path,
        ) -> Result<CommandOutput, CommandError> {
            Err(CommandError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "git executable not found",
            )))
        }
    }
    #[test]
    fn pull_propagates_runner_error() {
        let path = std::env::temp_dir().join("rs_repo_manager_pull_propagates_runner_error");

        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        let execution = Execution::<NeedsPull>::new(&path).unwrap();

        let runner = FailingCommandRunner;

        let result = execution.pull(&runner);

        assert!(matches!(
            result,
            Err(ExecutionError::CommandError(CommandError::IoError(_)))
        ));

        fs::remove_dir_all(&path).unwrap();
    }

    struct UnsuccessfulCommandRunner;

    impl CommandRunner for UnsuccessfulCommandRunner {
        fn run(
            &self,
            _command: &CommandSpec,
            _directory: &Path,
        ) -> Result<CommandOutput, CommandError> {
            Ok(CommandOutput {
                status: Some(1),
                stdout: String::new(),
                stderr: "git failed".into(),
            })
        }
    }

    #[test]
    fn pull_rejects_unsuccessful_command_output() {
        let path = std::env::temp_dir().join("rs_repo_manager_pull_rejects_unsuccessful_output");

        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        let execution = Execution::<NeedsPull>::new(&path).unwrap();

        let runner = UnsuccessfulCommandRunner;

        let result = execution.pull(&runner);

        assert!(matches!(
            result,
            Err(ExecutionError::CommandFailed {
                status: Some(1),
                ..
            })
        ));

        fs::remove_dir_all(&path).unwrap();
    }
}
