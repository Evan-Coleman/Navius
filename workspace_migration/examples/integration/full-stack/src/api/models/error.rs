use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use navius_core::error::Error;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error code enum for API errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// Bad request - client error
    BadRequest,
    /// Unauthorized - authentication required
    Unauthorized,
    /// Forbidden - insufficient permissions
    Forbidden,
    /// Not found - resource not found
    NotFound,
    /// Conflict - resource already exists or state conflict
    Conflict,
    /// Validation error - invalid input
    ValidationError,
    /// Internal server error
    InternalServerError,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::BadRequest => write!(f, "BAD_REQUEST"),
            ErrorCode::Unauthorized => write!(f, "UNAUTHORIZED"),
            ErrorCode::Forbidden => write!(f, "FORBIDDEN"),
            ErrorCode::NotFound => write!(f, "NOT_FOUND"),
            ErrorCode::Conflict => write!(f, "CONFLICT"),
            ErrorCode::ValidationError => write!(f, "VALIDATION_ERROR"),
            ErrorCode::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
        }
    }
}

/// Additional error details for validation errors
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorDetails {
    /// Field-specific validation errors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub field_errors: Vec<FieldError>,
    /// Additional contextual information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

/// Field-specific validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    /// Field name
    pub field: String,
    /// Error message
    pub message: String,
}

/// Standard error response for API errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: ErrorCode,
    /// Error message
    pub message: String,
    /// Error details (for validation errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ErrorDetails>,
    /// Request ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
            request_id: None,
        }
    }

    /// Create a new error response with details
    pub fn with_details(code: ErrorCode, message: String, details: ErrorDetails) -> Self {
        Self {
            code,
            message,
            details: Some(details),
            request_id: None,
        }
    }

    /// Add a request ID to the error response
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Convert Navius Error to ErrorResponse
    pub fn from_error(error: &Error) -> Self {
        match error.kind() {
            navius_core::error::ErrorKind::NotFound => {
                Self::new(ErrorCode::NotFound, error.to_string())
            }
            navius_core::error::ErrorKind::ValidationError => {
                Self::new(ErrorCode::ValidationError, error.to_string())
            }
            navius_core::error::ErrorKind::Unauthorized => {
                Self::new(ErrorCode::Unauthorized, error.to_string())
            }
            navius_core::error::ErrorKind::Forbidden => {
                Self::new(ErrorCode::Forbidden, error.to_string())
            }
            navius_core::error::ErrorKind::Conflict => {
                Self::new(ErrorCode::Conflict, error.to_string())
            }
            navius_core::error::ErrorKind::BadRequest => {
                Self::new(ErrorCode::BadRequest, error.to_string())
            }
            _ => Self::new(ErrorCode::InternalServerError, error.to_string()),
        }
    }
}

/// Implementation of IntoResponse for Error
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self.kind() {
            navius_core::error::ErrorKind::NotFound => StatusCode::NOT_FOUND,
            navius_core::error::ErrorKind::ValidationError => StatusCode::UNPROCESSABLE_ENTITY,
            navius_core::error::ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
            navius_core::error::ErrorKind::Forbidden => StatusCode::FORBIDDEN,
            navius_core::error::ErrorKind::Conflict => StatusCode::CONFLICT,
            navius_core::error::ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error_response = ErrorResponse::from_error(&self);
        (status, Json(error_response)).into_response()
    }
}
