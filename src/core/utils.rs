//! User-defined utility functions and helpers
//!
//! This module allows you to define custom utility functions that build
//! upon the core utilities. Follow the established patterns and error
//! handling guidelines when creating new utilities.

use crate::core::error::Result;

// User-extensible modules
pub mod api_logger;
pub mod api_resource;
pub mod request_id;

// Export specific items
pub use api_logger::{RequestLogger, log_request, log_response};
pub use api_resource::{ApiHandlerOptions, ApiResource, ApiResourceRegistry, create_api_handler};
pub use request_id::get_req_id;

// Add your custom utilities below
// Example:
// pub mod string_utils;
// pub mod validation_utils;
//
// Remember to:
// 1. Keep functions focused and single-purpose
// 2. Use proper error handling with AppError
// 3. Add comprehensive documentation
// 4. Include unit tests
// 5. Avoid duplicating core functionality
