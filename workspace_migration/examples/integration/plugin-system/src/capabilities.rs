//! Plugin Capabilities
//!
//! This module defines capability traits that plugins can implement
//! to provide functionality to the application.

use async_trait::async_trait;
use navius_core::error::Result;
use std::collections::HashMap;

/// Logging capability provides logging functionality
pub trait LoggingCapability: Send + Sync {
    /// Log an informational message
    fn info(&self, message: &str);

    /// Log a warning message
    fn warn(&self, message: &str);

    /// Log an error message
    fn error(&self, message: &str);

    /// Log a debug message
    fn debug(&self, message: &str);

    /// Set the log level
    fn set_level(&self, level: &str) -> Result<()>;
}

/// Configuration capability provides access to application configuration
pub trait ConfigCapability: Send + Sync {
    /// Load configuration from sources
    fn load(&self) -> Result<()>;

    /// Get a string value from configuration
    fn get_string(&self, key: &str) -> Result<String>;

    /// Get an integer value from configuration
    fn get_int(&self, key: &str) -> Result<i64>;

    /// Get a boolean value from configuration
    fn get_bool(&self, key: &str) -> Result<bool>;

    /// Get a map of values from configuration
    fn get_map(&self, key: &str) -> Result<HashMap<String, String>>;

    /// Set a configuration value
    fn set(&self, key: &str, value: &str) -> Result<()>;
}

/// Storage capability provides data storage functionality
#[async_trait]
pub trait StorageCapability: Send + Sync {
    /// Initialize the storage system
    async fn initialize(&self) -> Result<()>;

    /// Shutdown the storage system
    async fn shutdown(&self) -> Result<()>;

    /// Store a value with the given key
    async fn store(&self, key: &str, value: &[u8]) -> Result<()>;

    /// Retrieve a value by key
    async fn retrieve(&self, key: &str) -> Result<Vec<u8>>;

    /// Delete a value by key
    async fn delete(&self, key: &str) -> Result<()>;

    /// List all keys with a given prefix
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
}

/// HTTP capability provides HTTP server functionality
#[async_trait]
pub trait HttpCapability: Send + Sync {
    /// Start the HTTP server
    async fn start(&self, port: u16) -> Result<()>;

    /// Stop the HTTP server
    async fn stop(&self) -> Result<()>;

    /// Register a handler for a given path
    fn register_handler(
        &self,
        path: &str,
        handler: Box<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>,
    ) -> Result<()>;
}

/// Routing capability provides HTTP routing functionality
pub trait RoutingCapability: Send + Sync {
    /// Register a GET route
    fn get(&self, path: &str, handler: Box<dyn Fn() -> String + Send + Sync>) -> Result<()>;

    /// Register a POST route
    fn post(&self, path: &str, handler: Box<dyn Fn(&[u8]) -> String + Send + Sync>) -> Result<()>;

    /// Register a PUT route
    fn put(&self, path: &str, handler: Box<dyn Fn(&[u8]) -> String + Send + Sync>) -> Result<()>;

    /// Register a DELETE route
    fn delete(&self, path: &str, handler: Box<dyn Fn() -> String + Send + Sync>) -> Result<()>;
}

/// Health check capability provides health check functionality
pub trait HealthCheckCapability: Send + Sync {
    /// Register a health check
    fn register_check(&self, name: &str, check: Box<dyn Fn() -> bool + Send + Sync>) -> Result<()>;

    /// Run all health checks
    fn check_all(&self) -> HashMap<String, bool>;

    /// Run a specific health check
    fn check(&self, name: &str) -> Result<bool>;
}
