//! User-defined repositories that extend the core data access layer
//!
//! This module allows you to define custom repositories for your models
//! that build upon the core repository patterns. Follow the established
//! patterns and error handling guidelines when creating new repositories.

use crate::core::{database::PgPool, error::Result, repository};

// Re-export core repositories for convenience
pub use repository::*;

// Add your custom repositories below
// Example:
// pub mod user_repository;
// pub mod profile_repository;
//
// Remember to:
// 1. Use the provided PgPool for database access
// 2. Implement proper error handling using AppError
// 3. Follow the repository pattern conventions
// 4. Add unit tests for your repositories
// 5. Use transactions where appropriate
