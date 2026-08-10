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