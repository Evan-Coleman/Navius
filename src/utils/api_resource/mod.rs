//! # API Resource Abstraction
//!
//! This module provides a high-level abstraction for API resources that
//! handles common concerns like caching, retries, and error handling.
//!
//! See the [README.md](./README.md) for detailed usage examples and guidelines.

mod core;
#[cfg(test)]
mod tests;

// Re-export public items
pub use core::{ApiHandlerOptions, ApiResource, create_api_handler};
