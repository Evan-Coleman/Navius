//! HTTP server and client functionality for the Navius framework.
//!
//! This crate provides HTTP server and client functionality for the Navius framework,
//! built on top of Axum and Reqwest.

// Modules
#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;

pub mod error;
pub mod middleware;
pub mod util;

// Re-exports
pub use error::{Error, Result};

#[cfg(feature = "server")]
pub use server::{HttpServer, HttpServerHandle};

#[cfg(feature = "client")]
pub use client::HttpClient;

/// Navius HTTP version information
pub struct Version;

impl Version {
    /// Get the current version of Navius HTTP
    pub fn current() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Get the semver-compatible version string
    pub fn semver() -> String {
        format!("v{}", Self::current())
    }
}

/// Initialize HTTP functionality
pub fn init() -> Result<()> {
    tracing::info!("Initializing Navius HTTP v{}", Version::current());
    Ok(())
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
