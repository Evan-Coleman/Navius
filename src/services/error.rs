use crate::core::error::AppError;
use std::fmt;

/// Error type for service operations
#[derive(Debug)]
pub enum ServiceError {
    /// Repository error
    Repository(String),

    /// Validation error
    Validation(String),

    /// User not found
    UserNotFound,

    /// Username already exists
    UsernameExists,

    /// Email already exists
    EmailExists,

    /// Other generic error
    Other(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::Repository(msg) => write!(f, "Repository error: {}", msg),
            ServiceError::Validation(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::UserNotFound => write!(f, "User not found"),
            ServiceError::UsernameExists => write!(f, "Username already exists"),
            ServiceError::EmailExists => write!(f, "Email already exists"),
            ServiceError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl From<AppError> for ServiceError {
    fn from(error: AppError) -> Self {
        match error {
            AppError::NotFound(_msg) => ServiceError::UserNotFound,
            AppError::DatabaseError(msg) => ServiceError::Repository(msg),
            AppError::ValidationError(msg) => ServiceError::Validation(msg),
            _ => ServiceError::Other(error.to_string()),
        }
    }
}

/// Type alias for service results
pub type ServiceResult<T> = Result<T, ServiceError>;
