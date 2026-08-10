use std::path::PathBuf;

use crate::{
    configuration::{Configuration, ConfigurationError},
    test_support::TestDirectory,
};

#[test]
fn parses_repository_directory() {
    let input = r#"
    {
        "repositories": [
            {
                "directory": "/srv/my-service"
            }
        ]
    }
    "#;

    let configuration = Configuration::parse(input).unwrap();

    assert_eq!(configuration.repositories.len(), 1);

    assert_eq!(
        configuration.repositories[0].directory,
        PathBuf::from("/srv/my-service")
    );
}

#[test]
fn repository_configuration_defaults_deployment_options() {
    let input = r#"
    {
        "repositories": [
            {
                "directory": "/srv/my-service"
            }
        ]
    }
    "#;

    let configuration = Configuration::parse(input).unwrap();

    let repository = &configuration.repositories[0];

    assert!(!repository.compose_down);
    assert!(repository.after_pull.is_empty());
    assert!(repository.before_up.is_empty());
    assert!(repository.after_up.is_empty());
}

#[test]
fn parses_configured_command() {
    let input = r#"
    {
        "repositories": [
            {
                "directory": "/srv/my-service",
                "after_pull": [
                    {
                        "program": "cargo",
                        "args": ["test"]
                    }
                ]
            }
        ]
    }
    "#;

    let configuration = Configuration::parse(input).unwrap();

    let command = &configuration.repositories[0].after_pull[0];

    assert_eq!(command.program, "cargo");
    assert_eq!(command.args, vec!["test"]);
}

#[test]
fn converts_repository_configuration_to_runtime_repository() {
    let input = r#"
    {
        "repositories": [
            {
                "directory": "/srv/my-service",
                "compose_down": true,
                "after_pull": [
                    {
                        "program": "cargo",
                        "args": ["test"]
                    }
                ]
            }
        ]
    }
    "#;

    let configuration = Configuration::parse(input).unwrap();

    let repositories = configuration.into_repositories();

    assert_eq!(repositories.len(), 1);

    let repository = &repositories[0];

    assert_eq!(repository.directory, PathBuf::from("/srv/my-service"));

    assert!(repository.deployment_plan.compose_down);

    assert_eq!(repository.deployment_plan.after_pull[0].program, "cargo");

    assert_eq!(repository.deployment_plan.after_pull[0].args, vec!["test"]);
}

#[test]
fn loads_configuration_from_file() {
    let directory = TestDirectory::new("loads_configuration");

    let config_path = directory.path().join("config.json");

    std::fs::write(
        &config_path,
        r#"
        {
            "repositories": [
                {
                    "directory": "/srv/my-service"
                }
            ]
        }
        "#,
    )
    .unwrap();

    let configuration = Configuration::load(&config_path).unwrap();

    assert_eq!(configuration.repositories.len(), 1);
    assert_eq!(
        configuration.repositories[0].directory,
        PathBuf::from("/srv/my-service")
    );
}

#[test]
fn load_returns_io_error_when_file_does_not_exist() {
    let directory = TestDirectory::new("missing_configuration");

    let result = Configuration::load(directory.path().join("missing.json"));

    assert!(matches!(result, Err(ConfigurationError::Io(_))));
}

#[test]
fn load_returns_invalid_json_error() {
    let directory = TestDirectory::new("invalid_configuration");

    let config_path = directory.path().join("config.json");

    std::fs::write(&config_path, "{ definitely not json }").unwrap();

    let result = Configuration::load(&config_path);

    assert!(matches!(result, Err(ConfigurationError::InvalidJson(_))));
}
