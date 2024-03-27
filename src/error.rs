use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An error occurred while parsing command-line arguments.")]
    IoError(#[from] std::io::Error),
    #[error("An error occured while serializing to TOML.")]
    TomlSerializeError(#[from] toml::ser::Error),
    #[error("An error occured while deserializing from TOML.")]
    TomlDeserializeError(#[from] toml::de::Error),
    #[error("No file/directory found at path {0}.")]
    PathNotFoundError(PathBuf),
    #[error("An error occured because the directory {0} is not empty.")]
    NonEmptyDirectoryError(PathBuf),
    #[error("An error occured because the path {0} is not a directory.")]
    NotDirectoryError(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;
