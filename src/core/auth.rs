//! Authentication core functionality
//!
//! This module provides authentication and authorization functionality:
//! - Middleware for validating incoming bearer tokens (protect our API)
//! - Client for acquiring tokens for downstream API calls

#[cfg(feature = "auth")]
/// Authentication module for Navius
pub mod claims;
#[cfg(feature = "auth")]
pub mod client;
#[cfg(feature = "auth")]
pub mod error;
#[cfg(feature = "auth")]
pub mod interfaces;
#[cfg(feature = "auth")]
pub mod middleware;
#[cfg(feature = "auth")]
pub mod mock;
#[cfg(feature = "auth")]
pub mod models;
#[cfg(feature = "auth")]
pub mod providers;

#[cfg(feature = "auth")]
use serde::Deserialize;
#[cfg(feature = "auth")]
use std::collections::HashMap;

// Re-export commonly used items
#[cfg(feature = "auth")]
pub use self::{
    claims::StandardClaims,
    client::EntraTokenClient,
    error::AuthError,
    interfaces::{TokenClient as InterfaceTokenClient, TokenValidationResult},
    middleware::{
        AuthMiddleware, EntraAuthConfig, EntraAuthLayer, Permission, PermissionRequirement, Role,
        RoleRequirement, auth_middleware, require_auth, require_roles, role_from_string,
    },
    mock::MockTokenClient,
};

/// Core authentication configuration
#[cfg(feature = "auth")]
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// Default provider name
    pub default_provider: String,
    /// Map of configured providers
    pub providers: HashMap<String, ProviderConfig>,
}

/// Individual provider configuration
#[cfg(feature = "auth")]
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    /// OAuth2 client ID
    pub client_id: String,
    /// JWKS endpoint URL
    pub jwks_uri: String,
    /// Token issuer URL
    pub issuer_url: String,
    /// Expected audience value
    pub audience: String,
    /// Role mappings
    #[serde(default)]
    pub role_mappings: RoleMappings,
    /// Validation leeway in seconds
    #[serde(default = "default_leeway")]
    pub leeway: u64,
}

#[cfg(feature = "auth")]
fn default_leeway() -> u64 {
    30
}

/// Role mapping configuration
#[cfg(feature = "auth")]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RoleMappings {
    #[serde(default)]
    pub admin: Vec<String>,
    #[serde(default)]
    pub read_only: Vec<String>,
    #[serde(default)]
    pub full_access: Vec<String>,
}

/// TokenClient trait for authentication
#[cfg(feature = "auth")]
pub trait TokenClient: Send + Sync + std::fmt::Debug {
    /// Get a token for the specified resource
    fn get_token(&self, resource: &str) -> Result<String, error::AuthError>;
}
