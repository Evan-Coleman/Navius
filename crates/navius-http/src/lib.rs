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
#[cfg(feature = "server")]
mod web_plugin;

// Re-export error types
pub use error::{Error, Result};

// Re-export server types when the "server" feature is enabled
#[cfg(feature = "server")]
pub use server::{HttpServerConfig, ShutdownReceiver, ShutdownSender};

// Re-export route discovery types for automatic route discovery
#[cfg(feature = "server")]
pub use server::route_discovery::{RouteDiscoveryConfig, RouteRegistry};

// Re-export the WebPlugin for the Zero Boilerplate Initiative
#[cfg(feature = "server")]
pub use web_plugin::{AppState, WebPlugin};

// Re-export client types when the "client" feature is enabled
// #[cfg(feature = "client")]
// pub use client::{HttpClient}; // Only export HttpClient if it exists

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
    use navius_test::error::{TestResult, assert_contains, assert_true};

    #[test]
    fn test_version() -> TestResult<()> {
        let version = Version::current();
        assert_true(!version.is_empty(), "Version should not be empty")?;

        let semver = Version::semver();
        assert_contains(semver, "v", "Semver version should start with 'v'")?;

        Ok(())
    }

    #[test]
    fn test_init() -> TestResult<()> {
        let result = init();
        assert_true(result.is_ok(), "Init function should succeed")?;

        Ok(())
    }
}

// Optional: Re-export the prelude for convenience
pub mod prelude {
    #[cfg(feature = "server")]
    pub use crate::server::prelude::*; // Assuming server prelude exports axum types

    // Export the WebPlugin for easy access
    #[cfg(feature = "server")]
    pub use crate::web_plugin::WebPlugin;

    // #[cfg(feature = "client")]
    // pub use crate::client::prelude::*; // Comment out client prelude
}
