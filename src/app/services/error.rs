use std::fmt;

use crate::core::error::error_types::AppError;

/// Service-level errors
#[derive(Debug)]
pub enum ServiceError {
    /// Database or repository errors
    Repository(String),
    /// Input validation errors
    Validation(String),
    /// Resource not found errors
    NotFound(String),
    /// Resource conflict errors
    Conflict(String),
    /// Other service errors
    Other(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::Repository(msg) => write!(f, "Repository error: {}", msg),
            ServiceError::Validation(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServiceError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            ServiceError::Other(msg) => write!(f, "Service error: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<AppError> for ServiceError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::DatabaseError(msg) => ServiceError::Repository(msg),
            AppError::ValidationError(msg) => ServiceError::Validation(msg),
            AppError::NotFoundError(msg) => ServiceError::NotFound(msg),
            AppError::ConflictError(msg) => ServiceError::Conflict(msg),
            AppError::AuthenticationError(msg) => {
                ServiceError::Other(format!("Authentication error: {}", msg))
            }
            AppError::AuthorizationError(msg) => {
                ServiceError::Other(format!("Authorization error: {}", msg))
            }
            AppError::BadRequest(msg) => ServiceError::Validation(msg),
            AppError::InternalServerError(msg) => ServiceError::Other(msg),
            AppError::ConfigurationError(msg) => {
                ServiceError::Other(format!("Configuration error: {}", msg))
            }
            AppError::CacheError(msg) => ServiceError::Other(format!("Cache error: {}", msg)),
            AppError::NetworkError(msg) => ServiceError::Other(format!("Network error: {}", msg)),
            AppError::ExternalServiceError(msg) => {
                ServiceError::Other(format!("External service error: {}", msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_error_display() {
        let err = ServiceError::NotFound("User not found".to_string());
        assert_eq!(err.to_string(), "Not found: User not found");
    }

    #[test]
    fn test_service_error_to_app_error() {
        let service_err = ServiceError::NotFound("Resource not found".to_string());
        let app_err: AppError = service_err.into();
        match app_err {
            AppError::NotFoundError(msg) => assert_eq!(msg, "Resource not found"),
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_app_error_to_service_error() {
        let app_err = AppError::ValidationError("Invalid input".to_string());
        let service_err: ServiceError = app_err.into();
        match service_err {
            ServiceError::Validation(msg) => assert_eq!(msg, "Invalid input"),
            _ => panic!("Wrong error variant"),
        }
    }
}
