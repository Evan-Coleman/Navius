use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Pet not found")]
    PetNotFound,
    #[error("User not found")]
    UserNotFound,
    #[error("Username already exists")]
    UsernameExists,
    #[error("Email already exists")]
    EmailExists,
    #[error("Validation error: {0}")]
    ValidationError(String),
    // Alias for ValidationError for compatibility
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}
