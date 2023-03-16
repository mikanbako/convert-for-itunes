use std::{path::PathBuf, process::ExitStatus};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Path ({path}) is invalid.")]
    PathInvalid { path: PathBuf },

    #[error("{path} is not a directory.")]
    NotDirectory { path: PathBuf },

    #[error("{path} is not a file.")]
    NotFile { path: PathBuf },

    #[error("Not supported.")]
    NotSupported,

    #[error("Command {command} is not found: {error}")]
    CommandNotFound {
        command: String,
        error: which::Error,
    },

    #[error("Command {command} is failed: {status}")]
    CommandFailed { command: String, status: ExitStatus },

    #[error("Command {command} cannot be executed: {error}")]
    CommandCannotExecuted {
        command: String,
        error: std::io::Error,
    },

    #[error("I/O error: {error}")]
    IoError { error: std::io::Error },

    #[error("Metadata could not be read: {cause}")]
    CannotReadMetadata { cause: String },

    #[error("Metadata could not be write: {cause}")]
    CannotWriteMetadata { cause: String },

    #[error("Unknown error is occured.")]
    Unknown,
}
