//! Error types for navius-core

use std::fmt;
use thiserror::Error;

/// A specialized Result type for navius-core operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for navius-core.
#[derive(Error, Debug)]
pub enum Error {
    /// An internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),

    /// A validation error occurred.
    #[error("Validation error: {0}")]
    Validation(String),

    /// A configuration error occurred.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A serialization error occurred.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl Error {
    /// Create a new internal error.
    pub fn internal<T: fmt::Display>(msg: T) -> Self {
        Self::Internal(msg.to_string())
    }

    /// Create a new validation error.
    pub fn validation<T: fmt::Display>(msg: T) -> Self {
        Self::Validation(msg.to_string())
    }

    /// Create a new configuration error.
    pub fn configuration<T: fmt::Display>(msg: T) -> Self {
        Self::Configuration(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_construction() {
        let err = Error::internal("Something went wrong");
        assert!(matches!(err, Error::Internal(_)));

        let err = Error::configuration("Missing config");
        assert!(matches!(err, Error::Configuration(_)));
    }
}
