use std::ffi::NulError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to get destinations from CUPS server")]
    DestinationListFailed,

    #[error("Destination '{0}' not found")]
    DestinationNotFound(String),

    #[error("Failed to convert C string: {0}")]
    StringConversionError(#[from] std::str::Utf8Error),

    #[error("Null pointer encountered")]
    NullPointer,

    #[error("CUPS server error: {0}")]
    ServerError(String),

    #[error("Invalid name containing null bytes: {0}")]
    InvalidName(String),

    #[error("Enumeration error: {0}")]
    EnumerationError(String),

    #[error("Detailed destination information unavailable")]
    DetailedInfoUnavailable,

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Media size error: {0}")]
    MediaSizeError(String),

    #[error("Job creation failed: {0}")]
    JobCreationFailed(String),

    #[error("Document submission failed: {0}")]
    DocumentSubmissionFailed(String),

    #[error("Job management failed: {0}")]
    JobManagementFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("CUPS server unavailable")]
    ServerUnavailable,

    #[error("Authentication required for destination '{0}'")]
    AuthenticationRequired(String),

    #[error("Permission denied for operation on '{0}'")]
    PermissionDenied(String),

    #[error("Printer '{0}' is offline")]
    PrinterOffline(String),

    #[error("Printer '{0}' is not accepting jobs: {1}")]
    PrinterNotAccepting(String, String),

    #[error("Invalid document format '{0}' for destination '{1}'")]
    InvalidFormat(String, String),

    #[error("Document too large: {0} bytes (max: {1} bytes)")]
    DocumentTooLarge(usize, usize),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Timeout waiting for operation to complete")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<NulError> for Error {
    fn from(error: NulError) -> Self {
        Error::InvalidName(format!("String contains null bytes: {}", error))
    }
}

impl Error {
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::ServerUnavailable
            | Error::NetworkError(_)
            | Error::Timeout
            | Error::PrinterOffline(_) => true,

            Error::AuthenticationRequired(_)
            | Error::PermissionDenied(_)
            | Error::PrinterNotAccepting(_, _) => false,

            Error::DocumentTooLarge(_, _)
            | Error::InvalidFormat(_, _)
            | Error::ConfigurationError(_) => false,

            _ => false,
        }
    }

    pub fn error_category(&self) -> ErrorCategory {
        match self {
            Error::ServerUnavailable | Error::NetworkError(_) | Error::Timeout => {
                ErrorCategory::Network
            }
            Error::AuthenticationRequired(_) | Error::PermissionDenied(_) => {
                ErrorCategory::Authentication
            }
            Error::PrinterOffline(_) | Error::PrinterNotAccepting(_, _) => ErrorCategory::Printer,
            Error::InvalidFormat(_, _) | Error::DocumentTooLarge(_, _) => ErrorCategory::Document,
            Error::JobCreationFailed(_) | Error::JobManagementFailed(_) => ErrorCategory::Job,
            Error::ConfigurationError(_) => ErrorCategory::Configuration,
            _ => ErrorCategory::General,
        }
    }

    pub fn suggested_action(&self) -> &'static str {
        match self {
            Error::ServerUnavailable => {
                "Check if CUPS service is running: sudo systemctl status cups"
            }
            Error::AuthenticationRequired(_) => "Provide valid credentials for the printer",
            Error::PrinterOffline(_) => "Check printer connection and power status",
            Error::PrinterNotAccepting(_, _) => "Enable job acceptance: cupsaccept <printer>",
            Error::InvalidFormat(_, _) => {
                "Convert document to a supported format (PDF, PostScript, text)"
            }
            Error::DocumentTooLarge(_, _) => "Reduce document size or split into smaller files",
            Error::NetworkError(_) => "Check network connectivity to CUPS server",
            Error::Timeout => "Retry the operation or increase timeout value",
            Error::ConfigurationError(_) => "Check CUPS configuration files",
            _ => "Check CUPS logs for more details: sudo tail /var/log/cups/error_log",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Network,
    Authentication,
    Printer,
    Document,
    Job,
    Configuration,
    General,
}
