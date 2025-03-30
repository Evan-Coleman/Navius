//! Core types for authentication and authorization.
//!
//! This module defines the fundamental types used for authentication,
//! authorization, and identity management within the Navius framework.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Authentication configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AuthConfig {
    /// Enable authentication functionality.
    #[serde(default)]
    pub enabled: bool,
    /// Secret key for token signing and validation.
    #[serde(default)]
    pub secret_key: String,
    /// Token expiration time in seconds.
    #[serde(default = "default_token_expiry")]
    pub token_expiry: u64,
    /// Issuer claim for tokens.
    #[serde(default = "default_issuer")]
    pub issuer: String,
    /// Audience claim for tokens.
    #[serde(default = "default_audience")]
    pub audience: String,
    /// Enable debug mode (less secure but easier to troubleshoot).
    #[serde(default)]
    pub debug: bool,
    /// Configure OAuth providers.
    #[serde(default)]
    pub oauth: Option<OAuthConfig>,
    /// CORS settings for authentication endpoints.
    #[serde(default)]
    pub cors: Option<CorsConfig>,
}

fn default_token_expiry() -> u64 {
    3600 // 1 hour
}

fn default_issuer() -> String {
    "navius".to_string()
}

fn default_audience() -> String {
    "navius-app".to_string()
}

/// Claims used in authentication tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject identifier (usually user ID).
    pub sub: String,
    /// Issuer identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// Audience identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    /// Expiration time (as Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<u64>,
    /// Issued at time (as Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<u64>,
    /// Not before time (as Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<u64>,
    /// JWT ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    /// User roles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    /// User permissions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
    /// Additional custom claims.
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Represents a user's identity.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Identity {
    /// Unique identifier for the user.
    pub id: String,
    /// Username or login ID.
    pub username: String,
    /// User's email address.
    pub email: Option<String>,
    /// User's display name.
    pub display_name: Option<String>,
    /// User roles.
    pub roles: Vec<Role>,
    /// Direct permissions assigned to the user.
    pub permissions: Option<HashSet<Permission>>,
    /// Whether the identity is active.
    pub active: bool,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
    /// Provider-specific ID (for OAuth/external auth).
    pub provider_id: Option<String>,
    /// Authentication provider type.
    pub provider_type: Option<String>,
}

/// Authentication credentials for username/password authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Username or login ID.
    pub username: String,
    /// Password.
    pub password: String,
}

/// Permission for a resource.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Unique identifier.
    pub id: String,

    /// Resource name.
    pub resource: String,

    /// Action name.
    pub action: String,

    /// Optional conditions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<String>,
}

/// Authentication subject.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Subject {
    /// Unique identifier.
    pub id: String,

    /// Subject type (user, service, etc).
    pub subject_type: String,

    /// Display name.
    pub name: String,

    /// Subject roles.
    pub roles: Vec<Role>,

    /// Custom attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, String>>,
}

/// Represents a role that can be assigned to users.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Role {
    /// Unique identifier for the role.
    pub id: String,
    /// Role name (e.g., "admin", "user").
    pub name: String,
    /// Role description.
    pub description: Option<String>,
    /// Permissions granted by this role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
}

/// Configuration for OAuth providers.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OAuthConfig {
    /// OAuth providers configuration.
    pub providers: HashMap<String, OAuthProviderConfig>,
    /// Default provider.
    pub default_provider: Option<String>,
    /// Default callback URL.
    pub default_callback_url: Option<String>,
}

/// Configuration for a specific OAuth provider.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuthProviderConfig {
    /// Provider name/type (e.g., "github", "google").
    pub name: String,
    /// Client ID for this provider.
    pub client_id: String,
    /// Client secret for this provider.
    pub client_secret: String,
    /// Authorization endpoint URL.
    pub auth_url: String,
    /// Token endpoint URL.
    pub token_url: String,
    /// Redirect URL.
    pub redirect_url: String,
    /// Additional scopes to request.
    pub scopes: Vec<String>,
}

/// Configuration for CORS on auth endpoints.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CorsConfig {
    /// Allowed origins for CORS requests.
    pub allowed_origins: Vec<String>,
    /// Allowed methods for CORS requests.
    pub allowed_methods: Vec<String>,
    /// Allowed headers for CORS requests.
    pub allowed_headers: Vec<String>,
    /// Exposed headers for CORS requests.
    pub exposed_headers: Vec<String>,
    /// Whether to allow credentials in CORS requests.
    pub allow_credentials: bool,
    /// Max age for CORS requests.
    pub max_age: Option<u64>,
}
