use navius_cache::config::CacheConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for Redis cache connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisCacheConfig {
    /// Redis connection URL (redis://...)
    pub url: String,

    /// Key prefix for namespacing
    pub key_prefix: String,

    /// Default time to live for cache entries
    pub default_ttl: Duration,

    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum number of connections to maintain in the pool
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Redis database index (0-15)
    #[serde(default)]
    pub database: u8,

    /// Redis password
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Whether to use TLS
    #[serde(default)]
    pub use_tls: bool,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_seconds: u64,

    /// Command timeout in seconds
    #[serde(default = "default_command_timeout")]
    pub command_timeout_seconds: u64,

    /// Idle timeout in seconds - how long a connection can remain idle before being removed
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_seconds: u64,

    /// Maximum lifetime of a connection in seconds
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime_seconds: u64,

    /// Whether to retry failed commands
    #[serde(default = "default_retry_commands")]
    pub retry_commands: bool,

    /// Maximum number of retries for failed commands
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Health check interval in seconds
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval_seconds: u64,

    /// Circuit breaker threshold - number of consecutive failures before opening circuit
    #[serde(default = "default_circuit_breaker_threshold")]
    pub circuit_breaker_threshold: u32,

    /// Circuit breaker reset timeout in seconds
    #[serde(default = "default_circuit_reset_timeout")]
    pub circuit_reset_timeout_seconds: u64,

    /// Whether to enable connection pool metrics
    #[serde(default)]
    pub enable_metrics: bool,
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    2
}

fn default_connection_timeout() -> u64 {
    5
}

fn default_command_timeout() -> u64 {
    2
}

fn default_idle_timeout() -> u64 {
    60 // 1 minute
}

fn default_max_lifetime() -> u64 {
    300 // 5 minutes
}

fn default_health_check_interval() -> u64 {
    30 // 30 seconds
}

fn default_circuit_breaker_threshold() -> u32 {
    5
}

fn default_circuit_reset_timeout() -> u64 {
    5 // 5 seconds
}

fn default_retry_commands() -> bool {
    true
}

fn default_max_retries() -> u32 {
    3
}

impl From<CacheConfig> for RedisCacheConfig {
    fn from(config: CacheConfig) -> Self {
        Self {
            url: config.url,
            key_prefix: config.prefix,
            default_ttl: config.default_ttl,
            max_connections: config.max_connections as u32,
            min_connections: (config.max_connections / 5) as u32,
            database: 0,
            password: None,
            use_tls: false,
            connection_timeout_seconds: 5,
            command_timeout_seconds: 2,
            idle_timeout_seconds: default_idle_timeout(),
            max_lifetime_seconds: default_max_lifetime(),
            retry_commands: true,
            max_retries: 3,
            health_check_interval_seconds: default_health_check_interval(),
            circuit_breaker_threshold: default_circuit_breaker_threshold(),
            circuit_reset_timeout_seconds: default_circuit_reset_timeout(),
            enable_metrics: config.metrics,
        }
    }
}

impl RedisCacheConfig {
    /// Create a new Redis cache configuration with defaults
    pub fn new(url: String, key_prefix: String) -> Self {
        Self {
            url,
            key_prefix,
            default_ttl: Duration::from_secs(300),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            database: 0,
            password: None,
            use_tls: false,
            connection_timeout_seconds: default_connection_timeout(),
            command_timeout_seconds: default_command_timeout(),
            idle_timeout_seconds: default_idle_timeout(),
            max_lifetime_seconds: default_max_lifetime(),
            retry_commands: default_retry_commands(),
            max_retries: default_max_retries(),
            health_check_interval_seconds: default_health_check_interval(),
            circuit_breaker_threshold: default_circuit_breaker_threshold(),
            circuit_reset_timeout_seconds: default_circuit_reset_timeout(),
            enable_metrics: false,
        }
    }

    /// Create a new Redis cache configuration with specific TTL
    pub fn with_ttl(url: String, key_prefix: String, default_ttl: Duration) -> Self {
        let mut config = Self::new(url, key_prefix);
        config.default_ttl = default_ttl;
        config
    }

    /// Create a high-availability configuration with optimized settings
    pub fn high_availability(url: String, key_prefix: String) -> Self {
        Self {
            url,
            key_prefix,
            default_ttl: Duration::from_secs(300),
            max_connections: 20,
            min_connections: 5,
            database: 0,
            password: None,
            use_tls: true,
            connection_timeout_seconds: 3,
            command_timeout_seconds: 1,
            idle_timeout_seconds: 30,
            max_lifetime_seconds: 120,
            retry_commands: true,
            max_retries: 5,
            health_check_interval_seconds: 15,
            circuit_breaker_threshold: 3,
            circuit_reset_timeout_seconds: 3,
            enable_metrics: true,
        }
    }

    /// Create a prefixed key
    pub fn prefixed_key<K: AsRef<str>>(&self, key: K) -> String {
        if self.key_prefix.is_empty() {
            key.as_ref().to_string()
        } else {
            format!("{}:{}", self.key_prefix, key.as_ref())
        }
    }

    /// Get connection timeout as Duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_seconds)
    }

    /// Get command timeout as Duration
    pub fn command_timeout(&self) -> Duration {
        Duration::from_secs(self.command_timeout_seconds)
    }

    /// Get idle timeout as Duration
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_seconds)
    }

    /// Get max lifetime as Duration
    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_seconds)
    }

    /// Get health check interval as Duration
    pub fn health_check_interval(&self) -> Duration {
        Duration::from_secs(self.health_check_interval_seconds)
    }

    /// Get circuit reset timeout as Duration
    pub fn circuit_reset_timeout(&self) -> Duration {
        Duration::from_secs(self.circuit_reset_timeout_seconds)
    }
}
