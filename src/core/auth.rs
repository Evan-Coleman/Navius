//! Authentication core functionality
//!
//! This module provides authentication and authorization functionality:
//! - Middleware for validating incoming bearer tokens (protect our API)
//! - Client for acquiring tokens for downstream API calls

/// Authentication module for Navius
pub mod claims;
pub mod client;
pub mod error;
pub mod interfaces;
pub mod middleware;
pub mod mock;
pub mod models;
pub mod providers;

use serde::Deserialize;
use std::collections::HashMap;

// Re-export commonly used items
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
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// Default provider name
    pub default_provider: String,
    /// Map of configured providers
    pub providers: HashMap<String, ProviderConfig>,
}

/// Individual provider configuration
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

fn default_leeway() -> u64 {
    30
}

/// Role mapping configuration
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
pub trait TokenClient: Send + Sync + std::fmt::Debug {
    /// Get a token for the specified resource
    fn get_token(&self, resource: &str) -> Result<String, error::AuthError>;
}
