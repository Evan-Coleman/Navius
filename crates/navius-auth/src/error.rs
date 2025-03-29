//! Error handling for the Navius Auth crate.
//!
//! This module defines the error types and error handling functionality
//! for authentication and authorization operations.

use std::fmt;
use thiserror::Error;

/// A specialized Result type for authentication operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for authentication and authorization operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Authentication failed due to invalid credentials
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Token validation error
    #[error("Token validation failed: {0}")]
    TokenInvalid(String),

    /// Token expired
    #[error("Token expired")]
    TokenExpired,

    /// Missing or malformed token
    #[error("Missing or malformed token: {0}")]
    TokenMissing(String),

    /// Authorization failed - insufficient permissions
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Configuration error
    #[error("Authentication configuration error: {0}")]
    Configuration(String),

    /// Provider error
    #[error("Authentication provider error: {0}")]
    Provider(String),

    /// OAuth2 specific error
    #[cfg(feature = "oauth")]
    #[error("OAuth error: {0}")]
    OAuth(String),

    /// External service error (e.g., identity provider)
    #[error("External service error: {0}")]
    ExternalService(String),

    /// Data serialization or deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Internal error (unexpected or unhandled cases)
    #[error("Internal authentication error: {0}")]
    Internal(String),

    /// Core error (wrapped from navius-core)
    #[error("Core error: {0}")]
    Core(String),
}

impl Error {
    /// Create a new authentication failed error.
    pub fn authentication_failed<T: fmt::Display>(message: T) -> Self {
        Self::AuthenticationFailed(message.to_string())
    }

    /// Create a new token invalid error.
    pub fn token_invalid<T: fmt::Display>(message: T) -> Self {
        Self::TokenInvalid(message.to_string())
    }

    /// Create a new token expired error.
    pub fn token_expired() -> Self {
        Self::TokenExpired
    }

    /// Create a new token missing error.
    pub fn token_missing<T: fmt::Display>(message: T) -> Self {
        Self::TokenMissing(message.to_string())
    }

    /// Create a new authorization failed error.
    pub fn authorization_failed<T: fmt::Display>(message: T) -> Self {
        Self::AuthorizationFailed(message.to_string())
    }

    /// Create a new configuration error.
    pub fn configuration<T: fmt::Display>(message: T) -> Self {
        Self::Configuration(message.to_string())
    }

    /// Create a new provider error.
    pub fn provider<T: fmt::Display>(message: T) -> Self {
        Self::Provider(message.to_string())
    }

    /// Create a new OAuth error.
    #[cfg(feature = "oauth")]
    pub fn oauth<T: fmt::Display>(message: T) -> Self {
        Self::OAuth(message.to_string())
    }

    /// Create a new external service error.
    pub fn external_service<T: fmt::Display>(message: T) -> Self {
        Self::ExternalService(message.to_string())
    }

    /// Create a new serialization error.
    pub fn serialization<T: fmt::Display>(message: T) -> Self {
        Self::Serialization(message.to_string())
    }

    /// Create a new internal error.
    pub fn internal<T: fmt::Display>(message: T) -> Self {
        Self::Internal(message.to_string())
    }
}

impl From<navius_core::Error> for Error {
    fn from(err: navius_core::Error) -> Self {
        Error::Core(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

#[cfg(feature = "jwt")]
impl From<jsonwebtoken::errors::Error> for Error {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;
        match err.kind() {
            ErrorKind::ExpiredSignature => Error::TokenExpired,
            ErrorKind::InvalidToken => Error::TokenInvalid("Invalid token format".to_string()),
            ErrorKind::InvalidSignature => Error::TokenInvalid("Invalid signature".to_string()),
            _ => Error::TokenInvalid(err.to_string()),
        }
    }
}

#[cfg(feature = "oauth")]
impl From<oauth2::basic::BasicRequestTokenError<oauth2::reqwest::Error<reqwest::Error>>> for Error {
    fn from(
        err: oauth2::basic::BasicRequestTokenError<oauth2::reqwest::Error<reqwest::Error>>,
    ) -> Self {
        Error::OAuth(err.to_string())
    }
}

#[cfg(feature = "oauth")]
impl From<openid::error::Error> for Error {
    fn from(err: openid::error::Error) -> Self {
        Error::OAuth(err.to_string())
    }
}

impl From<Error> for navius_http::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::AuthenticationFailed(msg) => {
                Self::http(401, format!("Authentication failed: {}", msg))
            }
            Error::TokenInvalid(msg) => Self::http(401, format!("Invalid token: {}", msg)),
            Error::TokenExpired => Self::http(401, "Token expired"),
            Error::TokenMissing(msg) => Self::http(401, format!("Missing token: {}", msg)),
            Error::AuthorizationFailed(msg) => {
                Self::http(403, format!("Authorization failed: {}", msg))
            }
            Error::Configuration(msg) => Self::internal(format!("Configuration error: {}", msg)),
            Error::Provider(msg) => Self::internal(format!("Provider error: {}", msg)),
            #[cfg(feature = "oauth")]
            Error::OAuth(msg) => Self::internal(format!("OAuth error: {}", msg)),
            Error::ExternalService(msg) => {
                Self::internal(format!("External service error: {}", msg))
            }
            Error::Serialization(msg) => Self::internal(format!("Serialization error: {}", msg)),
            Error::Internal(msg) => Self::internal(format!("Internal error: {}", msg)),
            Error::Core(msg) => Self::internal(format!("Core error: {}", msg)),
        }
    }
}
