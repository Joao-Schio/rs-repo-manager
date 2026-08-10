use crate::{
    execution::DeploymentPlan,
    repository::{Repository, RepositoryManager},
    test_support::{FakeCommandRunner, FakeRepositoryState, TestDirectory},
};

#[test]
fn manager_does_not_deploy_repository_when_pull_is_up_to_date() {
    let directory = TestDirectory::new("manager_up_to_date");

    let repository = Repository {
        directory: directory.path().to_path_buf(),
        deployment_plan: DeploymentPlan::default(),
    };

    let runner = FakeCommandRunner::new([FakeRepositoryState::up_to_date(directory.path())]);
    let manager = RepositoryManager::new(&runner);

    manager.run(&[repository]).unwrap();

    let commands = runner.commands.borrow();

    assert!(
        commands
            .iter()
            .any(|command| { command.program == "git" && command.args == vec!["pull"] })
    );

    assert!(
        !commands
            .iter()
            .any(|command| { command.program == "docker" })
    );
}

#[test]
fn manager_deploys_repository_when_pull_detects_update() {
    let directory = TestDirectory::new("manager_updated");

    let repository = Repository {
        directory: directory.path().to_path_buf(),
        deployment_plan: DeploymentPlan::default(),
    };

    let runner = FakeCommandRunner::new([FakeRepositoryState::updated(directory.path())]);
    let manager = RepositoryManager::new(&runner);

    manager.run(&[repository]).unwrap();

    let commands = runner.commands.borrow();

    assert!(commands.iter().any(|command| {
        command.program == "docker" && command.args == vec!["compose", "up", "-d", "--build"]
    }));
}

#[test]
fn manager_deploys_only_repositories_that_changed() {
    let unchanged_directory = TestDirectory::new("manager_unchanged");
    let updated_directory = TestDirectory::new("manager_updated");

    let repositories = [
        Repository {
            directory: unchanged_directory.path().to_path_buf(),
            deployment_plan: DeploymentPlan::default(),
        },
        Repository {
            directory: updated_directory.path().to_path_buf(),
            deployment_plan: DeploymentPlan::default(),
        },
    ];

    let runner = FakeCommandRunner::new([
        FakeRepositoryState::up_to_date(unchanged_directory.path()),
        FakeRepositoryState::updated(updated_directory.path()),
    ]);

    let manager = RepositoryManager::new(&runner);

    manager.run(&repositories).unwrap();

    let commands = runner.commands.borrow();

    assert!(!commands.iter().any(|command| {
        command.program == "docker" && command.directory == unchanged_directory.path()
    }));

    assert!(commands.iter().any(|command| {
        command.program == "docker"
            && command.args == vec!["compose", "up", "-d", "--build"]
            && command.directory == updated_directory.path()
    }));
}

#[test]
fn manager_continues_after_repository_failure() {
    let missing_directory = std::env::temp_dir().join("rs_repo_manager_missing_repository");

    let valid_directory = TestDirectory::new("manager_after_failure");

    let repositories = [
        Repository {
            directory: missing_directory,
            deployment_plan: DeploymentPlan::default(),
        },
        Repository {
            directory: valid_directory.path().to_path_buf(),
            deployment_plan: DeploymentPlan::default(),
        },
    ];

    let runner = FakeCommandRunner::new([FakeRepositoryState::up_to_date(valid_directory.path())]);

    let manager = RepositoryManager::new(&runner);

    let result = manager.run(&repositories);

    assert!(result.is_err());

    let commands = runner.commands.borrow();

    assert!(commands.iter().any(|command| {
        command.program == "git"
            && command.args == vec!["pull"]
            && command.directory == valid_directory.path()
    }));
}

#[test]
fn manager_collects_all_repository_failures() {
    let first_missing = std::env::temp_dir().join("rs_repo_manager_missing_first");

    let second_missing = std::env::temp_dir().join("rs_repo_manager_missing_second");

    let repositories = [
        Repository {
            directory: first_missing.clone(),
            deployment_plan: DeploymentPlan::default(),
        },
        Repository {
            directory: second_missing.clone(),
            deployment_plan: DeploymentPlan::default(),
        },
    ];

    let runner = FakeCommandRunner::empty();
    let manager = RepositoryManager::new(&runner);

    let failures = manager.run(&repositories).unwrap_err();

    assert_eq!(failures.len(), 2);

    assert!(
        failures
            .iter()
            .any(|failure| { failure.directory == first_missing })
    );

    assert!(
        failures
            .iter()
            .any(|failure| { failure.directory == second_missing })
    );
}
