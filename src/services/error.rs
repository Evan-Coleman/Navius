use std::fmt;

/// Service layer errors
#[derive(Debug)]
pub enum ServiceError {
    /// Database error
    DatabaseError(String),

    /// Validation error
    ValidationError(String),

    /// User not found
    UserNotFound,

    /// Username already exists
    UsernameExists,

    /// Email already exists
    EmailExists,

    /// Pet not found
    PetNotFound,

    /// Internal server error
    Internal(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::UserNotFound => write!(f, "User not found"),
            ServiceError::UsernameExists => write!(f, "Username already exists"),
            ServiceError::EmailExists => write!(f, "Email already exists"),
            ServiceError::PetNotFound => write!(f, "Pet not found"),
            ServiceError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}
