//! User-defined models that extend the core data structures
//!
//! This module allows you to define custom models and data structures
//! that build upon the core models. Follow the established patterns
//! and error handling guidelines when creating new models.

use crate::core::error::Result;
use crate::core::models;

// Re-export core models for convenience
pub use models::*;

// Add your custom models below
// Example:
// pub mod user;
// pub mod profile;
//
// Remember to:
// 1. Implement proper error handling using AppError
// 2. Add validation methods where appropriate
// 3. Implement necessary database traits
// 4. Add unit tests for your models
