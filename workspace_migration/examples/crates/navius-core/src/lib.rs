//! # Navius Core
//!
//! This crate provides core functionality and abstractions for the Navius framework.
//!
//! Core features:
//! - Configuration management with YAML support
//! - Error handling with custom error types
//! - Dependency injection via component registry
//! - Common utilities for working with dates, strings, and IDs
//! - Type definitions and constants used throughout the framework

// Export core modules
pub mod config;
pub mod constants;
pub mod di;
pub mod error;
pub mod types;
pub mod util;

// Re-export common types
pub use config::Config;
pub use constants::*;
pub use di::Application;
pub use di::ApplicationBuilder;

// Re-export dependency injection traits and types for convenience
pub use di::{
    application::Environment,
    component::{
        AsyncLifecycle, ComponentRef, ComponentRegistry, ComponentScope, Lifecycle, LifecyclePhase,
    },
};

// Export error types
pub use error::{Error, Result};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Get the version information of the core library
pub fn version() -> String {
    format!("{} v{}", NAME, VERSION)
}

/// Initialize the core library
///
/// This function initializes the core functionality.
pub fn init() -> Result<Config> {
    let config = Config::default();
    Ok(config)
}

/// Initialize the core library with a custom configuration
///
/// This function initializes the core functionality with a custom configuration.
pub fn init_with_config(config: Config) -> Result<Config> {
    Ok(config)
}

/// Initialize a new application with dependency injection
///
/// This function creates a new application with the component registry.
pub fn init_application() -> ApplicationBuilder {
    ApplicationBuilder::new()
}

/// Initialize a new application with environment-specific configuration
///
/// This function creates a new application with the component registry and environment.
pub fn init_application_with_environment(env: Environment) -> ApplicationBuilder {
    ApplicationBuilder::new().with_environment(env)
}

// Add this block if the plugin module doesn't exist
pub mod plugin {
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    /// A registry for managing plugins
    pub struct PluginRegistry {
        plugins: RwLock<HashMap<String, Arc<dyn std::any::Any + Send + Sync>>>,
    }

    impl PluginRegistry {
        /// Create a new plugin registry
        pub fn new() -> Self {
            Self {
                plugins: RwLock::new(HashMap::new()),
            }
        }

        /// Register a plugin
        pub fn register<T: 'static + Send + Sync>(&self, name: &str, plugin: T) {
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(name.to_string(), Arc::new(plugin));
        }

        /// Get a plugin instance by name and type
        pub fn get<T: 'static + Send + Sync>(&self, name: &str) -> Option<Arc<T>> {
            let plugins = self.plugins.read().unwrap();
            plugins
                .get(name)
                .and_then(|plugin| plugin.clone().downcast::<T>().ok())
        }

        /// Check if a plugin exists
        pub fn contains(&self, name: &str) -> bool {
            let plugins = self.plugins.read().unwrap();
            plugins.contains_key(name)
        }

        /// Get all plugin names
        pub fn names(&self) -> Vec<String> {
            let plugins = self.plugins.read().unwrap();
            plugins.keys().cloned().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let config = init().unwrap();
        assert!(config.is_empty());
    }

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_init_application() {
        let app = init_application().build();
        assert_eq!(app.environment(), Environment::Development);
    }

    #[test]
    fn test_init_application_with_environment() {
        let app = init_application_with_environment(Environment::Production).build();
        assert_eq!(app.environment(), Environment::Production);
    }
}

// Include the error_test module for integration testing
#[cfg(test)]
mod error_test;
