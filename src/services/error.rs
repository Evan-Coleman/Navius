use std::fmt;

/// Service layer errors
#[derive(Debug)]
pub enum ServiceError {
    /// Repository error
    Repository(String),
    /// Validation error
    Validation(String),
    /// Other error
    Other(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::Repository(msg) => write!(f, "Repository error: {}", msg),
            ServiceError::Validation(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}
