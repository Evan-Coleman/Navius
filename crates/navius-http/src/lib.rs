//! Navius HTTP crate provides HTTP server and client functionality.
//!
//! This crate provides a modular HTTP server built on top of axum, as well as
//! an HTTP client built on top of reqwest. It includes middleware for common
//! HTTP server functionality like CORS, request ID generation, logging, and timeout.

mod error;
mod util;

// Conditionally compile modules based on features
#[cfg(feature = "client")]
pub mod client;
pub mod middleware;
#[cfg(feature = "server")]
pub mod server;

// Re-export error types
pub use error::{Error, Result};

// Re-export server types when the "server" feature is enabled
#[cfg(feature = "server")]
pub use server::{HttpServer, HttpServerHandle};

// Re-export client types when the "client" feature is enabled
#[cfg(feature = "client")]
pub use client::HttpClient;

// Re-export middleware
pub use middleware::{
    CorsConfig,
    CorsLayer,

    LoggingConfig,
    LoggingLayer,

    RequestIdLayer,

    TimeoutConfig,
    TimeoutLayer,

    // CORS middleware
    cors_layer,
    // Default middleware
    default_middleware,
    detailed_logging_layer,
    // Logging middleware
    logging_layer,
    permissive_cors_layer,
    // RequestId middleware
    request_id_layer,
    // Timeout middleware
    timeout_layer,
    timeout_layer_with_duration,
    with_timeout,
};

/// Version information for the Navius HTTP crate.
#[derive(Debug, Clone, Copy)]
pub struct Version;

impl Version {
    /// Get the current version of the crate.
    pub fn current() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Get a semver compatible version string.
    pub fn semver() -> String {
        format!("v{}", Self::current())
    }
}

/// Initialize HTTP functionality.
pub fn init() {
    tracing::info!(
        target: "navius::http",
        version = Version::current(),
        "Initializing Navius HTTP"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!Version::current().is_empty());
        assert!(Version::semver().starts_with('v'));
    }

    #[test]
    fn test_init() {
        let result = init();
        assert!(result.is_ok());
    }
}
