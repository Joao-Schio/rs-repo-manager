#[test]
fn pull_runs_git_pull_in_repository_directory() {
    let path =
        std::env::temp_dir()
            .join("rs_repo_manager_pull_runs_git_pull");

    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();

    let execution =
        Execution::<NeedsPull>::new(&path).unwrap();

    let runner = FakeCommandRunner::new();

    execution.pull(&runner).unwrap();

    let commands = runner.commands.borrow();

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].program, "git");
    assert_eq!(commands[0].args, vec!["pull"]);
    assert_eq!(commands[0].directory, path);

    fs::remove_dir_all(&path).unwrap();
}