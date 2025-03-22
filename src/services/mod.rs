//! # Services module
//!
//! This module provides services that implement business logic.
//! Services use repositories to interact with data and implement business rules.

pub mod user;

#[cfg(test)]
mod tests;

pub use error::ServiceError;
pub use user::UserService;

mod error {
    //! Error types for services

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
}

/// Type alias for service results
pub type ServiceResult<T> = Result<T, ServiceError>;
