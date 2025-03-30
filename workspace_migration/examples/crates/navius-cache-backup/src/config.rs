use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Base cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache provider name
    pub provider: String,

    /// Connection URL for the cache
    pub url: String,

    /// Prefix for cache keys
    pub prefix: String,

    /// Default TTL for cache entries
    #[serde(with = "humantime_serde")]
    pub default_ttl: Duration,

    /// Whether to enable logging of cache operations
    #[serde(default = "default_logging")]
    pub log_operations: bool,

    /// Configuration for connecting to the cache
    #[serde(default)]
    pub connection: ConnectionConfig,

    /// Configuration for cache operations
    #[serde(default)]
    pub operations: OperationsConfig,
}

impl CacheConfig {
    /// Create a new cache configuration
    pub fn new(
        provider: impl Into<String>,
        url: impl Into<String>,
        prefix: impl Into<String>,
        default_ttl: Duration,
    ) -> Self {
        Self {
            provider: provider.into(),
            url: url.into(),
            prefix: prefix.into(),
            default_ttl,
            log_operations: default_logging(),
            connection: ConnectionConfig::default(),
            operations: OperationsConfig::default(),
        }
    }
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,

    /// Whether to retry failed connections
    #[serde(default = "default_retry_connections")]
    pub retry_connections: bool,

    /// Maximum number of retries for failed connections
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Delay between retries in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            connection_timeout_secs: default_connection_timeout(),
            retry_connections: default_retry_connections(),
            max_retries: default_max_retries(),
            retry_delay_ms: default_retry_delay(),
        }
    }
}

/// Operations configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationsConfig {
    /// Command timeout in seconds
    #[serde(default = "default_command_timeout")]
    pub command_timeout_secs: u64,

    /// Default serialization format
    #[serde(default = "default_serialization_format")]
    pub serialization_format: String,

    /// Whether to use compression
    #[serde(default)]
    pub use_compression: bool,

    /// Compression level (0-9, where 0 is no compression and 9 is maximum compression)
    #[serde(default = "default_compression_level")]
    pub compression_level: u32,

    /// Whether to track cache metrics
    #[serde(default = "default_track_metrics")]
    pub track_metrics: bool,

    /// Whether to retry failed operations
    #[serde(default = "default_retry_operations")]
    pub retry_operations: bool,

    /// Maximum number of retries for failed operations
    #[serde(default = "default_max_operation_retries")]
    pub max_operation_retries: u32,
}

impl Default for OperationsConfig {
    fn default() -> Self {
        Self {
            command_timeout_secs: default_command_timeout(),
            serialization_format: default_serialization_format(),
            use_compression: false,
            compression_level: default_compression_level(),
            track_metrics: default_track_metrics(),
            retry_operations: default_retry_operations(),
            max_operation_retries: default_max_operation_retries(),
        }
    }
}

// Default configuration values
fn default_logging() -> bool {
    true
}

fn default_max_connections() -> u32 {
    10
}

fn default_connection_timeout() -> u64 {
    5
}

fn default_retry_connections() -> bool {
    true
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    100
}

fn default_command_timeout() -> u64 {
    2
}

fn default_serialization_format() -> String {
    "json".to_string()
}

fn default_compression_level() -> u32 {
    6
}

fn default_track_metrics() -> bool {
    true
}

fn default_retry_operations() -> bool {
    true
}

fn default_max_operation_retries() -> u32 {
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CacheConfig::new(
            "redis",
            "redis://localhost:6379",
            "app:",
            Duration::from_secs(300),
        );

        assert_eq!(config.provider, "redis");
        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.prefix, "app:");
        assert_eq!(config.default_ttl, Duration::from_secs(300));
        assert!(config.log_operations);
        assert_eq!(config.connection.max_connections, 10);
        assert_eq!(config.operations.serialization_format, "json");
    }

    #[test]
    fn test_deserialization() {
        let config_json = r#"{
            "provider": "redis",
            "url": "redis://localhost:6379",
            "prefix": "app:",
            "default_ttl": "5m",
            "log_operations": false,
            "connection": {
                "max_connections": 20,
                "connection_timeout_secs": 10
            },
            "operations": {
                "serialization_format": "binary",
                "use_compression": true
            }
        }"#;

        let config: CacheConfig = serde_json::from_str(config_json).unwrap();
        assert_eq!(config.provider, "redis");
        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.prefix, "app:");
        assert_eq!(config.default_ttl, Duration::from_secs(300));
        assert!(!config.log_operations);
        assert_eq!(config.connection.max_connections, 20);
        assert_eq!(config.connection.connection_timeout_secs, 10);
        assert_eq!(config.operations.serialization_format, "binary");
        assert!(config.operations.use_compression);
    }
}
