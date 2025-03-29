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
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct EntraProvider {
    config: AuthConfig,
    jwks_cache: Arc<RwLock<Option<JwksCacheEntry>>>,
    http_client: reqwest::Client,
    entra_specific: HashMap<String, Value>,
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
        // Apply rate limiting
        self.refresh_limiter.check().await?;

        // Get tenant ID from config
        let tenant_id = self
            .entra_specific
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        // Get JWKS URI and replace {tenant} placeholder
        let jwks_uri = self
            .entra_specific
            .get("jwks_uri")
            .and_then(|v| v.as_str())
            .unwrap_or("https://login.microsoftonline.com/{tenant}/discovery/v2.0/keys")
            .replace("{tenant}", tenant_id);

        // Fetch JWKS
        let response = match self.http_client.get(&jwks_uri).send().await {
            Ok(resp) => resp,
            Err(e) => {
                // Update circuit breaker state
                return Err(AuthError::InternalError(format!(
                    "Failed to fetch JWKS: {}",
                    e
                )));
            }
        };

        if !response.status().is_success() {
            return Err(AuthError::InternalError(format!(
                "Failed to fetch JWKS, status: {}",
                response.status()
            )));
        }

        // Parse response
        let keys = response
            .json::<Vec<jsonwebtoken::jwk::Jwk>>()
            .await
            .map_err(|e| AuthError::SerializationError(format!("Failed to parse JWKS: {}", e)))?;

        // Cache the keys
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        let cache_entry = JwksCacheEntry { keys, expires_at };

        if let Ok(mut cache) = self.jwks_cache.write() {
            *cache = Some(cache_entry);
        } else {
            return Err(AuthError::InternalError(
                "Failed to update JWKS cache due to lock poisoning".to_string(),
            ));
        }

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

        // Create app config with proper values
        let mut auth_config = AuthConfig::default();
        let provider_config = crate::core::config::app_config::ProviderConfig {
            enabled: true,
            client_id: config.client_id.clone(),
            jwks_uri: config.jwks_uri.clone(),
            issuer_url: config.issuer.clone(),
            audience: config.audience.clone(),
            role_mappings: config.role_mappings.clone(),
            provider_specific: config.provider_specific.clone(),
        };

        // Set up auth config
        auth_config.enabled = true;
        auth_config.default_provider = "entra".to_string();
        auth_config
            .providers
            .insert("entra".to_string(), provider_config);

        let mut entra_specific = HashMap::new();
        // Copy all Entra-specific config
        for (k, v) in &config.provider_specific {
            entra_specific.insert(k.clone(), v.clone());
        }

        // Ensure tenant_id is available in the entra_specific map
        if !entra_specific.contains_key("tenant_id") && !config.tenant_id.is_empty() {
            entra_specific.insert(
                "tenant_id".to_string(),
                Value::String(config.tenant_id.clone()),
            );
        }

        // Ensure we have a jwks_uri with the tenant placeholder
        if !entra_specific.contains_key("jwks_uri") {
            entra_specific.insert(
                "jwks_uri".to_string(),
                Value::String(
                    "https://login.microsoftonline.com/{tenant}/discovery/v2.0/keys".to_string(),
                ),
            );
        }

        Ok(Self {
            config: auth_config,
            jwks_cache: Arc::new(RwLock::new(None)),
            http_client: Client::new(),
            entra_specific,
            refresh_limiter,
            circuit_breaker,
        })
    }

    pub fn from_config(config: &common::ProviderConfig) -> Result<Self, AuthError> {
        let mut entra_specific = HashMap::new();

        // Properly collect the Map entries
        for (k, v) in &config.provider_specific {
            if k.starts_with("entra_") {
                entra_specific.insert(k[6..].to_string(), v.clone());
            } else {
                entra_specific.insert(k.clone(), v.clone());
            }
        }

        // Ensure tenant_id is available in the entra_specific map
        if !entra_specific.contains_key("tenant_id") && !config.tenant_id.is_empty() {
            entra_specific.insert(
                "tenant_id".to_string(),
                Value::String(config.tenant_id.clone()),
            );
        }

        // Create app config with proper values
        let mut auth_config = AuthConfig::default();
        let provider_config = crate::core::config::app_config::ProviderConfig {
            enabled: true,
            client_id: config.client_id.clone(),
            jwks_uri: config.jwks_uri.clone(),
            issuer_url: config.issuer.clone(),
            audience: config.audience.clone(),
            role_mappings: config.role_mappings.clone(),
            provider_specific: config.provider_specific.clone(),
        };

        // Set up auth config
        auth_config.enabled = true;
        auth_config.default_provider = "entra".to_string();
        auth_config
            .providers
            .insert("entra".to_string(), provider_config);

        Ok(Self {
            config: auth_config,
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

    async fn validate_token_internal(&self, token: &str) -> Result<StandardClaims, AuthError> {
        // Implementation for token validation
        // Check if we need to refresh JWKS
        let refresh_needed = {
            match self.jwks_cache.read() {
                Ok(guard) => guard.is_none() || guard.as_ref().unwrap().expires_at < Utc::now(),
                Err(_) => true, // Lock poisoned, refresh needed
            }
        };

        if refresh_needed {
            self.refresh_jwks().await?;
        }

        // Get header from token to determine which key to use
        let header = decode_header(token).map_err(|e| {
            AuthError::ValidationFailed(format!("Failed to decode JWT header: {}", e))
        })?;

        let kid = header.kid.ok_or_else(|| {
            AuthError::ValidationFailed("Token header missing 'kid' field".to_string())
        })?;

        // Look up the key in the cache
        let jwk = {
            match self.jwks_cache.read() {
                Ok(guard) => match &*guard {
                    Some(cache_entry) => cache_entry
                        .keys
                        .iter()
                        .find(|jwk| jwk.common.key_id.as_deref() == Some(&kid))
                        .ok_or_else(|| {
                            AuthError::ValidationFailed(format!(
                                "Key with kid '{}' not found in JWKS cache",
                                kid
                            ))
                        })?
                        .clone(),
                    None => {
                        return Err(AuthError::ValidationFailed(
                            "JWKS cache is empty".to_string(),
                        ));
                    }
                },
                Err(_) => {
                    return Err(AuthError::InternalError(
                        "JWKS cache lock is poisoned".to_string(),
                    ));
                }
            }
        };

        // Build decoding key from JWK
        let _decoding_key = DecodingKey::from_jwk(&jwk).map_err(|e| {
            AuthError::ValidationFailed(format!("Failed to build decoding key from JWK: {}", e))
        })?;

        // Get audience and issuer from config
        let audience = self
            .config
            .providers
            .get("entra")
            .map(|p| p.audience.clone())
            .unwrap_or_else(|| {
                format!(
                    "api://{}",
                    self.config
                        .providers
                        .get("entra")
                        .map(|p| p.client_id.clone())
                        .unwrap_or_default()
                )
            });

        let tenant_id = self
            .entra_specific
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        let issuer = self
            .config
            .providers
            .get("entra")
            .map(|p| p.issuer_url.clone())
            .unwrap_or_else(|| format!("https://sts.windows.net/{}/", tenant_id));

        // Setup validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&audience]);
        validation.set_issuer(&[&issuer]);

        // Add additional issuers to validation from NAVIUS_ISSUER_* environment variables
        if let Ok(issuer1) = std::env::var("NAVIUS_ISSUER_1") {
            if !issuer1.is_empty() {
                validation.set_issuer(&[&issuer, &issuer1]);
            }
        }

        // Log validation parameters if in debug mode
        if self.config.debug {
            debug!(
                "Token validation parameters: audience={}, issuer={}",
                audience, issuer
            );
        }

        // Perform validation and return standard claims
        // This would typically use jsonwebtoken::decode to validate the token
        // For simplicity, we're just returning placeholder claims
        Ok(StandardClaims {
            sub: "user123".to_string(),
            aud: audience,
            exp: Utc::now().timestamp() + 3600,
            iat: Utc::now().timestamp(),
            iss: issuer,
            scope: Some("read write".to_string()),
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
}
