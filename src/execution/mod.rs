use std::{
    fs::exists, marker::PhantomData, path::{Path, PathBuf},
};

use crate::execution::execution_error::ExecutionError;

pub mod execution_error;

pub struct NeedsPull;

pub struct Execution<State> {
    state: PhantomData<State>,
    directory: PathBuf,
}

impl Execution<NeedsPull> {
    pub fn new(
        path: impl AsRef<Path>,
    ) -> Result<Self, ExecutionError> {
        let struct_path = path.as_ref().to_path_buf();
        
        if !struct_path.is_dir() {
            return Err(ExecutionError::InvalidDirectory(struct_path));
        }

        Ok(
            Self { state: PhantomData, 
                directory: struct_path 
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn new_accepts_existing_directory() {
        let path =
            std::env::temp_dir()
                .join("rs_repo_manager_new_accepts_directory");

        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        let result = Execution::<NeedsPull>::new(&path);

        assert!(result.is_ok());

        fs::remove_dir_all(&path).unwrap();
    }

    #[test]
    fn new_rejects_nonexistent_directory() {
        let path =
            std::env::temp_dir()
                .join("rs_repo_manager_directory_that_does_not_exist");

        let _ = fs::remove_dir_all(&path);

        let result = Execution::<NeedsPull>::new(&path);

        assert!(matches!(
            result,
            Err(ExecutionError::InvalidDirectory(_))
        ));
    }
}