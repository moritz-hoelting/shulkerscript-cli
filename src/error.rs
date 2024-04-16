use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No file/directory found at path {0}.")]
    PathNotFoundError(PathBuf),
    #[error("An error occured because the directory {0} is not empty.")]
    NonEmptyDirectoryError(PathBuf),
    #[error("An error occured because the path {0} is not a directory.")]
    NotDirectoryError(PathBuf),
    #[error("An error occured because the path is neither a pack directory or a pack.toml file.")]
    InvalidPackPathError(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;
