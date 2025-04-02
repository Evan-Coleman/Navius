//! Authentication providers for Navius Auth.
//!
//! This module defines the provider interface and implementations for different
//! authentication methods such as JWT, OAuth, and basic authentication.

use crate::error::Error;
use crate::error::Result;
use crate::types::{Credentials, Identity, Subject};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Authentication provider type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    /// Basic authentication provider.
    Basic,
    /// JWT token provider.
    JWT,
    /// OAuth2 provider.
    OAuth2,
    /// OIDC provider.
    OIDC,
    /// SAML provider.
    SAML,
    /// External provider.
    External,
}

impl fmt::Display for ProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderType::Basic => write!(f, "basic"),
            ProviderType::JWT => write!(f, "jwt"),
            ProviderType::OAuth2 => write!(f, "oauth2"),
            ProviderType::OIDC => write!(f, "oidc"),
            ProviderType::SAML => write!(f, "saml"),
            ProviderType::External => write!(f, "external"),
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

/// Authentication provider trait.
///
/// This trait provides methods for authentication and token management.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Get the provider type.
    fn provider_type(&self) -> ProviderType;

    /// Get the provider name.
    fn name(&self) -> &str;

    /// Authenticate using the provided credentials.
    async fn authenticate(&self, credentials: &str) -> Result<Identity, Error>;

    /// Validate a token and return the subject.
    async fn validate_token(&self, token: &str) -> Result<Subject, Error>;

    /// Create a token for the provided subject.
    async fn create_token(&self, subject: &Subject) -> Result<String, Error>;

    /// Revoke a token.
    async fn revoke_token(&self, token: &str) -> Result<(), Error>;

    /// Refresh a token.
    async fn refresh_token(&self, token: &str) -> Result<String, Error>;
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
            ProviderType::OAuth2 => {
                let oauth_config = serde_json::from_value::<crate::oauth::OAuthProviderConfig>(
                    config.config.clone(),
                )?;
                Ok(Arc::new(crate::oauth::OAuthProvider::new(
                    config.name.clone(),
                    oauth_config,
                )))
            }
            ProviderType::OIDC => Err(Error::configuration(
                "OIDC providers are not supported in the current implementation",
            )),
            ProviderType::SAML => Err(Error::configuration(
                "SAML providers are not supported in the current implementation",
            )),
            ProviderType::External => Err(Error::configuration(
                "External providers are not supported in the current implementation",
            )),
            #[allow(unreachable_patterns)]
            _ => Err(Error::configuration(format!(
                "Unsupported provider type: {}",
                config.provider_type
            ))),
        }
    }
}
