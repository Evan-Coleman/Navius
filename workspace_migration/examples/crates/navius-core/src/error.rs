//! Error handling for the Navius framework.
//!
//! This module defines the core error types and result type used throughout Navius.
//! It provides a standardized approach to error handling with consistent categorization,
//! context preservation, and user-friendly error messages.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

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
}

impl ErrorCode {
    /// Get the HTTP status code associated with this error code.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Validation => 400,
            Self::Authentication => 401,
            Self::Authorization => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::Timeout => 408,
            Self::Internal | Self::Component | Self::Unknown => 500,
            Self::External => 502,
            Self::Database | Self::Cache | Self::Plugin | Self::Serialization | Self::Io => 500,
            Self::Configuration => 500,
        }
    }

    /// Returns a default message for this error code.
    pub fn default_message(&self) -> &'static str {
        match self {
            Self::Configuration => "A configuration error occurred",
            Self::Validation => "Validation failed for the provided input",
            Self::Authentication => "Authentication failed",
            Self::Authorization => "You don't have permission to perform this action",
            Self::NotFound => "The requested resource was not found",
            Self::Conflict => "A conflict occurred with the current state",
            Self::Internal => "An internal server error occurred",
            Self::External => "An error occurred while communicating with an external service",
            Self::Timeout => "The operation timed out",
            Self::Database => "A database error occurred",
            Self::Cache => "A cache error occurred",
            Self::Plugin => "A plugin error occurred",
            Self::Component => "A component error occurred",
            Self::Serialization => "A serialization error occurred",
            Self::Io => "An I/O error occurred",
            Self::Unknown => "An unknown error occurred",
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
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    /// Optional additional details about the error
    pub details: Option<serde_json::Value>,
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
    pub fn with_source<E>(code: ErrorCode, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            code,
            message: code.default_message().to_string(),
            source: Some(Box::new(source)),
            details: None,
            request_id: None,
        }
    }

    /// Add details to this error.
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
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

    /// Convert this error to a JSON response.
    pub fn to_json(&self) -> serde_json::Value {
        let mut error = serde_json::json!({
            "code": self.code,
            "message": self.message,
        });

        if let Some(details) = &self.details {
            error["details"] = details.clone();
        }

        if let Some(request_id) = &self.request_id {
            error["request_id"] = serde_json::Value::String(request_id.clone());
        }

        serde_json::json!({ "error": error })
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

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

// Common From implementations for standard errors

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::with_source(ErrorCode::Io, err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::with_source(ErrorCode::Serialization, err)
    }
}

/// Extension trait for Result that provides useful utility methods.
pub trait ResultExt<T, E> {
    /// Add context to the error and convert it to a Navius Error.
    fn with_context<C, F>(self, code: ErrorCode, f: F) -> Result<T>
    where
        C: Into<String>,
        F: FnOnce() -> C;

    /// Convert any error to a configuration error.
    fn configuration<C: Into<String>>(self, context: C) -> Result<T>;

    /// Convert any error to a validation error.
    fn validation<C: Into<String>>(self, context: C) -> Result<T>;

    /// Convert any error to a not found error.
    fn not_found<C: Into<String>>(self, context: C) -> Result<T>;

    /// Convert any error to an internal error.
    fn internal<C: Into<String>>(self, context: C) -> Result<T>;

    /// Convert any error to an external error.
    fn external<C: Into<String>>(self, context: C) -> Result<T>;
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: std::fmt::Display + 'static + std::error::Error + Send + Sync,
{
    fn with_context<C, F>(self, code: ErrorCode, f: F) -> Result<T>
    where
        C: Into<String>,
        F: FnOnce() -> C,
    {
        self.map_err(|err| {
            let ctx = f().into();
            Error {
                code,
                message: ctx,
                source: Some(Box::new(err)),
                details: None,
                request_id: None,
            }
        })
    }

    fn configuration<C: Into<String>>(self, context: C) -> Result<T> {
        self.with_context(ErrorCode::Configuration, || context)
    }

    fn validation<C: Into<String>>(self, context: C) -> Result<T> {
        self.with_context(ErrorCode::Validation, || context)
    }

    fn not_found<C: Into<String>>(self, context: C) -> Result<T> {
        self.with_context(ErrorCode::NotFound, || context)
    }

    fn internal<C: Into<String>>(self, context: C) -> Result<T> {
        self.with_context(ErrorCode::Internal, || context)
    }

    fn external<C: Into<String>>(self, context: C) -> Result<T> {
        self.with_context(ErrorCode::External, || context)
    }
}

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
        let err = Error::with_source(ErrorCode::Io, io_err);
        assert_eq!(err.code, ErrorCode::Io);
        assert_eq!(err.message, "An I/O error occurred");
        assert!(err.source.is_some());
    }

    #[test]
    fn test_error_with_details() {
        let details = serde_json::json!({
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
            .with_details(serde_json::json!({"user_id": "123"}))
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
            .with_context(ErrorCode::Authorization, || "Failed to access file")
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
        assert_eq!(ErrorCode::Validation.status_code(), 400);
        assert_eq!(ErrorCode::Authentication.status_code(), 401);
        assert_eq!(ErrorCode::Authorization.status_code(), 403);
        assert_eq!(ErrorCode::NotFound.status_code(), 404);
        assert_eq!(ErrorCode::Conflict.status_code(), 409);
        assert_eq!(ErrorCode::Internal.status_code(), 500);
        assert_eq!(ErrorCode::External.status_code(), 502);
    }
}
