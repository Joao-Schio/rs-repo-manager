use std::{
    cell::RefCell,
    collections::HashMap,
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
pub(crate) struct RecordedCommand {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) directory: PathBuf,
}

pub(crate) struct FakeRepositoryState {
    directory: PathBuf,
    head_before: String,
    head_after: String,
    pulled: bool,
}

impl FakeRepositoryState {
    pub(crate) fn up_to_date(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            head_before: "abc123".into(),
            head_after: "abc123".into(),
            pulled: false,
        }
    }

    pub(crate) fn updated(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            head_before: "abc123".into(),
            head_after: "def456".into(),
            pulled: false,
        }
    }
}

pub(crate) struct FakeCommandRunner {
    pub(crate) commands: RefCell<Vec<RecordedCommand>>,
    repositories: RefCell<HashMap<PathBuf, FakeRepositoryState>>,
}

impl FakeCommandRunner {
    pub(crate) fn new(
        repositories: impl IntoIterator<Item = FakeRepositoryState>,
    ) -> Self {
        let repositories = repositories
            .into_iter()
            .map(|state| (state.directory.clone(), state))
            .collect();

        Self {
            commands: RefCell::new(Vec::new()),
            repositories: RefCell::new(repositories),
        }
    }

    pub(crate) fn empty() -> Self {
        Self::new(std::iter::empty())
    }
}

impl CommandRunner for FakeCommandRunner {
    fn run(&self, command: &CommandSpec, directory: &Path) -> Result<CommandOutput, CommandError> {
        self.commands.borrow_mut().push(RecordedCommand {
            program: command.program.clone(),
            args: command.args.clone(),
            directory: directory.to_path_buf(),
        });

        if command.program == "git" && command.args == vec!["rev-parse", "HEAD"] {
            let repositories = self.repositories.borrow();
            let repository = repositories
                .get(directory)
                .expect("missing fake repository state");

            let stdout = if repository.pulled {
                &repository.head_after
            } else {
                &repository.head_before
            };

            return Ok(CommandOutput {
                status: Some(0),
                stdout: format!("{stdout}\n"),
                stderr: String::new(),
            });
        }

        if command.program == "git" && command.args == vec!["pull"] {
            let mut repositories = self.repositories.borrow_mut();
            let repository = repositories
                .get_mut(directory)
                .expect("missing fake repository state");

            repository.pulled = true;

            return Ok(CommandOutput {
                status: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            });
        }

        Ok(CommandOutput {
            status: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}
