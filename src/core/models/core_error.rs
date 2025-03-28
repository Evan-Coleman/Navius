use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub name: String,
    pub status: String,
    pub details: Option<String>,
}

impl DependencyStatus {
    pub fn new(name: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: status.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub uptime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: String,
    pub dependencies: Vec<DependencyStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoResponse {
    pub status: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub entries: Vec<ActuatorEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorEntry {
    pub name: String,
    pub url: String,
    pub value: String,
}

/// Generic API response structure (simple version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleApiResponse<T> {
    /// Response status code
    pub code: u16,
    /// Response message
    pub message: String,
    /// Response data
    pub data: Option<T>,
}

impl<T> SimpleApiResponse<T> {
    /// Create a new API response
    pub fn new(code: u16, message: String, data: Option<T>) -> Self {
        Self {
            code,
            message,
            data,
        }
    }

    /// Create a success response
    pub fn success(data: T) -> Self {
        Self {
            code: 200,
            message: "Success".to_string(),
            data: Some(data),
        }
    }

    /// Create an error response
    pub fn error(code: u16, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }
}
