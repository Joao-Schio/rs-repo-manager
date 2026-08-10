use super::*;
use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::command::{CommandError, CommandOutput, CommandRunner, CommandSpec};

static TEST_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(name: &str) -> Self {
        let id = TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "rs_repo_manager_{name}_{}_{}",
            std::process::id(),
            id
        ));

        fs::create_dir_all(&path).unwrap();

        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

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
    let directory = TestDirectory::new("new_accepts_directory");

    let result = Execution::<NeedsPull>::new(directory.path());

    assert!(result.is_ok());
}

#[test]
fn new_rejects_nonexistent_directory() {
    let directory = TestDirectory::new("new_rejects_nonexistent_directory");
    let path = directory.path().join("missing");

    let result = Execution::<NeedsPull>::new(&path);

    assert!(matches!(result, Err(ExecutionError::InvalidDirectory(_))));
}

#[test]
fn pull_runs_git_pull_in_repository_directory() {
    let directory = TestDirectory::new("pull_runs_git_pull");
    let path = directory.path().to_path_buf();

    let execution = Execution::<NeedsPull>::new(&path).unwrap();
    let runner = FakeCommandRunner::new();

    execution.pull(&runner).unwrap();

    let commands = runner.commands.borrow();

    assert!(commands.iter().any(|command| {
        command.program == "git" && command.args == vec!["pull"] && command.directory == path
    }));
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
    let directory = TestDirectory::new("pull_propagates_runner_error");

    let execution = Execution::<NeedsPull>::new(directory.path()).unwrap();
    let runner = FailingCommandRunner;

    let result = execution.pull(&runner);

    assert!(matches!(
        result,
        Err(ExecutionError::CommandError(CommandError::IoError(_)))
    ));
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
    let directory = TestDirectory::new("pull_rejects_unsuccessful_output");

    let execution = Execution::<NeedsPull>::new(directory.path()).unwrap();
    let runner = UnsuccessfulCommandRunner;

    let result = execution.pull(&runner);

    assert!(matches!(
        result,
        Err(ExecutionError::CommandFailed {
            status: Some(1),
            ..
        })
    ));
}

#[test]
fn pull_reads_head_before_git_pull() {
    let directory = TestDirectory::new("pull_reads_head_before_pull");

    let execution = Execution::<NeedsPull>::new(directory.path()).unwrap();
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
    let directory = TestDirectory::new("pull_reads_head_after_pull");

    let execution = Execution::<NeedsPull>::new(directory.path()).unwrap();
    let runner = FakeCommandRunner::new();

    execution.pull(&runner).unwrap();

    let commands = runner.commands.borrow();

    assert_eq!(commands.len(), 3);

    assert_eq!(commands[0].args, vec!["rev-parse", "HEAD"]);
    assert_eq!(commands[1].args, vec!["pull"]);
    assert_eq!(commands[2].args, vec!["rev-parse", "HEAD"]);
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
    let directory = TestDirectory::new("pull_reports_up_to_date");

    let execution = Execution::<NeedsPull>::new(directory.path()).unwrap();
    let runner = UpToDateCommandRunner::new();

    let result = execution.pull(&runner).unwrap();

    assert!(matches!(result, PullOutcome::UpToDate));
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
    let directory = TestDirectory::new("updated_ready_for_deployment");

    let execution = Execution::<NeedsPull>::new(directory.path()).unwrap();
    let runner = UpdatedCommandRunner::new();

    let outcome = execution.pull(&runner).unwrap();

    match outcome {
        PullOutcome::Updated(execution) => assert_needs_deploy(&execution),
        PullOutcome::UpToDate => panic!("expected repository to be updated"),
    }
}

#[test]
fn deploy_runs_docker_compose_up_in_repository_directory() {
    let directory = TestDirectory::new("deploy_runs_compose_up");
    let path = directory.path().to_path_buf();

    let execution = Execution::<NeedsDeploy> {
        state: PhantomData,
        directory: path.clone(),
    };

    let runner = FakeCommandRunner::new();
    let fake_plan = DeploymentPlan {
        compose_down: false,
        after_pull: vec![],
        before_up: vec![],
        after_up: vec![],
    };

    execution.deploy(&runner, &fake_plan).unwrap();

    let commands = runner.commands.borrow();

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].program, "docker");
    assert_eq!(commands[0].args, vec!["compose", "up", "-d", "--build"]);
    assert_eq!(commands[0].directory, path);
}

#[test]
fn deploy_runs_compose_down_before_up_when_enabled() {
    let directory = TestDirectory::new("deploy_runs_compose_down");
    let path = directory.path().to_path_buf();

    let execution = Execution::<NeedsDeploy> {
        state: PhantomData,
        directory: path.clone(),
    };

    let runner = FakeCommandRunner::new();
    let plan = DeploymentPlan {
        compose_down: true,
        after_pull: vec![],
        before_up: Vec::new(),
        after_up: vec![],
    };

    execution.deploy(&runner, &plan).unwrap();

    let commands = runner.commands.borrow();

    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].program, "docker");
    assert_eq!(commands[0].args, vec!["compose", "down"]);
    assert_eq!(commands[1].program, "docker");
    assert_eq!(commands[1].args, vec!["compose", "up", "-d", "--build"]);
    assert!(commands.iter().all(|command| command.directory == path));
}

#[test]
fn deploy_runs_after_pull_commands_before_compose_down() {
    let directory = TestDirectory::new("after_pull_before_down");
    let path = directory.path().to_path_buf();

    let execution = Execution::<NeedsDeploy> {
        state: PhantomData,
        directory: path.clone(),
    };

    let runner = FakeCommandRunner::new();
    let plan = DeploymentPlan {
        after_pull: vec![CommandSpec {
            program: "echo".into(),
            args: vec!["hello".into()],
        }],
        compose_down: true,
        before_up: vec![],
        after_up: vec![],
    };

    execution.deploy(&runner, &plan).unwrap();

    let commands = runner.commands.borrow();

    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].program, "echo");
    assert_eq!(commands[0].args, vec!["hello"]);
    assert_eq!(commands[1].program, "docker");
    assert_eq!(commands[1].args, vec!["compose", "down"]);
    assert_eq!(commands[2].program, "docker");
    assert_eq!(commands[2].args, vec!["compose", "up", "-d", "--build"]);
    assert!(commands.iter().all(|command| command.directory == path));
}

#[test]
fn deploy_runs_before_up_commands_after_compose_down() {
    let directory = TestDirectory::new("before_up_after_down");
    let path = directory.path().to_path_buf();

    let execution = Execution::<NeedsDeploy> {
        state: PhantomData,
        directory: path.clone(),
    };

    let runner = FakeCommandRunner::new();
    let plan = DeploymentPlan {
        after_pull: vec![],
        compose_down: true,
        before_up: vec![CommandSpec {
            program: "echo".into(),
            args: vec!["before-up".into()],
        }],
        after_up: vec![],
    };

    execution.deploy(&runner, &plan).unwrap();

    let commands = runner.commands.borrow();

    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].args, vec!["compose", "down"]);
    assert_eq!(commands[1].program, "echo");
    assert_eq!(commands[1].args, vec!["before-up"]);
    assert_eq!(commands[2].args, vec!["compose", "up", "-d", "--build"]);
    assert!(commands.iter().all(|command| command.directory == path));
}

#[test]
fn deploy_runs_after_up_commands_after_compose_up() {
    let directory = TestDirectory::new("after_up");
    let path = directory.path().to_path_buf();

    let execution = Execution::<NeedsDeploy> {
        state: PhantomData,
        directory: path.clone(),
    };

    let runner = FakeCommandRunner::new();
    let plan = DeploymentPlan {
        after_pull: vec![],
        compose_down: false,
        before_up: vec![],
        after_up: vec![CommandSpec {
            program: "echo".into(),
            args: vec!["after-up".into()],
        }],
    };

    execution.deploy(&runner, &plan).unwrap();

    let commands = runner.commands.borrow();

    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].program, "docker");
    assert_eq!(commands[0].args, vec!["compose", "up", "-d", "--build"]);
    assert_eq!(commands[1].program, "echo");
    assert_eq!(commands[1].args, vec!["after-up"]);
    assert!(commands.iter().all(|command| command.directory == path));
}
