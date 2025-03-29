//! Error handling for the Navius Auth crate.
//!
//! This module defines the error types and error handling functionality
//! for authentication and authorization operations.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A specialized Result type for authentication operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for authentication and authorization operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Authentication failed due to invalid credentials
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Token validation error
    #[error("Invalid token: {0}")]
    TokenInvalid(String),

    /// Token expired
    #[error("Token expired")]
    TokenExpired,

    /// Missing or malformed token
    #[error("Token is missing or malformed")]
    TokenMissing,

    /// Authorization failed - insufficient permissions
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Provider error
    #[error("Provider error: {0}")]
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
    #[error("Internal error: {0}")]
    Internal(String),

    /// Core error (wrapped from navius-core)
    #[error("Core error: {0}")]
    Core(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl Error {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            Error::AuthenticationFailed(_) => 401,
            Error::TokenInvalid(_) => 401,
            Error::TokenExpired => 401,
            Error::TokenMissing => 401,
            Error::AuthorizationFailed(_) => 403,
            Error::Configuration(_) => 500,
            Error::Provider(_) => 500,
            #[cfg(feature = "oauth")]
            Error::OAuth(_) => 500,
            Error::ExternalService(_) => 502,
            Error::Serialization(_) => 400,
            Error::Internal(_) => 500,
            Error::Core(_) => 500,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> String {
        match self {
            Error::AuthenticationFailed(_) => "AUTH_FAILED".to_string(),
            Error::TokenInvalid(_) => "TOKEN_INVALID".to_string(),
            Error::TokenExpired => "TOKEN_EXPIRED".to_string(),
            Error::TokenMissing => "TOKEN_MISSING".to_string(),
            Error::AuthorizationFailed(_) => "AUTHORIZATION_FAILED".to_string(),
            Error::Configuration(_) => "CONFIGURATION_ERROR".to_string(),
            Error::Provider(_) => "PROVIDER_ERROR".to_string(),
            #[cfg(feature = "oauth")]
            Error::OAuth(_) => "OAUTH_ERROR".to_string(),
            Error::ExternalService(_) => "EXTERNAL_SERVICE_ERROR".to_string(),
            Error::Serialization(_) => "SERIALIZATION_ERROR".to_string(),
            Error::Internal(_) => "INTERNAL_ERROR".to_string(),
            Error::Core(_) => "CORE_ERROR".to_string(),
        }
    }

    /// Convert to an error response
    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            code: self.error_code(),
            message: self.to_string(),
            details: None,
        }
    }

    /// Convert to an error response with details
    pub fn to_response_with_details(&self, details: serde_json::Value) -> ErrorResponse {
        ErrorResponse {
            code: self.error_code(),
            message: self.to_string(),
            details: Some(details),
        }
    }

    /// Create an authentication failed error
    pub fn authentication_failed<S: Into<String>>(message: S) -> Self {
        Error::AuthenticationFailed(message.into())
    }

    /// Create a token invalid error
    pub fn token_invalid<S: Into<String>>(message: S) -> Self {
        Error::TokenInvalid(message.into())
    }

    /// Create a token expired error
    pub fn token_expired() -> Self {
        Error::TokenExpired
    }

    /// Create a token missing error
    pub fn token_missing() -> Self {
        Error::TokenMissing
    }

    /// Create an authorization failed error
    pub fn authorization_failed<S: Into<String>>(message: S) -> Self {
        Error::AuthorizationFailed(message.into())
    }

    /// Create a configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Error::Configuration(message.into())
    }

    /// Create a provider error
    pub fn provider<S: Into<String>>(message: S) -> Self {
        Error::Provider(message.into())
    }

    /// Create an OAuth error
    #[cfg(feature = "oauth")]
    pub fn oauth<S: Into<String>>(message: S) -> Self {
        Error::OAuth(message.into())
    }

    /// Create an external service error
    pub fn external_service<S: Into<String>>(message: S) -> Self {
        Error::ExternalService(message.into())
    }

    /// Create a serialization error
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        Error::Serialization(message.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Error::Internal(message.into())
    }

    /// Create a core error
    pub fn core<S: Into<String>>(message: S) -> Self {
        Error::Core(message.into())
    }

    /// Check if this is an authentication error
    pub fn is_authentication_error(&self) -> bool {
        matches!(
            self,
            Error::AuthenticationFailed(_)
                | Error::TokenInvalid(_)
                | Error::TokenExpired
                | Error::TokenMissing
        )
    }

    /// Check if this is an authorization error
    pub fn is_authorization_error(&self) -> bool {
        matches!(self, Error::AuthorizationFailed(_))
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        let status = self.status_code();
        status >= 400 && status < 500
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        let status = self.status_code();
        status >= 500
    }
}

impl From<navius_core::Error> for Error {
    fn from(err: navius_core::Error) -> Self {
        Error::core(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::serialization(format!("JSON error: {}", err))
    }
}

#[cfg(feature = "jwt")]
impl From<jsonwebtoken::errors::Error> for Error {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => Error::token_expired(),
            jsonwebtoken::errors::ErrorKind::InvalidToken => {
                Error::token_invalid("Token is malformed")
            }
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                Error::token_invalid("Invalid token signature")
            }
            _ => Error::token_invalid(format!("JWT error: {}", err)),
        }
    }
}

#[cfg(feature = "oauth")]
impl From<oauth2::basic::BasicRequestTokenError<oauth2::reqwest::Error<reqwest::Error>>> for Error {
    fn from(
        err: oauth2::basic::BasicRequestTokenError<oauth2::reqwest::Error<reqwest::Error>>,
    ) -> Self {
        Error::oauth(err.to_string())
    }
}

#[cfg(feature = "oauth")]
impl From<openid::error::Error> for Error {
    fn from(err: openid::error::Error) -> Self {
        Error::oauth(err.to_string())
    }
}

#[cfg(feature = "http")]
impl From<Error> for navius_http::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::AuthenticationFailed(msg) => navius_http::Error::http(401, msg),
            Error::TokenInvalid(msg) => navius_http::Error::http(401, msg),
            Error::TokenExpired => navius_http::Error::http(401, "Token has expired"),
            Error::TokenMissing => navius_http::Error::http(401, "Authentication token is missing"),
            Error::AuthorizationFailed(msg) => navius_http::Error::http(403, msg),
            Error::Configuration(msg) => navius_http::Error::configuration(msg),
            Error::Provider(msg) => navius_http::Error::internal(msg),
            #[cfg(feature = "oauth")]
            Error::OAuth(msg) => navius_http::Error::internal(msg),
            Error::ExternalService(msg) => navius_http::Error::http(502, msg),
            Error::Serialization(msg) => navius_http::Error::http(400, msg),
            Error::Internal(msg) => navius_http::Error::internal(msg),
            Error::Core(msg) => navius_http::Error::internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        // Authentication errors (401)
        assert_eq!(Error::authentication_failed("test").status_code(), 401);
        assert_eq!(Error::token_invalid("test").status_code(), 401);
        assert_eq!(Error::token_expired().status_code(), 401);
        assert_eq!(Error::token_missing().status_code(), 401);

        // Authorization errors (403)
        assert_eq!(Error::authorization_failed("test").status_code(), 403);

        // Bad request errors (400)
        assert_eq!(Error::serialization("test").status_code(), 400);

        // Server errors (500)
        assert_eq!(Error::configuration("test").status_code(), 500);
        assert_eq!(Error::provider("test").status_code(), 500);
        assert_eq!(Error::oauth("test").status_code(), 500);
        assert_eq!(Error::internal("test").status_code(), 500);
        assert_eq!(Error::core("test").status_code(), 500);

        // Bad gateway errors (502)
        assert_eq!(Error::external_service("test").status_code(), 502);
    }

    #[test]
    fn test_error_categorization() {
        // Authentication errors
        assert!(Error::authentication_failed("test").is_authentication_error());
        assert!(Error::token_invalid("test").is_authentication_error());
        assert!(Error::token_expired().is_authentication_error());
        assert!(Error::token_missing().is_authentication_error());

        // Authorization errors
        assert!(Error::authorization_failed("test").is_authorization_error());

        // Client errors
        assert!(Error::authentication_failed("test").is_client_error());
        assert!(Error::authorization_failed("test").is_client_error());
        assert!(Error::serialization("test").is_client_error());

        // Server errors
        assert!(Error::configuration("test").is_server_error());
        assert!(Error::provider("test").is_server_error());
        assert!(Error::internal("test").is_server_error());
        assert!(Error::core("test").is_server_error());
        assert!(Error::external_service("test").is_server_error());
    }

    #[test]
    fn test_error_response_serialization() {
        let error = Error::authentication_failed("Invalid credentials");
        let response = error.to_response();

        assert_eq!(response.code, "AUTH_FAILED");
        assert_eq!(
            response.message,
            "Authentication failed: Invalid credentials"
        );
        assert!(response.details.is_none());

        // Test with details
        let details = serde_json::json!({
            "field": "password",
            "reason": "too short"
        });
        let response_with_details = error.to_response_with_details(details.clone());

        assert_eq!(response_with_details.code, "AUTH_FAILED");
        assert_eq!(
            response_with_details.message,
            "Authentication failed: Invalid credentials"
        );
        assert_eq!(response_with_details.details, Some(details));
    }
}
