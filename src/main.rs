use rs_repo_manager::{
    arguments::Arguments, command::process_runner::ProcessCommandRunner, configuration::Configuration, repository::RepositoryManager,
};

fn main() {
    let args = Arguments::parse(
        std::env::args().skip(1)
    );

    if args.is_err() {
        eprintln!("Arguments not loaded correctly");
        return;
    }


    let configuration = Configuration::load(args.unwrap().configuration_path).expect("failed to load configuration");

    let repositories = configuration.into_repositories();

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
