//! Error types for navius-http.

use std::fmt;
use thiserror::Error;

/// A specialized Result type for navius-http operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for navius-http.
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

    /// An HTTP error occurred.
    #[error("HTTP error: {status} - {message}")]
    Http {
        /// The HTTP status code.
        status: u16,
        /// The error message.
        message: String,
    },

    /// A request error occurred.
    #[error("Request error: {0}")]
    Request(String),

    /// A response error occurred.
    #[error("Response error: {0}")]
    Response(String),

    /// A client error occurred.
    #[error("Client error: {0}")]
    Client(#[from] reqwest::Error),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A serialization error occurred.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// A core error occurred.
    #[error("Core error: {0}")]
    Core(#[from] navius_core::Error),
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

    /// Create a new HTTP error.
    pub fn http<T: fmt::Display>(status: u16, msg: T) -> Self {
        Self::Http {
            status,
            message: msg.to_string(),
        }
    }

    /// Create a new request error.
    pub fn request<T: fmt::Display>(msg: T) -> Self {
        Self::Request(msg.to_string())
    }

    /// Create a new response error.
    pub fn response<T: fmt::Display>(msg: T) -> Self {
        Self::Response(msg.to_string())
    }

    /// Create a new client error from a reqwest::Error.
    pub fn client(err: reqwest::Error) -> Self {
        Self::Client(err)
    }

    /// Get the HTTP status code for this error, if applicable.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Http { status, .. } => Some(*status),
            Self::Validation(_) => Some(400),
            Self::Request(_) => Some(400),
            Self::Configuration(_) => Some(500),
            Self::Internal(_) => Some(500),
            Self::Response(_) => Some(500),
            Self::Client(_) => Some(500),
            Self::Io(_) => Some(500),
            Self::Serialization(_) => Some(500),
            Self::Core(_) => Some(500),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_construction() {
        let err = Error::internal("Something went wrong");
        assert!(matches!(err, Error::Internal(_)));

        let err = Error::http(404, "Not found");
        assert!(matches!(err, Error::Http { status: 404, .. }));
    }

    #[test]
    fn test_status_code() {
        let err = Error::validation("Invalid input");
        assert_eq!(err.status_code(), Some(400));

        let err = Error::http(418, "I'm a teapot");
        assert_eq!(err.status_code(), Some(418));
    }
}
