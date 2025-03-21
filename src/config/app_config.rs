//! # Application Configuration
//!
//! This module re-exports all configuration types and functions from the core implementation.
//! Users can extend this file to add custom configuration settings.

// Re-export everything from the core implementation
pub use crate::core::config::app_config::*;

// Users can add custom configuration extensions here:
//
// Example:
// ```
// /// Custom application-specific configuration
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct MyCustomConfig {
//     pub feature_enabled: bool,
//     pub api_key: String,
// }
//
// impl Default for MyCustomConfig {
//     fn default() -> Self {
//         Self {
//             feature_enabled: false,
//             api_key: String::new(),
//         }
//     }
// }
// ```
