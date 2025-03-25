use crate::core::auth::{AuthConfig, AuthError, StandardClaims};
use crate::core::config::AppConfig;
use async_trait::async_trait;
use governor::{Quota, RateLimiter};
use metrics::{counter, gauge, histogram};
use nonzero_ext::nonzero;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use tokio::sync::watch;
use tokio::time::Instant;

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
}

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub ready: bool,
    pub jwks_valid: bool,
    pub last_refresh: SystemTime,
    pub error: Option<String>,
    pub circuit_state: String,
}

/// Provider registry implementation
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn OAuthProvider>>,
    default_provider: String,
}

impl ProviderRegistry {
    pub fn new(config: AuthConfig) -> Self {
        let mut providers = HashMap::new();
        // Existing Entra provider initialization would go here
        Self {
            providers,
            default_provider: config.default_provider,
        }
    }

    pub fn get_provider(&self, name: &str) -> Option<&dyn OAuthProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    pub fn default_provider(&self) -> &dyn OAuthProvider {
        self.get_provider(&self.default_provider)
            .expect("Default provider not found")
    }

    pub fn initialize(config: AuthConfig) -> Result<Self, AuthError> {
        let mut providers = HashMap::new();

        for (name, provider_config) in &config.providers {
            if provider_config.enabled {
                info!("Initializing auth provider: {}", name);
                match name.as_str() {
                    "entra" => {
                        let entra_provider = EntraProvider::new(provider_config.clone())?;
                        providers.insert(name.clone(), Box::new(entra_provider));
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
        let mut registry = ProviderRegistry::initialize(config.auth.clone())?;

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
        let mut registry = Self::initialize(config)?;
        registry.start_jwks_refresh(300); // Refresh every 5 minutes
        Ok(registry)
    }

    pub async fn check_health(&self) -> HashMap<String, HealthStatus> {
        let mut statuses = HashMap::new();

        for (name, provider) in &self.providers {
            let status = provider.health_check().await;
            gauge!("auth_provider_ready", if status.ready { 1.0 } else { 0.0 }, "provider" => name);
            gauge!("auth_jwks_valid", if status.jwks_valid { 1.0 } else { 0.0 }, "provider" => name);
            statuses.insert(name.clone(), status);
        }

        statuses
    }
}

#[derive(Debug)]
struct RefreshLimiter {
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl RefreshLimiter {
    fn new(requests: u32, per_seconds: u32) -> Self {
        let quota = Quota::with_period(Duration::from_secs(per_seconds.into()))
            .unwrap()
            .allow_burst(nonzero!(requests));

        Self {
            limiter: RateLimiter::direct(quota),
        }
    }

    async fn check(&self) -> Result<(), AuthError> {
        self.limiter.until_ready().await;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct CircuitBreaker {
    state: Arc<watch::Sender<CircuitState>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open(Instant),
    HalfOpen,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
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
        if let CircuitState::HalfOpen = *self.state.borrow() {
            self.state.send(CircuitState::Closed).ok();
        }
    }

    fn record_failure(&self, reset_timeout: Duration) {
        let new_state = match *self.state.borrow() {
            CircuitState::Closed | CircuitState::HalfOpen => {
                CircuitState::Open(Instant::now() + reset_timeout)
            }
            _ => return,
        };
        self.state.send(new_state).ok();
    }
}

#[async_trait]
impl OAuthProvider for EntraProvider {
    async fn validate_token(&self, token: &str) -> Result<StandardClaims, AuthError> {
        self.circuit_breaker.check().await?;

        let result = self.inner_validate_token(token).await;

        match &result {
            Ok(_) => self.circuit_breaker.record_success(),
            Err(_) => self
                .circuit_breaker
                .record_failure(self.config.reset_timeout),
        }

        result
    }

    async fn refresh_jwks(&self) -> Result<(), AuthError> {
        let timer = metrics::Timer::start("auth_jwks_refresh_time");
        // ... existing refresh logic ...
        timer.stop();
        counter!("auth_jwks_refreshes", 1, "provider" => self.name());
        Ok(())
    }

    async fn health_check(&self) -> HealthStatus {
        let cache = self.jwks_cache.lock().unwrap();
        let circuit_state = match *self.circuit_breaker.state.borrow() {
            CircuitState::Closed => "closed",
            CircuitState::Open(_) => "open",
            CircuitState::HalfOpen => "half-open",
        };

        match &*cache {
            Some(entry) => HealthStatus {
                ready: true,
                jwks_valid: entry.expires_at > SystemTime::now(),
                last_refresh: entry
                    .expires_at
                    .checked_sub(Duration::from_secs(300))
                    .unwrap_or(entry.expires_at),
                error: None,
                circuit_state: circuit_state.to_string(),
            },
            None => HealthStatus {
                ready: false,
                jwks_valid: false,
                last_refresh: SystemTime::UNIX_EPOCH,
                error: Some("JWKS not initialized".to_string()),
                circuit_state: circuit_state.to_string(),
            },
        }
    }
}

// Update provider configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    // ... existing fields ...
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub per_seconds: u32,
}

fn default_refresh_rate() -> RateLimitConfig {
    RateLimitConfig {
        max_requests: 5,
        per_seconds: 60,
    }
}
