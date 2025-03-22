//! User-defined services that extend the core functionality
//!
//! This module allows you to define custom services that build upon the core services.
//! Place your service implementations here following the established patterns.

use crate::core::services;

// Re-export core services for convenience
pub use services::*;

// Add your custom services below
// Example:
// pub mod user_service;
// pub mod notification_service;
