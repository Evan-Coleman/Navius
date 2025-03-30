//! Authentication middleware for Navius Auth.
//!
//! This module provides middleware for integrating authentication
//! with HTTP servers and request handling.

#[cfg(feature = "http")]
use crate::error::Error;
#[cfg(feature = "http")]
use crate::providers::AuthProvider;
#[cfg(feature = "http")]
use crate::types::{Identity, Subject};
#[cfg(feature = "http")]
use std::sync::Arc;
#[cfg(feature = "http")]
use tracing::{debug, error};

/// Authentication layer for HTTP requests.
#[cfg(feature = "http")]
#[derive(Clone)]
pub struct AuthLayer {
    provider: Arc<dyn AuthProvider>,
    config: AuthConfig,
}

/// Configuration for authentication middleware.
#[cfg(feature = "http")]
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Whether authentication is required.
    pub required: bool,
    /// Bearer token header name (default: "Authorization").
    pub header_name: String,
    /// Bearer token prefix (default: "Bearer").
    pub token_prefix: String,
    /// Whether to include the identity in the request extension.
    pub include_identity: bool,
    /// List of paths that do not require authentication.
    pub exempt_paths: Vec<String>,
}

#[cfg(feature = "http")]
impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            required: true,
            header_name: "Authorization".to_string(),
            token_prefix: "Bearer".to_string(),
            include_identity: true,
            exempt_paths: vec![],
        }
    }
}

#[cfg(feature = "http")]
impl AuthLayer {
    /// Create a new authentication layer.
    pub fn new(provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            provider,
            config: AuthConfig::default(),
        }
    }

    /// Create a new authentication layer with custom configuration.
    pub fn with_config(provider: Arc<dyn AuthProvider>, config: AuthConfig) -> Self {
        Self { provider, config }
    }

    /// Create a permissive authentication layer that doesn't require authentication.
    pub fn optional(provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            provider,
            config: AuthConfig {
                required: false,
                ..AuthConfig::default()
            },
        }
    }
}

/// Extract the token from the request header.
#[cfg(feature = "http")]
pub fn extract_token<T>(headers: &http::HeaderMap, config: &AuthConfig) -> Option<String> {
    headers
        .get(&config.header_name)
        .and_then(|header| header.to_str().ok())
        .and_then(|header_str| {
            if header_str.starts_with(&config.token_prefix) {
                Some(
                    header_str
                        .trim_start_matches(&config.token_prefix)
                        .trim_start()
                        .to_string(),
                )
            } else {
                None
            }
        })
}

/// Authorization checker for verifying permissions based on roles.
#[cfg(feature = "http")]
pub struct AuthChecker {
    /// Authentication provider.
    provider: Arc<dyn AuthProvider>,
    /// Roles required for authorization.
    required_roles: Vec<String>,
    /// Whether all roles are required (AND) or any role is sufficient (OR).
    require_all_roles: bool,
}

#[cfg(feature = "http")]
impl AuthChecker {
    /// Create a new authorization checker.
    pub fn new(provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            provider,
            required_roles: vec![],
            require_all_roles: false,
        }
    }

    /// Set the roles required for authorization.
    pub fn require_roles(mut self, roles: Vec<String>) -> Self {
        self.required_roles = roles;
        self
    }

    /// Set whether all roles are required (true) or any role is sufficient (false).
    pub fn require_all_roles(mut self, require_all: bool) -> Self {
        self.require_all_roles = require_all;
        self
    }

    /// Check if a subject has the required roles.
    pub fn check_roles(&self, subject: &Subject) -> bool {
        if self.required_roles.is_empty() {
            return true;
        }

        if self.require_all_roles {
            // All roles are required
            self.required_roles.iter().all(|role| {
                subject
                    .roles
                    .iter()
                    .any(|subject_role| subject_role.name == *role)
            })
        } else {
            // Any role is sufficient
            self.required_roles.iter().any(|role| {
                subject
                    .roles
                    .iter()
                    .any(|subject_role| subject_role.name == *role)
            })
        }
    }

    /// Check if a subject is authorized.
    pub async fn check_authorization(&self, token: &str) -> Result<Subject, Error> {
        let subject = self.provider.validate_token(token).await?;

        if !self.check_roles(&subject) {
            return Err(Error::authorization_failed("Insufficient permissions"));
        }

        Ok(subject)
    }
}
