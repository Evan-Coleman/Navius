//! Data Transfer Objects for app services
//!
//! This module previously contained Pet-related DTOs,
//! which have been removed to simplify the application.

/// Common response format for service operations
pub struct ServiceResponse<T> {
    /// Data payload
    pub data: Option<T>,
    /// Success status
    pub success: bool,
    /// Error message if any
    pub message: Option<String>,
}

impl<T> ServiceResponse<T> {
    /// Create a successful response with data
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            success: true,
            message: None,
        }
    }

    /// Create a successful response without data
    pub fn success_no_data() -> Self {
        Self {
            data: None,
            success: true,
            message: None,
        }
    }

    /// Create an error response
    pub fn error(message: String) -> Self {
        Self {
            data: None,
            success: false,
            message: Some(message),
        }
    }
}
