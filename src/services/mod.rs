//! Services module for business logic
//!
//! This module provides services that encapsulate business logic.
//! Services use repositories to interact with data and implement business rules.

mod user;

#[cfg(test)]
mod tests;

pub use user::UserService;

use std::sync::Arc;

/// Service error types
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// Repository error
    #[error("Repository error: {0}")]
    Repository(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// User not found
    #[error("User not found")]
    UserNotFound,

    /// Username already exists
    #[error("Username already exists")]
    UsernameExists,

    /// Email already exists
    #[error("Email already exists")]
    EmailExists,
}

/// Result type for service operations
pub type ServiceResult<T> = Result<T, ServiceError>;
