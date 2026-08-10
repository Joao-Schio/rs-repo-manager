use std::process::ExitCode;

use rs_repo_manager::{
    arguments::Arguments,
    command::process_runner::ProcessCommandRunner,
    configuration::Configuration,
    repository::RepositoryManager,
};

fn main() -> ExitCode {
    let args = match Arguments::parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("Invalid arguments: {error:?}");
            return ExitCode::FAILURE;
        }
    };

    let configuration = match Configuration::load(args.configuration_path) {
        Ok(configuration) => configuration,
        Err(error) => {
            eprintln!("Failed to load configuration: {error:?}");
            return ExitCode::FAILURE;
        }
    };

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

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
