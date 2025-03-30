//! Authentication providers for Navius Auth.
//!
//! This module defines the provider interface and implementations for different
//! authentication methods such as JWT, OAuth, and basic authentication.

use crate::error::Result;
use crate::types::{Credentials, Identity, Subject};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Authentication provider type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    /// Basic username/password authentication.
    Basic,
    /// JSON Web Token authentication.
    JWT,
    /// OAuth 2.0 authentication.
    #[cfg(feature = "oauth")]
    OAuth,
    /// Custom authentication provider.
    Custom,
}

impl fmt::Display for ProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderType::Basic => write!(f, "basic"),
            ProviderType::JWT => write!(f, "jwt"),
            #[cfg(feature = "oauth")]
            ProviderType::OAuth => write!(f, "oauth"),
            ProviderType::Custom => write!(f, "custom"),
        }
    }
}

/// Authentication provider configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    /// Provider type.
    pub provider_type: ProviderType,
    /// Provider name.
    pub name: String,
    /// Provider-specific configuration.
    #[serde(flatten)]
    pub config: serde_json::Value,
}

/// Authentication provider interface.
#[async_trait]
pub trait AuthProvider: Send + Sync + 'static {
    /// Get the provider type.
    fn provider_type(&self) -> ProviderType;

    /// Get the provider name.
    fn name(&self) -> &str;

    /// Authenticate a user using credentials.
    async fn authenticate(&self, credentials: &Credentials) -> Result<Identity>;

    /// Validate an authentication token.
    async fn validate_token(&self, token: &str) -> Result<Subject>;

    /// Create an authentication token for a subject.
    async fn create_token(&self, subject: &Subject) -> Result<String>;

    /// Revoke an authentication token.
    async fn revoke_token(&self, token: &str) -> Result<()>;

    /// Refresh an authentication token.
    async fn refresh_token(&self, token: &str) -> Result<String>;
}

/// Factory for creating authentication providers.
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a new authentication provider from a configuration.
    pub fn create(config: &ProviderConfig) -> Result<Arc<dyn AuthProvider>> {
        use crate::error::Error;

        match config.provider_type {
            #[cfg(feature = "basic")]
            ProviderType::Basic => {
                let basic_config = serde_json::from_value::<crate::basic::BasicProviderConfig>(
                    config.config.clone(),
                )?;
                Ok(Arc::new(crate::basic::BasicProvider::new(
                    config.name.clone(),
                    basic_config,
                )))
            }
            #[cfg(feature = "jwt")]
            ProviderType::JWT => {
                let jwt_config = serde_json::from_value::<crate::token::TokenProviderConfig>(
                    config.config.clone(),
                )?;
                Ok(Arc::new(crate::token::JWTProvider::new(
                    config.name.clone(),
                    jwt_config,
                )))
            }
            #[cfg(feature = "oauth")]
            ProviderType::OAuth => {
                let oauth_config = serde_json::from_value::<crate::oauth::OAuthProviderConfig>(
                    config.config.clone(),
                )?;
                Ok(Arc::new(crate::oauth::OAuthProvider::new(
                    config.name.clone(),
                    oauth_config,
                )))
            }
            ProviderType::Custom => Err(Error::configuration(
                "Custom providers must be created manually",
            )),
            #[allow(unreachable_patterns)]
            _ => Err(Error::configuration(format!(
                "Unsupported provider type: {}",
                config.provider_type
            ))),
        }
    }
}
