use std::path::PathBuf;

use crate::configuration::Configuration;


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

    let configuration: Configuration =
        serde_json::from_str(input).unwrap();

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

    let configuration: Configuration =
        serde_json::from_str(input).unwrap();

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

    let configuration: Configuration =
        serde_json::from_str(input).unwrap();

    let command =
        &configuration.repositories[0].after_pull[0];

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

    let configuration: Configuration =
        serde_json::from_str(input).unwrap();

    let repositories = configuration.into_repositories();

    assert_eq!(repositories.len(), 1);

    let repository = &repositories[0];

    assert_eq!(
        repository.directory,
        PathBuf::from("/srv/my-service")
    );

    assert!(repository.deployment_plan.compose_down);

    assert_eq!(
        repository.deployment_plan.after_pull[0].program,
        "cargo"
    );

    assert_eq!(
        repository.deployment_plan.after_pull[0].args,
        vec!["test"]
    );
}