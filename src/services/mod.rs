//! # Services module
//!
//! This module provides services that implement business logic.
//! Services use repositories to interact with data and implement business rules.

#[cfg(test)]
use mock_it::Mock;

pub mod error;
pub mod user;
pub use error::ServiceError;
pub use user::{IUserService, UserService};

#[cfg(test)]
mod tests;

#[cfg(test)]
pub type MockUserService = Mock<dyn IUserService, ()>;

/// Type alias for service results
pub type ServiceResult<T> = Result<T, ServiceError>;
