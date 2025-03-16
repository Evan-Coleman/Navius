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
use utoipa::ToSchema;

pub type Result<T> = result::Result<T, AppError>;

// The struct to be returned from the API in case of an error
#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
#[derive(Error, Debug)]
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
    InternalError(String),
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
            AppError::InternalError(_) => ErrorSeverity::High,
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
            AppError::InternalError(_) => "internal_error",
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
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
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
