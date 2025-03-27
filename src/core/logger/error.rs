use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Errors that can occur during logging operations
#[derive(Debug, Error)]
pub enum LoggingError {
    /// The logging operation could not be completed
    #[error("Failed to log message: {0}")]
    LoggingFailed(String),

    /// Logger configuration error
    #[error("Invalid logger configuration: {0}")]
    ConfigurationError(String),

    /// Invalid log level provided
    #[error("Invalid log level: {0}")]
    InvalidLogLevel(String),

    /// Error with the logging provider
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// The logger failed to initialize
    #[error("Logger initialization failed: {0}")]
    InitializationError(String),

    /// The specified provider was not found
    #[error("Logging provider not found: {0}")]
    ProviderNotFound(String),

    /// I/O error occurred during logging
    #[error("I/O error during logging: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error occurred
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl LoggingError {
    /// Returns a descriptive error code for this error
    pub fn error_code(&self) -> &'static str {
        match self {
            LoggingError::LoggingFailed(_) => "LOGGING_FAILED",
            LoggingError::ConfigurationError(_) => "LOGGING_CONFIG_ERROR",
            LoggingError::InvalidLogLevel(_) => "INVALID_LOG_LEVEL",
            LoggingError::ProviderError(_) => "PROVIDER_ERROR",
            LoggingError::InitializationError(_) => "LOGGER_INIT_ERROR",
            LoggingError::ProviderNotFound(_) => "PROVIDER_NOT_FOUND",
            LoggingError::IoError(_) => "LOGGING_IO_ERROR",
            LoggingError::SerializationError(_) => "LOGGING_SERIALIZATION_ERROR",
        }
    }
}
