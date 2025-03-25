use super::*;
use crate::core::auth::error::AuthError;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode_header};
use reqwest::Client;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct EntraProvider {
    config: AuthConfig,
    jwks_cache: Arc<Mutex<Option<JwksCacheEntry>>>,
    http_client: reqwest::Client,
    entra_specific: HashMap<String, serde_yaml::Value>,
}

#[async_trait]
impl OAuthProvider for EntraProvider {
    async fn validate_token(&self, token: &str) -> Result<StandardClaims, AuthError> {
        // Existing validation logic adapted to return StandardClaims
        let header = decode_header(token)
            .map_err(|e| AuthError::ValidationFailed(format!("Invalid token header: {}", e)))?;

        // ... rest of validation logic ...

        Ok(StandardClaims {
            sub: claims.sub,
            aud: claims.aud,
            exp: claims.exp,
            iat: claims.iat,
            iss: claims.iss,
            scope: claims.scp,
        })
    }

    async fn refresh_jwks(&self) -> Result<(), AuthError> {
        // Existing JWKS refresh logic
    }

    fn config(&self) -> &AuthConfig {
        &self.config
    }

    async fn get_roles(&self, token: &str) -> Result<Vec<String>, AuthError> {
        // Decode token to get Entra-specific roles
        let claims = self.validate_token(token).await?;
        // Return roles from Entra-specific claims
        Ok(claims.roles)
    }

    fn name(&self) -> &str {
        "entra"
    }

    async fn get_roles_from_claims(
        &self,
        claims: &StandardClaims,
    ) -> Result<Vec<String>, AuthError> {
        // Convert standard claims to Entra-specific claims
        let entra_claims: EntraClaims = serde_json::from_value(serde_json::to_value(claims)?)
            .map_err(|e| AuthError::ValidationFailed(e.to_string()))?;

        Ok(entra_claims.roles)
    }

    async fn health_check(&self) -> HealthStatus {
        let cache = self.jwks_cache.lock().unwrap();
        match &*cache {
            Some(entry) => HealthStatus {
                ready: true,
                jwks_valid: entry.expires_at > SystemTime::now(),
                last_refresh: entry
                    .expires_at
                    .checked_sub(Duration::from_secs(300))
                    .unwrap_or(entry.expires_at),
                error: None,
            },
            None => HealthStatus {
                ready: false,
                jwks_valid: false,
                last_refresh: SystemTime::UNIX_EPOCH,
                error: Some("JWKS not initialized".to_string()),
            },
        }
    }

    async fn inner_validate_token(&self, token: &str) -> Result<StandardClaims, AuthError> {
        // ... existing validation logic ...
        // Convert Entra-specific claims to standard claims
        Ok(StandardClaims {
            sub: entra_claims.sub,
            aud: entra_claims.aud,
            exp: entra_claims.exp,
            iat: entra_claims.iat,
            iss: entra_claims.iss,
            scope: entra_claims.scp,
        })
    }
}

impl EntraProvider {
    pub fn new(config: ProviderConfig) -> Result<Self, AuthError> {
        Ok(Self {
            config,
            jwks_cache: Arc::new(Mutex::new(None)),
            http_client: Client::new(),
            entra_specific: HashMap::new(),
        })
    }

    pub fn from_config(config: &ProviderConfig) -> Result<Self, AuthError> {
        let entra_specific = config
            .provider_specific
            .iter()
            .filter(|(k, _)| k.starts_with("entra_"))
            .collect();

        Ok(Self {
            config: config.clone(),
            jwks_cache: Arc::new(Mutex::new(None)),
            http_client: Client::new(),
            entra_specific,
        })
    }
}
