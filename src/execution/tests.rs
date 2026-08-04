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
    fn run(&self, command: &CommandSpec, directory: &Path) -> Result<CommandOutput, CommandError> {
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

    assert!(commands.iter().any(|command| {
        command.program == "git" && command.args == vec!["pull"] && command.directory == path
    }));

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

#[test]
fn pull_reads_head_before_git_pull() {
    let path = std::env::temp_dir().join("rs_repo_manager_pull_reads_head_before_pull");

    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();

    let execution = Execution::<NeedsPull>::new(&path).unwrap();
    let runner = FakeCommandRunner::new();

    execution.pull(&runner).unwrap();

    let commands = runner.commands.borrow();

    assert!(commands.len() >= 2);

    assert_eq!(commands[0].program, "git");
    assert_eq!(commands[0].args, vec!["rev-parse", "HEAD"]);

    assert_eq!(commands[1].program, "git");
    assert_eq!(commands[1].args, vec!["pull"]);
}

#[test]
fn pull_reads_head_after_git_pull() {
    let path = std::env::temp_dir().join("rs_repo_manager_pull_reads_head_after_pull");

    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();

    let execution = Execution::<NeedsPull>::new(&path).unwrap();
    let runner = FakeCommandRunner::new();

    execution.pull(&runner).unwrap();

    let commands = runner.commands.borrow();

    assert_eq!(commands.len(), 3);

    assert_eq!(commands[0].args, vec!["rev-parse", "HEAD"]);

    assert_eq!(commands[1].args, vec!["pull"]);

    assert_eq!(commands[2].args, vec!["rev-parse", "HEAD"]);

    fs::remove_dir_all(&path).unwrap();
}

struct UpToDateCommandRunner {
    commands: RefCell<Vec<RecordedCommand>>,
    invocation: RefCell<usize>,
}

impl UpToDateCommandRunner {
    fn new() -> Self {
        Self {
            commands: RefCell::new(Vec::new()),
            invocation: RefCell::new(0),
        }
    }
}

impl CommandRunner for UpToDateCommandRunner {
    fn run(&self, command: &CommandSpec, directory: &Path) -> Result<CommandOutput, CommandError> {
        self.commands.borrow_mut().push(RecordedCommand {
            program: command.program.clone(),
            args: command.args.clone(),
            directory: directory.to_path_buf(),
        });

        let mut invocation = self.invocation.borrow_mut();

        let stdout = match *invocation {
            0 => "abc123\n",
            1 => "",
            2 => "abc123\n",
            _ => panic!("unexpected command"),
        };

        *invocation += 1;

        Ok(CommandOutput {
            status: Some(0),
            stdout: stdout.into(),
            stderr: String::new(),
        })
    }
}

#[test]
fn pull_reports_up_to_date_when_head_does_not_change() {
    let path = std::env::temp_dir().join("rs_repo_manager_pull_reports_up_to_date");

    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();

    let execution = Execution::<NeedsPull>::new(&path).unwrap();
    let runner = UpToDateCommandRunner::new();

    let result = execution.pull(&runner).unwrap();

    assert!(matches!(result, PullOutcome::UpToDate));

    fs::remove_dir_all(&path).unwrap();
}

struct UpdatedCommandRunner {
    invocation: RefCell<usize>,
}

impl UpdatedCommandRunner {
    fn new() -> Self {
        Self {
            invocation: RefCell::new(0),
        }
    }
}

impl CommandRunner for UpdatedCommandRunner {
    fn run(
        &self,
        _command: &CommandSpec,
        _directory: &Path,
    ) -> Result<CommandOutput, CommandError> {
        let mut invocation = self.invocation.borrow_mut();

        let stdout = match *invocation {
            0 => "abc123\n",
            1 => "",
            2 => "def456\n",
            _ => panic!("unexpected command"),
        };

        *invocation += 1;

        Ok(CommandOutput {
            status: Some(0),
            stdout: stdout.into(),
            stderr: String::new(),
        })
    }
}

fn assert_needs_deploy(_execution: &Execution<NeedsDeploy>) {}

#[test]
fn updated_pull_returns_execution_ready_for_deployment() {
    let path = std::env::temp_dir().join("rs_repo_manager_updated_ready_for_deployment");

    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();

    let execution = Execution::<NeedsPull>::new(&path).unwrap();
    let runner = UpdatedCommandRunner::new();

    let outcome = execution.pull(&runner).unwrap();

    match outcome {
        PullOutcome::Updated(execution) => {
            assert_needs_deploy(&execution);
        }
        PullOutcome::UpToDate => {
            panic!("expected repository to be updated");
        }
    }

    fs::remove_dir_all(&path).unwrap();
}

#[test]
fn deploy_runs_docker_compose_up_in_repository_directory() {
    let path = std::env::temp_dir().join("rs_repo_manager_deploy_runs_compose_up");

    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();

    let execution = Execution::<NeedsDeploy> {
        state: PhantomData,
        directory: path.clone(),
    };

    let runner = FakeCommandRunner::new();
    let fake_plan = DeploymentPlan { compose_down: false, before_down: vec![] };

    execution.deploy(&runner, &fake_plan).unwrap();

    let commands = runner.commands.borrow();

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].program, "docker");
    assert_eq!(commands[0].args, vec!["compose", "up", "-d", "--build"]);
    assert_eq!(commands[0].directory, path);

    fs::remove_dir_all(&path).unwrap();
}

#[test]
fn deploy_runs_compose_down_before_up_when_enabled() {
    let path = std::env::temp_dir().join("rs_repo_manager_deploy_runs_compose_down");

    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();

    let execution = Execution::<NeedsDeploy> {
        state: PhantomData,
        directory: path.clone(),
    };

    let runner = FakeCommandRunner::new();

    let plan = DeploymentPlan {
        compose_down: true,
        before_down: vec![],
    };

    execution.deploy(&runner, &plan).unwrap();

    let commands = runner.commands.borrow();

    assert_eq!(commands.len(), 2);

    assert_eq!(commands[0].program, "docker");
    assert_eq!(commands[0].args, vec!["compose", "down"]);

    assert_eq!(commands[1].program, "docker");
    assert_eq!(commands[1].args, vec!["compose", "up", "-d", "--build"]);

    assert!(commands.iter().all(|command| { command.directory == path }));

    fs::remove_dir_all(&path).unwrap();
}
