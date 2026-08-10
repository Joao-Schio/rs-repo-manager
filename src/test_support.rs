use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::command::{CommandError, CommandOutput, CommandRunner, CommandSpec};

static TEST_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    pub(crate) fn new(name: &str) -> Self {
        let id = TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);

        let path = std::env::temp_dir().join(format!(
            "rs_repo_manager_{name}_{}_{}",
            std::process::id(),
            id
        ));

        fs::create_dir_all(&path).unwrap();

        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
#[derive(Debug)]
pub struct RecordedCommand {
    pub program: String,
    pub args: Vec<String>,
    pub directory: PathBuf,
}

pub struct UpToDateCommandRunner {
    pub commands: RefCell<Vec<RecordedCommand>>,
    pub invocation: RefCell<usize>,
}

impl UpToDateCommandRunner {
    pub fn new() -> Self {
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

pub struct UpdatedCommandRunner {
    pub invocation: RefCell<usize>,
}

impl UpdatedCommandRunner {
    pub fn new() -> Self {
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
