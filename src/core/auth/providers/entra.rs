//! Microsoft Entra ID (Azure AD) OAuth provider

use super::common::{
    self, CircuitBreaker, CircuitState, HealthStatus, JwksCacheEntry, OAuthProvider,
    ProviderConfig, RateLimitConfig, RefreshLimiter, StandardClaims,
};
use crate::config::app_config::AuthConfig;
use crate::core::auth::error::AuthError;
use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode_header};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::sync::watch;

#[derive(Debug, Clone)]
pub struct EntraProvider {
    config: AuthConfig,
    jwks_cache: Arc<RwLock<Option<JwksCacheEntry>>>,
    http_client: reqwest::Client,
    entra_specific: HashMap<String, serde_yaml::Value>,
    refresh_limiter: RefreshLimiter,
    circuit_breaker: CircuitBreaker,
}

#[derive(Debug, Deserialize)]
struct EntraClaims {
    sub: String,
    aud: String,
    exp: i64,
    iat: i64,
    iss: String,
    scp: Option<String>,
    roles: Vec<String>,
}

#[async_trait]
impl OAuthProvider for EntraProvider {
    async fn validate_token(&self, token: &str) -> Result<StandardClaims, AuthError> {
        self.validate_token_internal(token).await
    }

    async fn refresh_jwks(&self) -> Result<(), AuthError> {
        // Existing JWKS refresh logic
        Ok(())
    }

    fn config(&self) -> &AuthConfig {
        &self.config
    }

    async fn get_roles(&self, token: &str) -> Result<Vec<String>, AuthError> {
        let claims = self.validate_token(token).await?;
        self.extract_roles(&claims).await
    }

    fn name(&self) -> &str {
        "entra"
    }

    async fn health_check(&self) -> HealthStatus {
        let jwks_valid = {
            match self.jwks_cache.read() {
                Ok(guard) => guard.is_some(),
                Err(_) => false, // Lock poisoned
            }
        };

        let last_refresh = SystemTime::now(); // Placeholder - would come from the actual cache

        HealthStatus {
            ready: true,
            jwks_valid,
            last_refresh,
            error: None,
            circuit_state: common::CircuitState::Closed,
        }
    }

    fn box_clone(&self) -> Box<dyn OAuthProvider> {
        Box::new(self.clone())
    }
}

impl EntraProvider {
    pub fn new(config: common::ProviderConfig) -> Result<Self, AuthError> {
        let refresh_limiter = RefreshLimiter::new(
            config.refresh_rate_limit.max_requests,
            config.refresh_rate_limit.per_seconds,
        );

        let (tx, _) = watch::channel(CircuitState::Closed);
        let circuit_breaker = CircuitBreaker {
            state: Arc::new(tx),
        };

        Ok(Self {
            config: AuthConfig::default(),
            jwks_cache: Arc::new(RwLock::new(None)),
            http_client: Client::new(),
            entra_specific: HashMap::new(),
            refresh_limiter,
            circuit_breaker,
        })
    }

    pub fn from_config(config: &common::ProviderConfig) -> Result<Self, AuthError> {
        let mut entra_specific = HashMap::new();

        // Properly collect the Map entries
        for (k, v) in &config.provider_specific {
            if k.starts_with("entra_") {
                entra_specific.insert(k.clone(), v.clone());
            }
        }

        Ok(Self {
            config: AuthConfig::default(),
            jwks_cache: Arc::new(RwLock::new(None)),
            http_client: Client::new(),
            entra_specific,
            refresh_limiter: RefreshLimiter::new(
                config.refresh_rate_limit.max_requests,
                config.refresh_rate_limit.per_seconds,
            ),
            circuit_breaker: CircuitBreaker {
                state: Arc::new(watch::channel(common::CircuitState::Closed).0),
            },
        })
    }

    async fn extract_roles(&self, claims: &StandardClaims) -> Result<Vec<String>, AuthError> {
        // For Entra, roles are typically in the scope field
        if let Some(scope) = &claims.scope {
            Ok(scope.split(' ').map(String::from).collect())
        } else {
            Ok(Vec::new())
        }
    }

    async fn validate_token_internal(&self, _token: &str) -> Result<StandardClaims, AuthError> {
        // Safely acquire the read lock
        let cache_entry = match self.jwks_cache.read() {
            Ok(guard) => guard.clone(), // Clone the Option to avoid holding the lock
            Err(_) => None,             // Lock poisoned, treat as empty cache
        };

        if let Some(entry) = cache_entry {
            if entry.expires_at > Utc::now() {
                // Use cached JWKS
                // ... existing validation code ...
            }
        }

        // Refresh JWKS if needed
        self.refresh_jwks().await?;

        // ... rest of validation code ...
        todo!("Implement token validation")
    }
}
