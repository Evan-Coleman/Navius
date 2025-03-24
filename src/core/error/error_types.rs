use axum::{
    Json,
    response::{IntoResponse, Response},
};
use config::ConfigError;
use metrics::counter;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{fmt, result};
use thiserror::Error;
use tracing::{error, warn};

pub type Result<T> = result::Result<T, AppError>;

// The struct to be returned from the API in case of an error
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,      // Minor issues that don't affect functionality
    Medium,   // Issues that affect functionality but don't cause system failure
    High,     // Critical issues that could cause system instability
    Critical, // Severe issues requiring immediate attention
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "low"),
            ErrorSeverity::Medium => write!(f, "medium"),
            ErrorSeverity::High => write!(f, "high"),
            ErrorSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Application error types
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("HTTP client error: {0}")]
    ClientError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl AppError {
    // Get the severity level of the error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::NotFound(_) => ErrorSeverity::Low,
            AppError::BadRequest(_) => ErrorSeverity::Low,
            AppError::ValidationError(_) => ErrorSeverity::Low,
            AppError::Unauthorized(_) => ErrorSeverity::Medium,
            AppError::Forbidden(_) => ErrorSeverity::Medium,
            AppError::RateLimited(_) => ErrorSeverity::Medium,
            AppError::CacheError(_) => ErrorSeverity::Medium,
            AppError::ClientError(_) => ErrorSeverity::Medium,
            AppError::ExternalServiceError(_) => ErrorSeverity::High,
            AppError::DatabaseError(_) => ErrorSeverity::High,
            AppError::ConfigError(_) => ErrorSeverity::High,
            AppError::IoError(_) => ErrorSeverity::High,
            AppError::InternalServerError(_) => ErrorSeverity::High,
        }
    }

    // Get the error type as a string
    pub fn error_type(&self) -> String {
        match self {
            AppError::ConfigError(_) => "config_error",
            AppError::ClientError(_) => "client_error",
            AppError::IoError(_) => "io_error",
            AppError::NotFound(_) => "not_found",
            AppError::BadRequest(_) => "bad_request",
            AppError::Unauthorized(_) => "unauthorized",
            AppError::Forbidden(_) => "forbidden",
            AppError::RateLimited(_) => "rate_limited",
            AppError::ExternalServiceError(_) => "external_service_error",
            AppError::DatabaseError(_) => "database_error",
            AppError::CacheError(_) => "cache_error",
            AppError::ValidationError(_) => "validation_error",
            AppError::InternalServerError(_) => "internal_error",
        }
        .to_string()
    }

    // Get HTTP status code for the error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::ClientError(_) => StatusCode::BAD_GATEWAY,
            AppError::ExternalServiceError(_) => StatusCode::BAD_GATEWAY,
            AppError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::CacheError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::InternalServerError(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }

    pub fn database_error(message: impl Into<String>) -> Self {
        Self::DatabaseError(message.into())
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }
}

// Implement conversion to HTTP response for AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_type = self.error_type();
        let error_message = self.to_string();
        let severity = self.severity();

        // Add detailed error info for internal errors if not in production
        let details = if status.is_server_error() && !cfg!(feature = "production") {
            Some(format!("{:?}", self))
        } else {
            None
        };

        // Increment error counter with metadata
        let _ = counter!(
            "api.errors",
            "status" => status.as_u16().to_string(),
            "type" => error_type.clone(),
            "severity" => severity.to_string()
        );

        // Log the error with appropriate level based on severity
        match severity {
            ErrorSeverity::Critical | ErrorSeverity::High => {
                error!(status = %status.as_u16(), error_type = %error_type, message = %error_message, "Error occurred");
            }
            _ => {
                warn!(status = %status.as_u16(), error_type = %error_type, message = %error_message, "Error occurred");
            }
        }

        // Return the HTTP response
        (
            status,
            Json(ErrorResponse {
                code: status.as_u16(),
                message: error_message,
                error_type,
                details,
            }),
        )
            .into_response()
    }
}

impl From<crate::app::services::error::ServiceError> for AppError {
    fn from(err: crate::app::services::error::ServiceError) -> Self {
        use crate::app::services::error::ServiceError;

        match err {
            ServiceError::PetNotFound => Self::NotFound("Pet not found".to_string()),
            ServiceError::UserNotFound => Self::NotFound("User not found".to_string()),
            ServiceError::UsernameExists => Self::BadRequest("Username already exists".to_string()),
            ServiceError::EmailExists => Self::BadRequest("Email already exists".to_string()),
            ServiceError::ValidationError(msg) => Self::ValidationError(msg),
            ServiceError::DatabaseError(msg) => Self::DatabaseError(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        // Test severity levels for different error types
        assert_eq!(
            AppError::NotFound("test".into()).severity(),
            ErrorSeverity::Low
        );
        assert_eq!(
            AppError::BadRequest("test".into()).severity(),
            ErrorSeverity::Low
        );
        assert_eq!(
            AppError::ValidationError("test".into()).severity(),
            ErrorSeverity::Low
        );

        assert_eq!(
            AppError::Unauthorized("test".into()).severity(),
            ErrorSeverity::Medium
        );
        assert_eq!(
            AppError::Forbidden("test".into()).severity(),
            ErrorSeverity::Medium
        );
        assert_eq!(
            AppError::RateLimited("test".into()).severity(),
            ErrorSeverity::Medium
        );
        assert_eq!(
            AppError::CacheError("test".into()).severity(),
            ErrorSeverity::Medium
        );

        assert_eq!(
            AppError::DatabaseError("test".into()).severity(),
            ErrorSeverity::High
        );
        assert_eq!(
            AppError::InternalServerError("test".into()).severity(),
            ErrorSeverity::High
        );
    }

    #[test]
    fn test_error_type_string() {
        // Test error type string representations
        assert_eq!(AppError::NotFound("test".into()).error_type(), "not_found");
        assert_eq!(
            AppError::BadRequest("test".into()).error_type(),
            "bad_request"
        );
        assert_eq!(
            AppError::Unauthorized("test".into()).error_type(),
            "unauthorized"
        );
        assert_eq!(AppError::Forbidden("test".into()).error_type(), "forbidden");
        assert_eq!(
            AppError::ValidationError("test".into()).error_type(),
            "validation_error"
        );
    }

    #[test]
    fn test_status_code_mapping() {
        // Test HTTP status code mappings
        assert_eq!(
            AppError::NotFound("test".into()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::BadRequest("test".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::Unauthorized("test".into()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::Forbidden("test".into()).status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            AppError::RateLimited("test".into()).status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            AppError::ValidationError("test".into()).status_code(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            AppError::InternalServerError("test".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_display() {
        // Test the Display implementation for errors
        let error = AppError::NotFound("test entity".into());
        assert_eq!(error.to_string(), "Not found: test entity");

        let error = AppError::BadRequest("invalid input".into());
        assert_eq!(error.to_string(), "Bad request: invalid input");
    }

    #[test]
    fn test_error_severity_display() {
        // Test the Display implementation for error severity
        assert_eq!(ErrorSeverity::Low.to_string(), "low");
        assert_eq!(ErrorSeverity::Medium.to_string(), "medium");
        assert_eq!(ErrorSeverity::High.to_string(), "high");
        assert_eq!(ErrorSeverity::Critical.to_string(), "critical");
    }
}
