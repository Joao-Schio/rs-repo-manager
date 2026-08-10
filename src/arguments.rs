use std::path::PathBuf;

#[derive(Debug)]
pub enum ArgumentError {
    NotEnoughArgs,
}
#[derive(Debug)]
pub struct Arguments {
    pub configuration_path: PathBuf,
}

impl Arguments {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, ArgumentError> {
        let path = args
            .into_iter()
            .next()
            .ok_or(ArgumentError::NotEnoughArgs)?;
        Ok(Self {
            configuration_path: PathBuf::from(path),
        })
    }
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::arguments::Arguments;
    
    #[test]
    fn parses_configuration_path_from_arguments() {
        let arguments = Arguments::parse([
            "/etc/rs-repo-manager/config.json".to_string(),
        ])
        .unwrap();
    
        assert_eq!(
            arguments.configuration_path,
            PathBuf::from("/etc/rs-repo-manager/config.json")
        );
    }
}

