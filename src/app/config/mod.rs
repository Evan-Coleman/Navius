// Copyright (c) 2025 Navius Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Application Configuration
//!
//! This module provides access to application configuration settings.
//! It serves as a user-friendly wrapper around the core configuration system.

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
/// use crate::app::config::get_config;
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

// Add your custom configuration modules and functions below
// Example:
// pub mod app_settings;
// pub mod feature_flags;
