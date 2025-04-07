//! Error types for navius-http.

use navius_core::error::ErrorCode as CoreErrorCode;
use std::error::Error as StdError;
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

impl From<Error> for navius_core::Error {
    fn from(err: Error) -> Self {
        let core_code = match err.status_code() {
            Some(400) => CoreErrorCode::Validation,
            Some(401) => CoreErrorCode::Authentication,
            Some(403) => CoreErrorCode::Authorization,
            Some(404) => CoreErrorCode::NotFound,
            Some(409) => CoreErrorCode::Conflict,
            Some(408) => CoreErrorCode::Timeout,
            Some(500) => CoreErrorCode::Internal,
            Some(503) => CoreErrorCode::External,
            Some(504) => CoreErrorCode::External,
            Some(507) => CoreErrorCode::Conflict,
            Some(508) => CoreErrorCode::Authorization,
            Some(509) => CoreErrorCode::Authentication,
            Some(510) => CoreErrorCode::Authorization,
            Some(511) => CoreErrorCode::Authentication,
            _ => CoreErrorCode::Unknown,
        };

        let mut core_error = navius_core::Error::new(core_code, err.to_string());

        match err {
            Error::Core(core_err) => {
                if let Some(source) = core_err.source {
                    core_error.source = Some(source);
                }
            }
            _ => {
                core_error.source = Some(Box::new(err));
            }
        }

        core_error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use navius_test::error::{TestResult, assert_eq, assert_true};

    #[test]
    fn test_error_construction() -> TestResult<()> {
        let err = Error::internal("Something went wrong");
        assert_true(
            matches!(err, Error::Internal(_)),
            "Error should be an Internal error",
        )?;

        let err = Error::http(404, "Not found");
        assert_true(
            matches!(err, Error::Http { status: 404, .. }),
            "Error should be an HTTP error with status 404",
        )?;

        Ok(())
    }

    #[test]
    fn test_status_code() -> TestResult<()> {
        let err = Error::validation("Invalid input");
        assert_eq(
            err.status_code(),
            Some(400),
            "Validation error should have status code 400",
        )?;

        let err = Error::http(418, "I'm a teapot");
        assert_eq(
            err.status_code(),
            Some(418),
            "HTTP error should preserve the provided status code",
        )?;

        Ok(())
    }
}
