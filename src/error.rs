//! Error types for CUPS operations

use std::ffi::NulError;
use thiserror::Error;

/// Errors that can occur when interacting with CUPS
#[derive(Error, Debug)]
pub enum Error {
    /// Failed to get destinations from CUPS server
    #[error("Failed to get destinations from CUPS server")]
    DestinationListFailed,

    /// Destination not found
    #[error("Destination '{0}' not found")]
    DestinationNotFound(String),

    /// String conversion error
    #[error("Failed to convert C string: {0}")]
    StringConversionError(#[from] std::str::Utf8Error),

    /// Null pointer encountered
    #[error("Null pointer encountered")]
    NullPointer,

    /// CUPS server error
    #[error("CUPS server error: {0}")]
    ServerError(String),

    /// Invalid name containing null bytes
    #[error("Invalid name containing null bytes: {0}")]
    InvalidName(String),

    /// Enumeration error
    #[error("Enumeration error: {0}")]
    EnumerationError(String),

    /// Detailed information unavailable
    #[error("Detailed destination information unavailable")]
    DetailedInfoUnavailable,

    /// Unsupported feature
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    /// Media size error
    #[error("Media size error: {0}")]
    MediaSizeError(String),

    /// Job creation failed
    #[error("Job creation failed: {0}")]
    JobCreationFailed(String),

    // Document submission failed
    #[error("Document submission failed: {0}")]
    DocumentSubmissionFailed(String),

    // IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for CUPS operations
pub type Result<T> = std::result::Result<T, Error>;

impl From<NulError> for Error {
    fn from(error: NulError) -> Self {
        Error::InvalidName(format!("String contains null bytes: {}", error))
    }
}
