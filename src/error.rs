//! Error types for the wfmash-rs library.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for wfmash operations.
pub type Result<T> = std::result::Result<T, WfmashError>;

/// Errors that can occur during wfmash operations.
#[derive(Error, Debug)]
pub enum WfmashError {
    /// Input file not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// I/O error during file operations
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// wfmash execution failed
    #[error("wfmash execution failed: {0}")]
    ExecutionFailed(String),

    /// wfmash binary not found
    #[error("wfmash binary not found. Install wfmash or ensure it's in PATH.")]
    BinaryNotFound,

    /// Invalid configuration parameter
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Temporary directory creation failed
    #[error("Failed to create temporary directory")]
    TempDirError,

    /// UTF-8 conversion error
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    /// Generic error with custom message
    #[error("{0}")]
    Other(String),
}

impl From<tempfile::PersistError> for WfmashError {
    fn from(_: tempfile::PersistError) -> Self {
        WfmashError::TempDirError
    }
}
