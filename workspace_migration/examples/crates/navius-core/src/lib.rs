//! Core functionality for the Navius framework.
//!
//! This crate provides essential types, utilities, and functionality used by all other Navius crates.
//! It includes error handling, configuration management, common constants, and utility functions.

pub mod config;
pub mod constants;
pub mod error;
pub mod types;
pub mod util;

// Re-export commonly used types for convenience
pub use config::Config;
pub use error::{Error, Result};
pub use types::*;

/// Navius version information
pub struct Version;

impl Version {
    /// Get the current version of Navius
    pub fn current() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Get the semver-compatible version string
    pub fn semver() -> String {
        format!("v{}", Self::current())
    }
}

/// Initialize core functionality
pub fn init() -> Result<()> {
    tracing::info!("Initializing Navius Core v{}", Version::current());
    Ok(())
}

/// Initialize with custom configuration
pub fn init_with_config(config: Config) -> Result<()> {
    tracing::info!(
        "Initializing Navius Core v{} with custom config",
        Version::current()
    );
    tracing::debug!("Configuration: {:?}", config);
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
