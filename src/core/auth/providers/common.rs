use crate::config::app_config::AuthConfig;
use crate::core::auth::AuthError;
use crate::core::auth::providers::entra::EntraProvider;
use crate::core::config::AppConfig;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use governor::{Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::NotKeyed};
use metrics::{counter, gauge, histogram};
use nonzero_ext::nonzero;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardClaims {
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub scope: Option<String>,
}

#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// Validate a token and return standard claims
    async fn validate_token(&self, token: &str) -> Result<StandardClaims, AuthError>;

    /// Refresh JWKS keys
    async fn refresh_jwks(&self) -> Result<(), AuthError>;

    /// Get provider configuration
    fn config(&self) -> &AuthConfig;

    /// Get provider-specific roles from token
    async fn get_roles(&self, token: &str) -> Result<Vec<String>, AuthError>;

    /// Get provider name
    fn name(&self) -> &str;

    /// Health check for the provider
    async fn health_check(&self) -> HealthStatus;

    /// Clone the provider
    fn box_clone(&self) -> Box<dyn OAuthProvider>;
}

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub ready: bool,
    pub jwks_valid: bool,
    pub last_refresh: SystemTime,
    pub error: Option<String>,
    #[serde(rename = "circuitState")]
    pub circuit_state: CircuitState,
}

/// Provider registry implementation
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn OAuthProvider>>,
    pub default_provider: String,
}

impl ProviderRegistry {
    pub fn new(config: AuthConfig) -> Self {
        let providers = HashMap::new();
        // Existing Entra provider initialization would go here
        Self {
            providers,
            default_provider: config.default_provider,
        }
    }

    pub fn get_provider(&self, name: &str) -> Option<&dyn OAuthProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    pub fn get_provider_arc(&self, name: &str) -> Option<Arc<dyn OAuthProvider>> {
        self.providers.get(name).cloned()
    }

    pub fn default_provider(&self) -> &dyn OAuthProvider {
        self.get_provider(&self.default_provider)
            .expect("Default provider not found")
    }

    pub fn initialize(config: AuthConfig) -> Result<Self, AuthError> {
        let mut providers: HashMap<String, Arc<dyn OAuthProvider>> = HashMap::new();

        for (name, provider_config) in &config.providers {
            if provider_config.enabled {
                info!("Initializing auth provider: {}", name);
                match name.as_str() {
                    "entra" => {
                        let common_config = ProviderConfig::from_app_config(provider_config);
                        let entra_provider = EntraProvider::new(common_config)?;
                        providers.insert(name.clone(), Arc::new(entra_provider));
                    }
                    // Add other providers here
                    _ => {
                        return Err(AuthError::ConfigurationError(format!(
                            "Unknown provider: {}",
                            name
                        )));
                    }
                }
            }
        }

        Ok(Self {
            providers,
            default_provider: config.default_provider,
        })
    }

    pub fn from_app_config(config: &AppConfig) -> Result<Self, AuthError> {
        let registry = Self::initialize(config.auth.clone())?;

        if config.auth.debug {
            debug!(
                "Auth provider registry initialized with {} providers",
                registry.providers.len()
            );
        }
        Ok(registry)
    }

    pub fn start_jwks_refresh(&self, interval_secs: u64) -> tokio::task::JoinHandle<()> {
        let providers = self.providers.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;

                for (name, provider) in &providers {
                    if let Err(e) = provider.refresh_jwks().await {
                        error!("Failed to refresh JWKS for {}: {}", name, e);
                    }
                }
            }
        })
    }

    pub fn initialize_with_refresh(config: AuthConfig) -> Result<Self, AuthError> {
        let registry = Self::initialize(config)?;
        registry.start_jwks_refresh(300); // Refresh every 5 minutes
        Ok(registry)
    }

    pub async fn check_health(&self) -> HashMap<String, HealthStatus> {
        let mut statuses = HashMap::new();

        for (name, provider) in &self.providers {
            let status = provider.health_check().await;

            // Record metrics - just log them for now
            let ready_value = if status.ready { 1.0 } else { 0.0 };
            let valid_value = if status.jwks_valid { 1.0 } else { 0.0 };

            debug!(
                "Provider {} status: ready={}, jwks_valid={}",
                name, ready_value, valid_value
            );

            statuses.insert(name.clone(), status);
        }

        statuses
    }
}

#[derive(Debug)]
pub struct RefreshLimiter {
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl RefreshLimiter {
    pub fn new(_requests: u32, per_seconds: u32) -> Self {
        let quota = Quota::with_period(Duration::from_secs(per_seconds.into()))
            .unwrap()
            .allow_burst(nonzero!(10u32));

        Self {
            limiter: RateLimiter::direct(quota),
        }
    }

    pub async fn check(&self) -> Result<(), AuthError> {
        self.limiter.until_ready().await;
        Ok(())
    }
}

// Manual Clone implementation for RefreshLimiter
impl Clone for RefreshLimiter {
    fn clone(&self) -> Self {
        // Create a new instance with the same configuration
        Self::new(10, 60) // Default values
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,
    Open(Instant),
    HalfOpen,
}

// Implement custom serialization for CircuitState to handle Instant
impl Serialize for CircuitState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            CircuitState::Closed => serializer.serialize_str("Closed"),
            CircuitState::HalfOpen => serializer.serialize_str("HalfOpen"),
            CircuitState::Open(instant) => {
                // Convert Instant to duration since UNIX_EPOCH
                let now = Instant::now();
                let duration = if *instant > now {
                    instant.duration_since(now)
                } else {
                    Duration::from_secs(0)
                };
                serializer.serialize_str(&format!("Open({:?})", duration))
            }
        }
    }
}

// Implement custom deserialization for CircuitState
impl<'de> Deserialize<'de> for CircuitState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Closed" => Ok(CircuitState::Closed),
            "HalfOpen" => Ok(CircuitState::HalfOpen),
            s if s.starts_with("Open(") => {
                // Default to 30 seconds if we can't parse the duration
                Ok(CircuitState::Open(Instant::now() + Duration::from_secs(30)))
            }
            _ => Ok(CircuitState::Closed), // Default
        }
    }
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub state: Arc<watch::Sender<CircuitState>>,
}

impl CircuitBreaker {
    fn new(_reset_timeout: Duration) -> Self {
        let (tx, _) = watch::channel(CircuitState::Closed);
        Self {
            state: Arc::new(tx),
        }
    }

    async fn check(&self) -> Result<(), AuthError> {
        match *self.state.borrow() {
            CircuitState::Closed => Ok(()),
            CircuitState::Open(until) if Instant::now() >= until => {
                self.state.send(CircuitState::HalfOpen)?;
                Ok(())
            }
            CircuitState::Open(_) => Err(AuthError::CircuitOpen),
            CircuitState::HalfOpen => Ok(()),
        }
    }

    fn record_success(&self) {
        if *self.state.borrow() == CircuitState::HalfOpen {
            self.state.send(CircuitState::Closed).unwrap_or_default();
        }
    }

    fn record_failure(&self, reset_timeout: Duration) {
        let until = Instant::now() + reset_timeout;
        match *self.state.borrow() {
            CircuitState::HalfOpen | CircuitState::Closed => {
                self.state
                    .send(CircuitState::Open(until))
                    .unwrap_or_default();
            }
            CircuitState::Open(current) if until > current => {
                self.state
                    .send(CircuitState::Open(until))
                    .unwrap_or_default();
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    // Basic fields that match with app_config::ProviderConfig
    pub enabled: bool,
    pub client_id: String,
    pub audience: String,
    pub jwks_uri: String,
    #[serde(rename = "issuer_url")]
    pub issuer: String,
    #[serde(default)]
    pub role_mappings: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub provider_specific: HashMap<String, Value>,
    // Additional fields specific to common::ProviderConfig
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate_limit: RateLimitConfig,
    pub tenant_id: String,
}

impl ProviderConfig {
    // Convert from app_config::ProviderConfig
    pub fn from_app_config(config: &crate::core::config::app_config::ProviderConfig) -> Self {
        // Extract tenant_id from provider_specific for Entra
        let tenant_id = config
            .provider_specific
            .get("entra_tenant_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Derive standard values if not explicitly provided
        let audience = if config.audience.is_empty() {
            format!("api://{}", config.client_id)
        } else {
            config.audience.clone()
        };

        let issuer = if config.issuer_url.is_empty() {
            format!("https://sts.windows.net/{}/", tenant_id)
        } else {
            config.issuer_url.clone()
        };

        Self {
            enabled: config.enabled,
            client_id: config.client_id.clone(),
            audience,
            jwks_uri: config.jwks_uri.clone(),
            issuer,
            role_mappings: config.role_mappings.clone(),
            provider_specific: config.provider_specific.clone(),
            refresh_rate_limit: default_refresh_rate(),
            tenant_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub per_seconds: u32,
}

fn default_refresh_rate() -> RateLimitConfig {
    RateLimitConfig {
        max_requests: 10,
        per_seconds: 60,
    }
}

#[derive(Debug, Clone)]
pub struct JwksCacheEntry {
    pub keys: Vec<jsonwebtoken::jwk::Jwk>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
