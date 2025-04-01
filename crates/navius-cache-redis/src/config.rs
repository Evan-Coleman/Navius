use navius_cache::config::CacheConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for Redis cache connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL (redis://...)
    pub url: String,

    /// Key prefix for namespacing
    pub key_prefix: String,

    /// Default time to live for cache entries
    pub default_ttl: Duration,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_seconds: u64,

    /// Command timeout in seconds
    #[serde(default = "default_command_timeout")]
    pub command_timeout_seconds: u64,
}

fn default_connection_timeout() -> u64 {
    5
}

fn default_command_timeout() -> u64 {
    2
}

impl From<CacheConfig> for RedisConfig {
    fn from(config: CacheConfig) -> Self {
        Self {
            url: config.url,
            key_prefix: config.prefix,
            default_ttl: config.default_ttl,
            connection_timeout_seconds: 5,
            command_timeout_seconds: 2,
        }
    }
}

impl RedisConfig {
    /// Create a new Redis cache configuration with defaults
    pub fn new(url: String, key_prefix: String) -> Self {
        Self {
            url,
            key_prefix,
            default_ttl: Duration::from_secs(300),
            connection_timeout_seconds: default_connection_timeout(),
            command_timeout_seconds: default_command_timeout(),
        }
    }

    /// Create a new Redis cache configuration with specific TTL
    pub fn with_ttl(url: String, key_prefix: String, default_ttl: Duration) -> Self {
        let mut config = Self::new(url, key_prefix);
        config.default_ttl = default_ttl;
        config
    }

    /// Get connection timeout as Duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_seconds)
    }

    /// Get command timeout as Duration
    pub fn command_timeout(&self) -> Duration {
        Duration::from_secs(self.command_timeout_seconds)
    }
}
