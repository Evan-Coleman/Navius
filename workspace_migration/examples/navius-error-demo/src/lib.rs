// Error handling for the Navius framework
//
// This module defines the standard error types and result types used
// throughout the Navius framework.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::error::Error as StdError;
use std::fmt;

/// A specialized Result type for Navius operations
pub type Result<T> = std::result::Result<T, Error>;

/// Standard error codes used across the Navius application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    Configuration,
    Validation,
    Authentication,
    Authorization,
    NotFound,
    Conflict,
    Internal,
    External,
    Timeout,
    Database,
    Cache,
    Plugin,
    Component,
    Serialization,
    Io,
    Unknown,
}

impl ErrorCode {
    /// Get the corresponding HTTP status code for this error code
    pub fn status_code(&self) -> u16 {
        match self {
            ErrorCode::Configuration => 500,
            ErrorCode::Validation => 400,
            ErrorCode::Authentication => 401,
            ErrorCode::Authorization => 403,
            ErrorCode::NotFound => 404,
            ErrorCode::Conflict => 409,
            ErrorCode::Internal => 500,
            ErrorCode::External => 502,
            ErrorCode::Timeout => 408,
            ErrorCode::Database => 500,
            ErrorCode::Cache => 500,
            ErrorCode::Plugin => 500,
            ErrorCode::Component => 500,
            ErrorCode::Serialization => 400,
            ErrorCode::Io => 500,
            ErrorCode::Unknown => 500,
        }
    }

    /// Get the default message for this error code
    pub fn default_message(&self) -> &'static str {
        match self {
            ErrorCode::Configuration => "Configuration error",
            ErrorCode::Validation => "Validation error",
            ErrorCode::Authentication => "Authentication failed",
            ErrorCode::Authorization => "Not authorized",
            ErrorCode::NotFound => "Resource not found",
            ErrorCode::Conflict => "Resource conflict",
            ErrorCode::Internal => "Internal server error",
            ErrorCode::External => "External service error",
            ErrorCode::Timeout => "Operation timed out",
            ErrorCode::Database => "Database error",
            ErrorCode::Cache => "Cache error",
            ErrorCode::Plugin => "Plugin error",
            ErrorCode::Component => "Component error",
            ErrorCode::Serialization => "Serialization error",
            ErrorCode::Io => "I/O error",
            ErrorCode::Unknown => "Unknown error",
        }
    }
}

/// The core error type for the Navius framework
#[derive(Debug)]
pub struct Error {
    /// The error code category
    pub code: ErrorCode,
    /// User-friendly error message
    pub message: String,
    /// The original error that caused this error, if any
    pub source: Option<Box<dyn StdError + Send + Sync>>,
    /// Additional details about the error, if any
    pub details: Option<Value>,
    /// Request ID for tracing, if available
    pub request_id: Option<String>,
}

impl Error {
    /// Create a new error with a code and message
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source: None,
            details: None,
            request_id: None,
        }
    }

    /// Create a new configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Configuration, message)
    }

    /// Create a new validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Validation, message)
    }

    /// Create a new authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Authentication, message)
    }

    /// Create a new authorization error
    pub fn authorization(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Authorization, message)
    }

    /// Create a new not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    /// Create a new conflict error
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Conflict, message)
    }

    /// Create a new internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    /// Create a new external error
    pub fn external(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::External, message)
    }

    /// Create a new timeout error
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Timeout, message)
    }

    /// Create a new database error
    pub fn database(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Database, message)
    }

    /// Create a new cache error
    pub fn cache(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Cache, message)
    }

    /// Create a new plugin error
    pub fn plugin(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Plugin, message)
    }

    /// Create a new component error
    pub fn component(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Component, message)
    }

    /// Create a new error from a source error with a specific error code
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Add additional details to the error
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Add a request ID to the error for tracing
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Convert the error to a JSON response format
    pub fn to_json(&self) -> Value {
        let mut error = json!({
            "error": {
                "code": format!("{:?}", self.code).to_lowercase(),
                "message": self.message,
                "status": self.code.status_code(),
            }
        });

        if let Some(details) = &self.details {
            error["error"]["details"] = details.clone();
        }

        if let Some(request_id) = &self.request_id {
            error["error"]["request_id"] = json!(request_id);
        }

        error
    }

    /// Check if this is a configuration error
    pub fn is_configuration(&self) -> bool {
        self.code == ErrorCode::Configuration
    }

    /// Check if this is a validation error
    pub fn is_validation(&self) -> bool {
        self.code == ErrorCode::Validation
    }

    /// Check if this is a not found error
    pub fn is_not_found(&self) -> bool {
        self.code == ErrorCode::NotFound
    }

    /// Check if this is a conflict error
    pub fn is_conflict(&self) -> bool {
        self.code == ErrorCode::Conflict
    }

    /// Check if this is an internal error
    pub fn is_internal(&self) -> bool {
        self.code == ErrorCode::Internal
    }

    /// Check if this is an external error
    pub fn is_external(&self) -> bool {
        self.code == ErrorCode::External
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|s| s.as_ref() as &(dyn StdError + 'static))
    }
}

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

/// Extension trait for Result to make error handling more convenient
pub trait ResultExt<T, E> {
    /// Convert the error to a Navius error with a specific error code and message
    fn with_context<F>(self, code: ErrorCode, message_fn: F) -> Result<T>
    where
        F: FnOnce() -> String;

    /// Convert the error to a configuration error
    fn configuration<S: Into<String>>(self, message: S) -> Result<T>;

    /// Convert the error to a validation error
    fn validation<S: Into<String>>(self, message: S) -> Result<T>;

    /// Convert the error to a not found error
    fn not_found<S: Into<String>>(self, message: S) -> Result<T>;

    /// Convert the error to a conflict error
    fn conflict<S: Into<String>>(self, message: S) -> Result<T>;

    /// Convert the error to an internal error
    fn internal<S: Into<String>>(self, message: S) -> Result<T>;

    /// Convert the error to an external error
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

    fn conflict<S: Into<String>>(self, message: S) -> Result<T> {
        self.map_err(|e| Error::conflict(message).with_source(e))
    }

    fn internal<S: Into<String>>(self, message: S) -> Result<T> {
        self.map_err(|e| Error::internal(message).with_source(e))
    }

    fn external<S: Into<String>>(self, message: S) -> Result<T> {
        self.map_err(|e| Error::external(message).with_source(e))
    }
}

// Demo code to showcase error handling in action
pub mod demo {
    use super::*;
    use std::io;
    use std::path::Path;

    /// A mock function that simulates a database query that could fail
    pub fn db_query(should_fail: bool) -> Result<String> {
        if should_fail {
            Err(Error::database("Database connection failed"))
        } else {
            Ok("query result".to_string())
        }
    }

    /// A mock function that simulates file operations that could fail
    pub fn file_read(path: &Path) -> std::result::Result<String, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {}", path.display()),
        ))
    }

    /// A function that simulates a chain of operations with error handling
    pub fn process_request(input: &str) -> Result<String> {
        // First step: validate input
        if input.is_empty() {
            return Err(Error::validation("Input cannot be empty"));
        }

        // Second step: try to read from a file (will fail)
        let file_path = Path::new("/non/existent/file.txt");
        let _file_content = file_read(file_path).not_found(format!(
            "Could not find configuration file at {}",
            file_path.display()
        ))?;

        // This code won't be reached due to the error above
        let db_result = db_query(false)?;

        Ok(format!("Processed: {} with {}", input, db_result))
    }

    /// A function that uses with_context for detailed error messages
    pub fn access_resource(should_fail: bool) -> Result<()> {
        let result: std::result::Result<(), io::Error> = if should_fail {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Access denied",
            ))
        } else {
            Ok(())
        };

        result.with_context(ErrorCode::Authorization, || {
            format!(
                "Failed to access resource at {}",
                std::time::SystemTime::now().elapsed().unwrap().as_secs()
            )
        })
    }

    /// A function that constructs a detailed error with additional context
    pub fn create_detailed_error() -> Error {
        Error::validation("Username is invalid")
            .with_details(json!({
                "field": "username",
                "validation": {
                    "min_length": 3,
                    "max_length": 20,
                    "pattern": "^[a-zA-Z0-9_]+$"
                },
                "received": "u$er"
            }))
            .with_request_id("req-12345-abcde")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_creation() {
        let err = Error::new(ErrorCode::Validation, "Invalid input");
        assert_eq!(err.code, ErrorCode::Validation);
        assert_eq!(err.message, "Invalid input");
        assert!(err.source.is_none());
    }

    #[test]
    fn test_error_with_source() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let err = Error::validation("Invalid file").with_source(io_err);
        assert_eq!(err.code, ErrorCode::Validation);
        assert_eq!(err.message, "Invalid file");
        assert!(err.source.is_some());
    }

    #[test]
    fn test_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let err: Error = io_err.into();
        assert_eq!(err.code, ErrorCode::Io);
        assert!(err.message.contains("I/O error"));
        assert!(err.source.is_some());
    }

    #[test]
    fn test_demo_process_request() {
        let result = demo::process_request("test_input");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.is_not_found());
        assert!(err.message.contains("configuration file"));
        assert!(err.source.is_some());
    }

    #[test]
    fn test_demo_access_resource() {
        let result = demo::access_resource(true);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::Authorization);
        assert!(err.message.contains("Failed to access resource"));
        assert!(err.source.is_some());
    }

    #[test]
    fn test_demo_detailed_error() {
        let error = demo::create_detailed_error();
        let json = error.to_json();

        assert_eq!(json["error"]["code"], "validation");
        assert_eq!(json["error"]["message"], "Username is invalid");
        assert_eq!(json["error"]["details"]["field"], "username");
        assert_eq!(json["error"]["details"]["received"], "u$er");
        assert_eq!(json["error"]["details"]["validation"]["min_length"], 3);
        assert_eq!(json["error"]["request_id"], "req-12345-abcde");
    }

    #[test]
    fn test_result_ext() {
        // Create a std::io::Error
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
        let std_result: std::result::Result<(), io::Error> = Err(io_err);

        // Convert to different error types using ResultExt
        let validation_error = std_result.validation("Invalid file permissions");
        assert!(validation_error.is_err());
        let err = validation_error.unwrap_err();
        assert_eq!(err.code, ErrorCode::Validation);
        assert_eq!(err.message, "Invalid file permissions");
    }

    #[test]
    fn test_error_code_mapping() {
        assert_eq!(ErrorCode::Validation.status_code(), 400);
        assert_eq!(ErrorCode::Authentication.status_code(), 401);
        assert_eq!(ErrorCode::Authorization.status_code(), 403);
        assert_eq!(ErrorCode::NotFound.status_code(), 404);
        assert_eq!(ErrorCode::Conflict.status_code(), 409);
        assert_eq!(ErrorCode::Internal.status_code(), 500);
    }
}
