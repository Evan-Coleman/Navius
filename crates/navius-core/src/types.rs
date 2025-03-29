//! Common types used throughout the Navius framework.
//!
//! This module provides type definitions that are used across different parts of the framework.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

/// A unique identifier for resources.
pub type Id = Uuid;

/// A simple key-value map for metadata.
pub type Metadata = HashMap<String, String>;

/// A timestamp in milliseconds since the Unix epoch.
pub type Timestamp = i64;

/// Response status enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    /// The operation was successful.
    Ok,

    /// The operation failed.
    Error,

    /// The operation is still in progress.
    Pending,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Ok => write!(f, "OK"),
            Status::Error => write!(f, "ERROR"),
            Status::Pending => write!(f, "PENDING"),
        }
    }
}

/// A trait for types that can be identified by an ID.
pub trait Identifiable {
    /// Get the ID of this object.
    fn id(&self) -> Id;
}

/// A trait for types that have metadata.
pub trait WithMetadata {
    /// Get the metadata of this object.
    fn metadata(&self) -> &Metadata;

    /// Get a mutable reference to the metadata of this object.
    fn metadata_mut(&mut self) -> &mut Metadata;
}

/// A trait for types that have a timestamp.
pub trait Timestamped {
    /// Get the created timestamp of this object.
    fn created_at(&self) -> Timestamp;

    /// Get the updated timestamp of this object.
    fn updated_at(&self) -> Timestamp;
}

/// A shared reference to any type that implements `Send + Sync`.
pub type SharedRef<T> = Arc<T>;

/// Create a new shared reference.
pub fn shared<T>(value: T) -> SharedRef<T> {
    Arc::new(value)
}

/// A standard response wrapper for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// The status of the response.
    pub status: Status,

    /// An optional message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// The response data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Create a new successful response.
    pub fn ok(data: T) -> Self {
        Self {
            status: Status::Ok,
            message: None,
            data: Some(data),
        }
    }

    /// Create a new successful response with a message.
    pub fn ok_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            status: Status::Ok,
            message: Some(message.into()),
            data: Some(data),
        }
    }

    /// Create a new error response.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: Status::Error,
            message: Some(message.into()),
            data: None,
        }
    }

    /// Create a new pending response.
    pub fn pending(message: impl Into<String>) -> Self {
        Self {
            status: Status::Pending,
            message: Some(message.into()),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_display() {
        assert_eq!(Status::Ok.to_string(), "OK");
        assert_eq!(Status::Error.to_string(), "ERROR");
        assert_eq!(Status::Pending.to_string(), "PENDING");
    }

    #[test]
    fn test_api_response_success() {
        let data = "test data";
        let response = ApiResponse::ok(data);
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.data, Some(data));
        assert_eq!(response.message, None);
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse::error("error message");
        assert_eq!(response.status, Status::Error);
        assert_eq!(response.data, None);
        assert_eq!(response.message, Some("error message".to_string()));
    }
}
