//! # Application Configuration
//!
//! This module provides access to application configuration settings.
//! It serves as a user-friendly wrapper around the core configuration system.

pub mod app_config;
pub mod constants;
#[cfg(test)]
mod tests;

// Re-export key components from core config
pub use crate::core::config::app_config::{
    AppConfig, AuthConfig, CacheConfig, EnvironmentType, LoggingConfig, ReliabilityConfig,
    ServerConfig, load_config,
};

use lazy_static::lazy_static;
use std::sync::Arc;

lazy_static! {
    pub static ref CONFIG: Arc<AppConfig> =
        Arc::new(load_config().expect("Failed to load configuration"));
}

/// Shortcut function to get the application configuration
///
/// # Returns
///
/// The application configuration object, or panics if configuration cannot be loaded
///
/// # Example
///
/// ```
/// use rust_backend::config::get_config;
///
/// let config = get_config();
/// println!("Server address: {}", config.server_addr());
/// ```
pub fn get_config() -> AppConfig {
    load_config().expect("Failed to load application configuration")
}

/// Get the cache configuration
pub fn get_cache_config() -> CacheConfig {
    get_config().cache
}

/// Get the server configuration
pub fn get_server_config() -> ServerConfig {
    get_config().server
}
