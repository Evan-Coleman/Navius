//! # Logger Module
//!
//! Generic logging interface and implementations for the Navius application.
//!
//! This module provides a flexible logging system with pluggable providers that can be configured
//! at runtime. The core interface is defined by the `LoggingOperations` trait, which abstracts away
//! the details of how logs are actually processed and stored.
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```
//! use navius::core::logger::{init, LogInfo, LogLevel};
//! use navius::core::logger::config::LoggingConfig;
//!
//! async fn example() {
//!     // Initialize the logging system with default configuration
//!     let config = LoggingConfig::default();
//!     let logger = init(&config).await.unwrap();
//!     
//!     // Log a simple message
//!     logger.info(LogInfo::new("Hello, world!")).unwrap();
//!     
//!     // Log with additional context
//!     logger.warn(
//!         LogInfo::new("Warning: resource not found")
//!             .with_context("user-service")
//!             .with_request_id("req-123")
//!     ).unwrap();
//! }
//! ```
//!
//! ### Creating a Custom Logger
//!
//! ```
//! use std::sync::Arc;
//! use async_trait::async_trait;
//! use navius::core::logger::{
//!     LoggingProvider, LoggingOperations, LogInfo, LogLevel,
//!     LoggingError, LoggingConfig, StructuredLog
//! };
//!
//! // Define a custom logger implementation
//! struct MyCustomLogger;
//!
//! impl LoggingOperations for MyCustomLogger {
//!     // Implement required methods...
//!     // ...
//! }
//!
//! // Define a provider for the custom logger
//! struct MyCustomLoggerProvider;
//!
//! #[async_trait]
//! impl LoggingProvider for MyCustomLoggerProvider {
//!     async fn create_logger(&self, config: &LoggingConfig) -> Result<Arc<dyn LoggingOperations>, LoggingError> {
//!         Ok(Arc::new(MyCustomLogger))
//!     }
//!     
//!     fn name(&self) -> &'static str {
//!         "custom"
//!     }
//!     
//!     fn supports(&self, config: &LoggingConfig) -> bool {
//!         config.logger_type == "custom"
//!     }
//! }
//! ```

pub mod config;
pub mod console;
pub mod error;
pub mod interface;
pub mod provider;
#[cfg(test)]
mod tests;
pub mod tracing;

// Re-export main types and traits
pub use config::LoggingConfig;
pub use console::ConsoleLoggerProvider;
pub use error::LoggingError;
pub use interface::{LogInfo, LogLevel, LoggingOperations, StructuredLog};
pub use provider::{LoggingProvider, LoggingProviderRegistry};
pub use tracing::TracingLoggerProvider;

use std::sync::Arc;

/// Initialize the logging system with all available providers
pub async fn init(config: &LoggingConfig) -> Result<Arc<dyn LoggingOperations>, LoggingError> {
    // Create the provider registry
    let registry = Arc::new(LoggingProviderRegistry::new());

    // Register all available providers
    registry.register_provider(Arc::new(TracingLoggerProvider::new()))?;
    registry.register_provider(Arc::new(ConsoleLoggerProvider::new()))?;

    // Set the default provider based on configuration
    registry.set_default_provider("tracing")?;

    // Create a logger from the configuration
    registry.create_logger_from_config(config).await
}

/// Create a global logger instance with default configuration
pub async fn create_default_logger() -> Result<Arc<dyn LoggingOperations>, LoggingError> {
    let config = LoggingConfig::default();
    init(&config).await
}
