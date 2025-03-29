//! Authentication and authorization framework for Navius applications.
//!
//! This crate provides a flexible, modular authentication system for Navius applications.
//! It supports various authentication methods, including basic authentication, JWT tokens,
//! and OAuth, with a pluggable provider architecture for easy extension.

// Authentication modules
pub mod authorize;
pub mod error;
pub mod providers;
pub mod types;

// Conditionally compiled modules
#[cfg(feature = "basic")]
pub mod basic;
#[cfg(feature = "http")]
pub mod middleware;
#[cfg(feature = "jwt")]
pub mod token;

// Re-exports
pub use error::{Error, Result};
pub use providers::{AuthProvider, ProviderType};
pub use types::{AuthConfig, Claims, Credentials, Identity, Permission, Role, Subject};

#[cfg(feature = "basic")]
pub use basic::BasicProvider;
#[cfg(feature = "http")]
pub use middleware::{extract_token, AuthChecker, AuthLayer};
#[cfg(feature = "jwt")]
pub use token::{JWTProvider, TokenProviderConfig};
#[cfg(feature = "oauth")]
pub use types::{OAuthConfig, OAuthProviderConfig};

/// Version information for the crate
pub struct Version;

impl Version {
    /// Get the current version of the crate
    pub fn current() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Get a semver-compatible version string
    pub fn semver() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

/// Initialize the authentication system
pub fn init() {
    use tracing::info;
    info!("Initializing Navius Auth v{}", Version::current());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!Version::current().is_empty());
        assert!(!Version::semver().is_empty());
    }
}
