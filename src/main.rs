use rs_repo_manager::{
    command::process_runner::ProcessCommandRunner,
    configuration::Configuration,
    repository::RepositoryManager,
};

fn main() {
    let configuration =
        Configuration::load("config.json")
            .expect("failed to load configuration");

    let repositories =
        configuration.into_repositories();

    let runner = ProcessCommandRunner;
    let manager = RepositoryManager::new(&runner);

    if let Err(failures) = manager.run(&repositories) {
        for failure in failures {
            eprintln!(
                "Repository {} failed: {:?}",
                failure.directory.display(),
                failure.error
            );
        }
    }
}