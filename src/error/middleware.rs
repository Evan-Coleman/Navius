//! # Error Middleware
//!
//! This module allows users to extend the core error middleware functionality.

// Re-export everything from core
pub use crate::core::error::middleware::*;

// Add custom middleware extensions below:

// Example:
// ```
// pub fn custom_error_handler<B>(error: BoxError) -> Response
// where
//     B: HttpBody + Send + 'static,
// {
//     // Custom error handling logic
// }
// ```
