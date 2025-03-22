//! User-defined API endpoints and handlers
//!
//! This module allows you to define custom API endpoints and handlers
//! that build upon the core API functionality. Follow the established
//! patterns and error handling guidelines when creating new endpoints.

use crate::core::{api, error::Result};

// Re-export core API components for convenience
pub use api::*;

// Add your custom API endpoints below
// Example:
// pub mod user_api;
// pub mod profile_api;
//
// Remember to:
// 1. Use proper error handling with AppError
// 2. Follow REST conventions
// 3. Document your endpoints with OpenAPI annotations
// 4. Add appropriate validation
// 5. Include unit and integration tests
// 6. Use the ApiContext for request handling
