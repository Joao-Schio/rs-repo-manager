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
