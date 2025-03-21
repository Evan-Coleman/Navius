//! # Utility Functions
//!
//! This module provides user-extensible utility functions and helpers.
//! The core implementations are in the core/utils module.

// Re-export core functionality
pub use crate::core::utils::*;

// User-extensible modules
pub mod api_resource;

// Add your custom utility functions below
//
// Example:
//
// /// Format a date string in ISO 8601 format
// pub fn format_iso_date(date: &chrono::DateTime<chrono::Utc>) -> String {
//     date.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
// }
