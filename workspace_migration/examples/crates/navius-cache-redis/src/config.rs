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

    /// Whether to retry failed commands
    #[serde(default = "default_retry_commands")]
    pub retry_commands: bool,

    /// Maximum number of retries for failed commands
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_max_connections() -> u32 {
    10
}

fn default_connection_timeout() -> u64 {
    5
}

fn default_command_timeout() -> u64 {
    2
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
            database: 0,
            password: None,
            use_tls: false,
            connection_timeout_seconds: 5,
            command_timeout_seconds: 2,
            retry_commands: true,
            max_retries: 3,
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
            database: 0,
            password: None,
            use_tls: false,
            connection_timeout_seconds: default_connection_timeout(),
            command_timeout_seconds: default_command_timeout(),
            retry_commands: default_retry_commands(),
            max_retries: default_max_retries(),
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
}
