//! Error handling for the Navius framework.
//!
//! This module defines the core error types and result type used throughout Navius.
//! It provides a standardized approach to error handling with consistent categorization,
//! context preservation, and user-friendly error messages.

// Use imports only if features are enabled
// #[cfg(feature = "axum")]
// use axum::Json;
// #[cfg(feature = "axum")]
// use axum::response::{IntoResponse, Response};
// #[cfg(feature = "http")]
// use http::{
//     StatusCode,
//     header::{HeaderName, HeaderValue},
// };

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
// use std::collections::HashMap; // Unused
use std::error::Error as StdError;
// use uuid::Uuid; // Unused

/// A specialized Result type for Navius operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error code enum to categorize different types of errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Configuration-related errors (missing config, invalid format, etc.)
    Configuration,
    /// Validation errors (invalid input, format, etc.)
    Validation,
    /// Authentication errors (invalid credentials, expired token, etc.)
    Authentication,
    /// Authorization errors (insufficient permissions, etc.)
    Authorization,
    /// Not found errors (resource doesn't exist)
    NotFound,
    /// Conflict errors (resource already exists, version conflict, etc.)
    Conflict,
    /// Internal server errors (unexpected failures)
    Internal,
    /// External service errors (third-party API failures)
    External,
    /// Timeout errors (operation took too long)
    Timeout,
    /// Database errors
    Database,
    /// Cache errors
    Cache,
    /// Plugin errors
    Plugin,
    /// Component errors (DI system)
    Component,
    /// Serialization/deserialization errors
    Serialization,
    /// I/O errors
    Io,
    /// Unknown errors (fallback)
    Unknown,
    /// Invalid argument errors
    InvalidArgument,
}

impl ErrorCode {
    /// Get the HTTP status code associated with this error code.
    // #[cfg(feature = "http")]
    // pub fn status_code(&self) -> StatusCode {
    //     match self {
    //         Self::Validation => StatusCode::BAD_REQUEST,
    //         Self::Authentication => StatusCode::UNAUTHORIZED,
    //         Self::Authorization => StatusCode::FORBIDDEN,
    //         Self::NotFound => StatusCode::NOT_FOUND,
    //         Self::Conflict => StatusCode::CONFLICT,
    //         Self::Timeout => StatusCode::GATEWAY_TIMEOUT,
    //         Self::External => StatusCode::SERVICE_UNAVAILABLE,
    //         _ => StatusCode::INTERNAL_SERVER_ERROR,
    //     }
    // }

    /// Returns a default message for this error code.
    pub fn default_message(&self) -> &'static str {
        match self {
            Self::Configuration => "Configuration error",
            Self::Validation => "Validation error",
            Self::Authentication => "Authentication failed",
            Self::Authorization => "Not authorized",
            Self::NotFound => "Resource not found",
            Self::Conflict => "Resource conflict",
            Self::Internal => "Internal server error",
            Self::External => "External service error",
            Self::Timeout => "Operation timed out",
            Self::Database => "Database error",
            Self::Cache => "Cache error",
            Self::Plugin => "Plugin error",
            Self::Component => "Component error",
            Self::Serialization => "Serialization error",
            Self::Io => "I/O error",
            Self::Unknown => "Unknown error",
            Self::InvalidArgument => "Invalid argument",
        }
    }

    /// Returns a string representation of the error code.
    pub fn as_code_str(&self) -> &'static str {
        match self {
            Self::Unknown => "UNKNOWN_ERROR",
            Self::Internal => "INTERNAL_SERVER_ERROR",
            Self::Configuration => "CONFIGURATION_ERROR",
            Self::Validation => "VALIDATION_ERROR",
            Self::Database => "DATABASE_ERROR",
            Self::NotFound => "NOT_FOUND",
            Self::Authentication => "AUTHENTICATION_ERROR",
            Self::Authorization => "AUTHORIZATION_ERROR",
            Self::External => "EXTERNAL_SERVICE_ERROR",
            Self::Timeout => "TIMEOUT_ERROR",
            Self::Plugin => "PLUGIN_ERROR",
            Self::Component => "COMPONENT_ERROR",
            Self::Serialization => "SERIALIZATION_ERROR",
            Self::Io => "IO_ERROR",
            Self::Cache => "CACHE_ERROR",
            Self::Conflict => "CONFLICT_ERROR",
            Self::InvalidArgument => "INVALID_ARGUMENT",
        }
    }
}

/// Core error struct for the Navius framework.
#[derive(Debug)]
pub struct Error {
    /// The error code categorizing the error
    pub code: ErrorCode,
    /// A user-friendly error message
    pub message: String,
    /// Optional source error that caused this error
    pub source: Option<Box<dyn StdError + Send + Sync>>,
    /// Optional additional details about the error
    pub details: Option<Value>,
    /// Optional request ID for tracing
    pub request_id: Option<String>,
}

impl Error {
    /// Create a new error with the given code and message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source: None,
            details: None,
            request_id: None,
        }
    }

    /// Create a new error with the given code and source error.
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Add details to this error.
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Add a request ID to this error.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Create a configuration error.
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Configuration, message)
    }

    /// Create a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Validation, message)
    }

    /// Create an authentication error.
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Authentication, message)
    }

    /// Create an authorization error.
    pub fn authorization(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Authorization, message)
    }

    /// Create a not found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    /// Create a conflict error.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Conflict, message)
    }

    /// Create an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    /// Create an external error.
    pub fn external(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::External, message)
    }

    /// Create a timeout error.
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Timeout, message)
    }

    /// Create a database error.
    pub fn database(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Database, message)
    }

    /// Create a cache error.
    pub fn cache(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Cache, message)
    }

    /// Create a plugin error.
    pub fn plugin(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Plugin, message)
    }

    /// Create a component error.
    pub fn component(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Component, message)
    }

    /// Create a new invalid argument error
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidArgument, message)
    }

    /// Convert this error to a JSON response.
    pub fn to_json(&self) -> Value {
        json!({
            "error": {
                "code": self.code.as_code_str(),
                "message": self.message,
                // "status": self.code.status_code(), // FIXME: status_code was removed, need web-layer mapping
                "details": self.details,
                "source": self.source.as_ref().map(|s| s.to_string()),
                "request_id": self.request_id,
            }
        })
    }

    /// Returns true if this error matches the given code.
    pub fn is_code(&self, code: ErrorCode) -> bool {
        self.code == code
    }

    /// Returns true if this is a validation error.
    pub fn is_validation(&self) -> bool {
        self.is_code(ErrorCode::Validation)
    }

    /// Returns true if this is an authentication error.
    pub fn is_authentication(&self) -> bool {
        self.is_code(ErrorCode::Authentication)
    }

    /// Returns true if this is an authorization error.
    pub fn is_authorization(&self) -> bool {
        self.is_code(ErrorCode::Authorization)
    }

    /// Returns true if this is a not found error.
    pub fn is_not_found(&self) -> bool {
        self.is_code(ErrorCode::NotFound)
    }

    /// Returns true if this is a conflict error.
    pub fn is_conflict(&self) -> bool {
        self.is_code(ErrorCode::Conflict)
    }

    /// Returns true if this is an internal error.
    pub fn is_internal(&self) -> bool {
        self.is_code(ErrorCode::Internal)
    }

    /// Returns true if this is a configuration error.
    pub fn is_configuration(&self) -> bool {
        self.is_code(ErrorCode::Configuration)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(source) = &self.source {
            write!(f, ": {}", source)?;
        }
        Ok(())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn StdError + 'static))
    }
}

// Common From implementations for standard errors

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::new(ErrorCode::Io, format!("I/O error: {}", err)).with_source(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::new(
            ErrorCode::Serialization,
            format!("JSON serialization error: {}", err),
        )
        .with_source(err)
    }
}

// impl From<redis::RedisError> for Error {
//     fn from(err: redis::RedisError) -> Self {
//         Error::new(ErrorCode::Cache, err.to_string())
//     }
// }

/// Extension trait for Result that provides useful utility methods.
pub trait ResultExt<T, E> {
    /// Add context to the error and convert it to a Navius Error.
    fn with_context<F>(self, code: ErrorCode, message_fn: F) -> Result<T>
    where
        F: FnOnce() -> String;

    /// Convert any error to a configuration error.
    fn configuration<S: Into<String>>(self, message: S) -> Result<T>;

    /// Convert any error to a validation error.
    fn validation<S: Into<String>>(self, message: S) -> Result<T>;

    /// Convert any error to a not found error.
    fn not_found<S: Into<String>>(self, message: S) -> Result<T>;

    /// Convert any error to an internal error.
    fn internal<S: Into<String>>(self, message: S) -> Result<T>;

    /// Convert any error to an external error.
    fn external<S: Into<String>>(self, message: S) -> Result<T>;
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn with_context<F>(self, code: ErrorCode, message_fn: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| Error::new(code, message_fn()).with_source(e))
    }

    fn configuration<S: Into<String>>(self, message: S) -> Result<T> {
        self.map_err(|e| Error::configuration(message).with_source(e))
    }

    fn validation<S: Into<String>>(self, message: S) -> Result<T> {
        self.map_err(|e| Error::validation(message).with_source(e))
    }

    fn not_found<S: Into<String>>(self, message: S) -> Result<T> {
        self.map_err(|e| Error::not_found(message).with_source(e))
    }

    fn internal<S: Into<String>>(self, message: S) -> Result<T> {
        self.map_err(|e| Error::internal(message).with_source(e))
    }

    fn external<S: Into<String>>(self, message: S) -> Result<T> {
        self.map_err(|e| Error::external(message).with_source(e))
    }
}

// Implement IntoResponse for Error - gated by features
// #[cfg(all(feature = "axum", feature = "http"))]
// impl IntoResponse for Error {
//     fn into_response(self) -> Response {
//         let status_code = self.code.status_code();
//         let request_id = self
//             .request_id
//             .unwrap_or_else(|| Uuid::new_v4().to_string());
//
//         let body = json!({
//             "error": {
//                 "code": self.code.as_code_str(), // Call helper method
//                 "message": self.message,
//                 // ... optional details/source ...
//             },
//             "request_id": request_id,
//         });
//
//         const REQUEST_ID_HEADER_NAME: HeaderName = HeaderName::from_static("x-request-id");
//
//         let mut response = (status_code, Json(body)).into_response();
//         match HeaderValue::from_str(&request_id) {
//             Ok(val) => {
//                 response.headers_mut().insert(REQUEST_ID_HEADER_NAME, val);
//             }
//             Err(_) => {
//                 response.headers_mut().insert(
//                     REQUEST_ID_HEADER_NAME,
//                     HeaderValue::from_static("invalid-request-id"),
//                 );
//             }
//         }
//         response
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn test_error_creation() {
        let err = Error::validation("Invalid input");
        assert_eq!(err.code, ErrorCode::Validation);
        assert_eq!(err.message, "Invalid input");
        assert!(err.source.is_none());
        assert!(err.details.is_none());
        assert!(err.request_id.is_none());
    }

    #[test]
    fn test_error_with_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let err = Error::validation("Invalid file").with_source(io_err);
        assert_eq!(err.code, ErrorCode::Validation);
        assert_eq!(err.message, "Invalid file");
        assert!(err.source.is_some());
    }

    #[test]
    fn test_error_with_details() {
        let details = json!({
            "field": "username",
            "reason": "too short"
        });
        let err = Error::validation("Invalid input").with_details(details.clone());
        assert_eq!(err.code, ErrorCode::Validation);
        assert_eq!(err.message, "Invalid input");
        assert_eq!(err.details, Some(details));
    }

    #[test]
    fn test_error_with_request_id() {
        let err = Error::internal("Something went wrong").with_request_id("req-123");
        assert_eq!(err.code, ErrorCode::Internal);
        assert_eq!(err.message, "Something went wrong");
        assert_eq!(err.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_error_to_json() {
        let err = Error::not_found("User not found")
            .with_details(json!({"user_id": "123"}))
            .with_request_id("req-456");

        let json = err.to_json();
        assert_eq!(json["error"]["code"], "not_found");
        assert_eq!(json["error"]["message"], "User not found");
        assert_eq!(json["error"]["details"]["user_id"], "123");
        assert_eq!(json["error"]["request_id"], "req-456");
    }

    #[test]
    fn test_result_ext() {
        let result: std::result::Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Permission denied",
        ));

        // Test with_context
        let err = result
            .clone()
            .with_context(ErrorCode::Authorization, || {
                "Failed to access file".to_string()
            })
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Authorization);
        assert_eq!(err.message, "Failed to access file");
        assert!(err.source.is_some());

        // Test validation
        let err = result.clone().validation("Invalid file path").unwrap_err();
        assert_eq!(err.code, ErrorCode::Validation);
        assert_eq!(err.message, "Invalid file path");
        assert!(err.is_validation());

        // Test not_found
        let err = result.clone().not_found("File not found").unwrap_err();
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(err.message, "File not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn test_from_implementations() {
        // Test From<std::io::Error>
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "IO error");
        let err: Error = io_err.into();
        assert_eq!(err.code, ErrorCode::Io);

        // Test From<serde_json::Error>
        let json_str = r#"{"invalid": json"#;
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let err: Error = json_err.into();
        assert_eq!(err.code, ErrorCode::Serialization);
    }

    #[test]
    fn test_error_categorization() {
        // HTTP status codes are now handled at the web layer, remove these checks
        // assert_eq!(ErrorCode::Validation.status_code(), 400);
        // assert_eq!(ErrorCode::Authentication.status_code(), 401);
        // assert_eq!(ErrorCode::Authorization.status_code(), 403);
        // assert_eq!(ErrorCode::NotFound.status_code(), 404);
        // assert_eq!(ErrorCode::Conflict.status_code(), 409);
        // assert_eq!(ErrorCode::Internal.status_code(), 500);
        // assert_eq!(ErrorCode::External.status_code(), 502);

        // Test default messages and code strings instead
        assert_eq!(ErrorCode::Validation.default_message(), "Validation error");
        assert_eq!(ErrorCode::Validation.as_code_str(), "VALIDATION_ERROR");
        assert_eq!(
            ErrorCode::Internal.default_message(),
            "Internal server error"
        );
        assert_eq!(ErrorCode::Internal.as_code_str(), "INTERNAL_SERVER_ERROR");
    }
}
