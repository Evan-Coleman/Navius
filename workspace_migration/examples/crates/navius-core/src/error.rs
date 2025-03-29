//! Error handling for the Navius framework.
//!
//! This module defines the core error types and result type used throughout Navius.

use std::fmt;
use thiserror::Error;

/// A specialized Result type for Navius operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Core error type for the Navius framework.
#[derive(Error, Debug)]
pub enum Error {
    /// An error occurred in the configuration subsystem.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A serialization or deserialization error occurred.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// An error occurred in a third-party library.
    #[error("External error: {0}")]
    External(String),

    /// An internal error occurred within Navius.
    #[error("Internal error: {0}")]
    Internal(String),

    /// An error that wraps multiple other errors.
    #[error("Multiple errors: {0}")]
    Multiple(String),

    /// A validation error occurred.
    #[error("Validation error: {0}")]
    Validation(String),

    /// A resource was not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// A conflict occurred.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// An error that is specific to a Navius module.
    #[error("Module error: {0}")]
    Module { name: String, message: String },
}

impl Error {
    /// Create a new configuration error.
    pub fn configuration<T: fmt::Display>(msg: T) -> Self {
        Self::Configuration(msg.to_string())
    }

    /// Create a new external error.
    pub fn external<T: fmt::Display>(msg: T) -> Self {
        Self::External(msg.to_string())
    }

    /// Create a new internal error.
    pub fn internal<T: fmt::Display>(msg: T) -> Self {
        Self::Internal(msg.to_string())
    }

    /// Create a new validation error.
    pub fn validation<T: fmt::Display>(msg: T) -> Self {
        Self::Validation(msg.to_string())
    }

    /// Create a new not found error.
    pub fn not_found<T: fmt::Display>(msg: T) -> Self {
        Self::NotFound(msg.to_string())
    }

    /// Create a new conflict error.
    pub fn conflict<T: fmt::Display>(msg: T) -> Self {
        Self::Conflict(msg.to_string())
    }

    /// Create a new module error.
    pub fn module<N: fmt::Display, M: fmt::Display>(name: N, msg: M) -> Self {
        Self::Module {
            name: name.to_string(),
            message: msg.to_string(),
        }
    }

    /// Returns true if this is a configuration error.
    pub fn is_configuration(&self) -> bool {
        matches!(self, Self::Configuration(_))
    }

    /// Returns true if this is a validation error.
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation(_))
    }

    /// Returns true if this is a not found error.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Returns true if this is a conflict error.
    pub fn is_conflict(&self) -> bool {
        matches!(self, Self::Conflict(_))
    }
}

/// Extension trait for Result that provides useful utility methods.
pub trait ResultExt<T> {
    /// Convert an error to a configuration error.
    fn configuration<C: fmt::Display>(self, context: C) -> Result<T>;

    /// Convert an error to an internal error.
    fn internal<C: fmt::Display>(self, context: C) -> Result<T>;

    /// Convert an error to an external error.
    fn external<C: fmt::Display>(self, context: C) -> Result<T>;
}

impl<T, E: fmt::Display> ResultExt<T> for std::result::Result<T, E> {
    fn configuration<C: fmt::Display>(self, context: C) -> Result<T> {
        self.map_err(|e| Error::configuration(format!("{}: {}", context, e)))
    }

    fn internal<C: fmt::Display>(self, context: C) -> Result<T> {
        self.map_err(|e| Error::internal(format!("{}: {}", context, e)))
    }

    fn external<C: fmt::Display>(self, context: C) -> Result<T> {
        self.map_err(|e| Error::external(format!("{}: {}", context, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_construction() {
        let err = Error::configuration("Missing config key");
        assert!(err.is_configuration());
        assert!(!err.is_validation());

        let err = Error::validation("Invalid value");
        assert!(err.is_validation());
        assert!(!err.is_not_found());
    }

    #[test]
    fn test_result_ext() {
        let result: std::result::Result<(), &str> = Err("something failed");
        let err = result.configuration("Config loading").unwrap_err();
        assert!(err.is_configuration());
        assert!(err.to_string().contains("Config loading"));
        assert!(err.to_string().contains("something failed"));
    }
}
