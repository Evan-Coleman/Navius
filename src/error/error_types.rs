//! # Error Types
//!
//! This module allows users to extend the core error types with additional functionality.

// Re-export everything from core
pub use crate::core::error::error_types::*;

// Add custom error types and extensions below:

// Example:
// ```
// impl AppError {
//     pub fn rate_limited(message: impl Into<String>) -> Self {
//         AppError::RateLimited(message.into())
//     }
// }
// ```
