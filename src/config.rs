use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct BenchConfig {
    pub dir_path: PathBuf,
}

impl BenchConfig {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let path_ref = path.as_ref();

        if !path_ref.exists() {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                format!("The path '{:?}' does not exist.", path_ref),
            ));
        }

        if !path_ref.is_dir() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("The path '{:?}' is not a directory.", path_ref),
            ));
        }

        Ok(BenchConfig {
            dir_path: path_ref.to_path_buf(),
        })
    }
}
