use super::constants;
use config::{Config, ConfigError, Environment, File};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::time::Duration;
use tracing::info;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_capacity: u64,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub cat_fact_url: String,
    pub petstore_url: String,
    pub api_key: Option<String>,
}

/// Entra ID (Azure AD) authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntraConfig {
    /// Tenant ID (from environment variable)
    #[serde(default)]
    pub tenant_id: String,

    /// Client ID (from environment variable)
    #[serde(default)]
    pub client_id: String,

    /// Audience (from environment variable)
    #[serde(default)]
    pub audience: String,

    /// Permission (defaults to "default-rust-backend")
    #[serde(default = "default_permission")]
    pub permission: String,

    /// Scope (from environment variable)
    #[serde(default)]
    pub scope: String,

    /// Token URL (from environment variable)
    #[serde(default)]
    pub token_url: String,

    /// Admin roles (users with these roles can access admin endpoints)
    #[serde(default = "default_admin_roles")]
    pub admin_roles: Vec<String>,

    /// Read-only roles (users with these roles can access read-only endpoints)
    #[serde(default = "default_read_only_roles")]
    pub read_only_roles: Vec<String>,

    /// Full access roles (users with these roles can access full access endpoints)
    #[serde(default = "default_full_access_roles")]
    pub full_access_roles: Vec<String>,
}

/// Default permission value
fn default_permission() -> String {
    constants::auth::permissions::DEFAULT_PERMISSION.to_string()
}

/// Default admin roles
fn default_admin_roles() -> Vec<String> {
    vec!["admin".to_string(), "pet-manager".to_string()]
}

/// Default read-only roles
fn default_read_only_roles() -> Vec<String> {
    vec!["reader".to_string(), "viewer".to_string()]
}

/// Default full access roles
fn default_full_access_roles() -> Vec<String> {
    vec!["editor".to_string(), "contributor".to_string()]
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub debug: bool,
    pub entra: EntraConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debug: false,
            entra: EntraConfig {
                tenant_id: env::var(constants::auth::env_vars::TENANT_ID).unwrap_or_default(),
                client_id: env::var(constants::auth::env_vars::CLIENT_ID).unwrap_or_default(),
                audience: env::var(constants::auth::env_vars::AUDIENCE).unwrap_or_default(),
                permission: env::var(constants::auth::env_vars::PERMISSION).unwrap_or_else(|_| {
                    constants::auth::permissions::DEFAULT_PERMISSION.to_string()
                }),
                scope: env::var(constants::auth::env_vars::SCOPE).unwrap_or_default(),
                token_url: env::var(constants::auth::env_vars::TOKEN_URL).unwrap_or_default(),
                admin_roles: default_admin_roles(),
                read_only_roles: default_read_only_roles(),
                full_access_roles: default_full_access_roles(),
            },
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub response_fields: ResponseLoggingConfig,
}

/// Response logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseLoggingConfig {
    pub cat_fact_fields: Vec<String>,
    pub health_fields: Vec<String>,
    pub api_response_fields: Vec<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            response_fields: ResponseLoggingConfig {
                cat_fact_fields: vec!["fact".to_string(), "length".to_string()],
                health_fields: vec!["status".to_string(), "uptime_seconds".to_string()],
                api_response_fields: vec!["code".to_string(), "message".to_string()],
            },
        }
    }
}

/// Reliability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityConfig {
    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,

    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,

    /// Timeout configuration
    #[serde(default)]
    pub timeout: TimeoutConfig,

    /// Concurrency limits
    #[serde(default)]
    pub concurrency: ConcurrencyConfig,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Whether retries are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum number of retry attempts
    #[serde(default = "default_retry_attempts")]
    pub max_attempts: u32,

    /// Base delay between retries in milliseconds
    #[serde(default = "default_retry_delay")]
    pub base_delay_ms: u64,

    /// Max delay between retries in milliseconds
    #[serde(default = "default_retry_max_delay")]
    pub max_delay_ms: u64,

    /// Whether to use exponential backoff
    #[serde(default = "default_true")]
    pub use_exponential_backoff: bool,

    /// Status codes that should trigger a retry
    #[serde(default = "default_retry_status_codes")]
    pub retry_status_codes: Vec<u16>,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Whether circuit breaker is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Number of consecutive failures before opening the circuit (legacy mode)
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,

    /// Time window in seconds for tracking failure rate
    #[serde(default = "default_window_seconds")]
    pub window_seconds: u64,

    /// Failure percentage threshold (0-100) that triggers the circuit breaker
    #[serde(default = "default_failure_percentage")]
    pub failure_percentage: u8,

    /// Whether to use the legacy consecutive failures mode (false = use rolling window)
    #[serde(default = "default_false")]
    pub use_consecutive_failures: bool,

    /// HTTP status codes that should be considered failures
    #[serde(default = "default_failure_status_codes")]
    pub failure_status_codes: Vec<u16>,

    /// Time in milliseconds the circuit stays open before moving to half-open
    #[serde(default = "default_reset_timeout")]
    pub reset_timeout_ms: u64,

    /// Number of successful requests in half-open state to close the circuit
    #[serde(default = "default_success_threshold")]
    pub success_threshold: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Number of requests allowed per time window
    #[serde(default = "default_rate_limit")]
    pub requests_per_window: u32,

    /// Time window in seconds
    #[serde(default = "default_rate_window")]
    pub window_seconds: u64,

    /// Whether to apply per-client rate limiting
    #[serde(default = "default_false")]
    pub per_client: bool,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Whether to enable request timeouts
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

/// Concurrency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// Whether to limit concurrency
    #[serde(default = "default_false")]
    pub enabled: bool,

    /// Maximum number of concurrent requests
    #[serde(default = "default_max_concurrency")]
    pub max_concurrent_requests: u32,
}

impl Default for ReliabilityConfig {
    fn default() -> Self {
        Self {
            retry: RetryConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            rate_limit: RateLimitConfig::default(),
            timeout: TimeoutConfig::default(),
            concurrency: ConcurrencyConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            max_attempts: default_retry_attempts(),
            base_delay_ms: default_retry_delay(),
            max_delay_ms: default_retry_max_delay(),
            use_exponential_backoff: default_true(),
            retry_status_codes: default_retry_status_codes(),
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            failure_threshold: default_failure_threshold(),
            window_seconds: default_window_seconds(),
            failure_percentage: default_failure_percentage(),
            use_consecutive_failures: default_false(),
            failure_status_codes: default_failure_status_codes(),
            reset_timeout_ms: default_reset_timeout(),
            success_threshold: default_success_threshold(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            requests_per_window: default_rate_limit(),
            window_seconds: default_rate_window(),
            per_client: default_false(),
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            timeout_seconds: default_timeout(),
        }
    }
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            enabled: default_false(),
            max_concurrent_requests: default_max_concurrency(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_retry_attempts() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    100
}

fn default_retry_max_delay() -> u64 {
    1000
}

fn default_failure_threshold() -> u32 {
    5
}

fn default_reset_timeout() -> u64 {
    30000
}

fn default_success_threshold() -> u32 {
    2
}

fn default_rate_limit() -> u32 {
    100
}

fn default_rate_window() -> u64 {
    60
}

fn default_timeout() -> u64 {
    30
}

fn default_max_concurrency() -> u32 {
    100
}

fn default_retry_status_codes() -> Vec<u16> {
    vec![408, 429, 500, 502, 503, 504]
}

fn default_window_seconds() -> u64 {
    60
}

fn default_failure_percentage() -> u8 {
    50
}

fn default_failure_status_codes() -> Vec<u16> {
    vec![500, 502, 503, 504]
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub api: ApiConfig,
    pub app: ApplicationConfig,
    pub cache: CacheConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub reliability: ReliabilityConfig,
    #[serde(default)]
    pub auth: AuthConfig,
}

/// Application metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: String,
    pub log_level: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        // Use the load_config function which implements the layered configuration approach
        load_config()
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    pub fn cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache.ttl_seconds)
    }

    /// Get the full Petstore API URL
    pub fn petstore_api_url(&self) -> String {
        self.api.petstore_url.trim_end_matches('/').to_string()
    }
}

/// Load configuration from files and environment variables
pub fn load_config() -> Result<AppConfig, ConfigError> {
    // Load .env file for secrets and overrides
    let _ = dotenv();

    // Determine the configuration directory
    let config_dir = env::var("CONFIG_DIR").unwrap_or_else(|_| "./config".to_string());

    // Determine the environment (development, production, etc.)
    let environment = env::var("RUN_ENV").unwrap_or_else(|_| "development".to_string());

    info!("Loading configuration for environment: {}", environment);

    // Build configuration with the following priority (highest to lowest):
    // 1. Environment variables (for secrets and CI/CD overrides)
    // 2. Environment-specific local overrides (local-{env}.yaml - not in version control)
    // 3. Environment-specific config ({env}.yaml)
    // 4. Local overrides (local.yaml - not in version control)
    // 5. Default config (default.yaml)
    let config = Config::builder()
        // 5. Start with default settings
        .add_source(File::from(Path::new(&config_dir).join("default.yaml")).required(false))
        // 4. Add local settings (not in version control)
        .add_source(File::from(Path::new(&config_dir).join("local.yaml")).required(false))
        // 3. Add environment-specific settings
        .add_source(
            File::from(Path::new(&config_dir).join(format!("{}.yaml", environment)))
                .required(false),
        )
        // 2. Add environment-specific local overrides (not in version control)
        .add_source(
            File::from(Path::new(&config_dir).join(format!("local-{}.yaml", environment)))
                .required(false),
        )
        // 1. Add environment variables (highest priority, for secrets and CI/CD)
        .add_source(Environment::with_prefix("SERVER").separator("_"))
        .add_source(Environment::with_prefix("API").separator("_"))
        .add_source(Environment::with_prefix("APP").separator("_"))
        .add_source(Environment::with_prefix("CACHE").separator("_"))
        .add_source(Environment::with_prefix("AUTH").separator("_"))
        .add_source(Environment::with_prefix("RELIABILITY").separator("_"))
        // Add specific environment variables for Entra ID auth
        .add_source(Environment::with_prefix("RUST_BACKEND").separator("_"))
        // Add legacy environment variables
        .add_source(Environment::default().try_parsing(true))
        // Build the config
        .build()?;

    // Deserialize the config into our AppConfig struct
    let mut app_config: AppConfig = config.try_deserialize()?;

    // Manually set Entra ID configuration from environment variables if they exist
    // This ensures the environment variables are properly mapped to the configuration
    if let Ok(tenant_id) = env::var(constants::auth::env_vars::TENANT_ID) {
        if !tenant_id.is_empty() {
            app_config.auth.entra.tenant_id = tenant_id;
        }
    }

    if let Ok(client_id) = env::var(constants::auth::env_vars::CLIENT_ID) {
        if !client_id.is_empty() {
            app_config.auth.entra.client_id = client_id;
        }
    }

    if let Ok(audience) = env::var(constants::auth::env_vars::AUDIENCE) {
        if !audience.is_empty() {
            app_config.auth.entra.audience = audience;
        }
    }

    if let Ok(permission) = env::var(constants::auth::env_vars::PERMISSION) {
        if !permission.is_empty() {
            app_config.auth.entra.permission = permission;
        }
    }

    if let Ok(scope) = env::var(constants::auth::env_vars::SCOPE) {
        if !scope.is_empty() {
            app_config.auth.entra.scope = scope;
        }
    }

    if let Ok(token_url) = env::var(constants::auth::env_vars::TOKEN_URL) {
        if !token_url.is_empty() {
            app_config.auth.entra.token_url = token_url;
        }
    }

    if let Ok(debug_auth) = env::var(constants::auth::env_vars::DEBUG_AUTH) {
        if !debug_auth.is_empty() {
            app_config.auth.debug = debug_auth.parse().unwrap_or(false);
        }
    }

    Ok(app_config)
}
