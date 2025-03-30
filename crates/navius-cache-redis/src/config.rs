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

impl RedisCacheConfig {
    /// Create a new Redis cache configuration
    pub fn new(url: String, key_prefix: String, default_ttl: Duration) -> Self {
        Self {
            url,
            key_prefix,
            default_ttl,
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

    /// Get the Redis connection string
    pub fn connection_string(&self) -> String {
        let mut url = self.url.clone();

        // Check if URL already includes database
        if !url.contains("/") {
            url = format!("{}/{}", url, self.database);
        }

        url
    }

    /// Get the Redis client options
    pub fn client_options(&self) -> redis::RedisClientOptions {
        let mut options = redis::RedisClientOptions::default();

        options.connection_timeout = Some(Duration::from_secs(self.connection_timeout_seconds));
        options.command_timeout = Some(Duration::from_secs(self.command_timeout_seconds));

        options
    }

    /// Get the full key with prefix
    pub fn prefixed_key<K: AsRef<str>>(&self, key: K) -> String {
        format!("{}{}", self.key_prefix, key.as_ref())
    }
}

impl From<RedisCacheConfig> for CacheConfig {
    fn from(config: RedisCacheConfig) -> Self {
        CacheConfig {
            provider: "redis".to_string(),
            connection_string: config.connection_string(),
            key_prefix: config.key_prefix.clone(),
            default_ttl: config.default_ttl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_string() {
        let config = RedisCacheConfig::new(
            "redis://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(3600),
        );

        assert_eq!(config.connection_string(), "redis://localhost:6379/0");

        let config = RedisCacheConfig {
            url: "redis://localhost:6379/2".to_string(),
            key_prefix: "test:".to_string(),
            default_ttl: Duration::from_secs(3600),
            max_connections: 20,
            database: 5, // This should be ignored as the URL already has a database
            password: Some("secret".to_string()),
            use_tls: true,
            connection_timeout_seconds: 10,
            command_timeout_seconds: 5,
            retry_commands: true,
            max_retries: 5,
        };

        assert_eq!(config.connection_string(), "redis://localhost:6379/2");
    }

    #[test]
    fn test_prefixed_key() {
        let config = RedisCacheConfig::new(
            "redis://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(3600),
        );

        assert_eq!(config.prefixed_key("user:123"), "test:user:123");
    }
}
