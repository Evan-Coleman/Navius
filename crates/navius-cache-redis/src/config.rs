use crate::error::{RedisCacheError, RedisCacheResult};
use crate::serialization::SerializationFormat;
use navius_cache::config::CacheConfig;
use navius_cache::connection::ConnectionPoolConfig;
use navius_cache::error::CacheResult;
use std::fmt::Debug;
use std::time::Duration;

/// Options for key validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyValidationOptions {
    /// Whether to check key format
    pub check_format: bool,
    /// Whether to check for reserved characters
    pub check_reserved: bool,
}

impl Default for KeyValidationOptions {
    fn default() -> Self {
        Self {
            check_format: false,
            check_reserved: false,
        }
    }
}

impl KeyValidationOptions {
    /// Create a new KeyValidationOptions
    pub fn new(check_format: bool, check_reserved: bool) -> Self {
        Self {
            check_format,
            check_reserved,
        }
    }

    /// Check if options are default
    pub fn is_default(&self) -> bool {
        !self.check_format && !self.check_reserved
    }

    /// Check if options are valid
    pub fn is_valid(&self) -> bool {
        true
    }
}

/// Redis cache configuration
#[derive(Debug, Clone)]
pub struct RedisCacheConfig {
    /// Redis connection URL
    pub url: String,
    /// Redis key prefix
    pub key_prefix: Option<String>,
    /// Maximum connections in the pool
    pub max_connections: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Command timeout
    pub command_timeout: Duration,
    /// Whether to enable command retries
    pub retry_commands: bool,
    /// Maximum number of command retries
    pub max_retries: usize,
    /// Serialization format
    pub serialization_format: SerializationFormat,
    /// Default TTL for cache entries
    pub default_ttl: Option<Duration>,
    /// Metrics reporting interval (if enabled)
    pub metrics_interval: Option<Duration>,
    /// Key validation options
    pub key_validation: KeyValidationOptions,
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            key_prefix: None,
            max_connections: 10,
            connection_timeout: Duration::from_secs(5),
            command_timeout: Duration::from_secs(2),
            retry_commands: true,
            max_retries: 3,
            serialization_format: SerializationFormat::Json,
            default_ttl: Some(Duration::from_secs(3600)), // 1 hour
            metrics_interval: None,
            key_validation: KeyValidationOptions::default(),
        }
    }
}

impl RedisCacheConfig {
    /// Create a new Redis cache configuration
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> RedisCacheResult<()> {
        if self.url.is_empty() {
            return Err(RedisCacheError::Configuration(
                "Redis URL cannot be empty".to_string(),
            ));
        }

        if self.max_connections == 0 {
            return Err(RedisCacheError::Configuration(
                "Max connections must be greater than zero".to_string(),
            ));
        }

        if self.connection_timeout.as_millis() == 0 {
            return Err(RedisCacheError::Configuration(
                "Connection timeout must be greater than zero".to_string(),
            ));
        }

        if self.command_timeout.as_millis() == 0 {
            return Err(RedisCacheError::Configuration(
                "Command timeout must be greater than zero".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the prefixed key
    pub fn prefixed_key(&self, key: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}{}", prefix, key),
            None => key.to_string(),
        }
    }

    /// Set the key prefix
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// Set the default TTL
    pub fn with_default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = Some(ttl);
        self
    }

    /// Set the maximum number of connections
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = max_connections;
        self
    }

    /// Set the connection timeout
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set the command timeout
    pub fn with_command_timeout(mut self, timeout: Duration) -> Self {
        self.command_timeout = timeout;
        self
    }

    /// Set whether to retry commands on failure
    pub fn with_retry_commands(mut self, retry: bool) -> Self {
        self.retry_commands = retry;
        self
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the serialization format
    pub fn with_serialization_format(mut self, format: SerializationFormat) -> Self {
        self.serialization_format = format;
        self
    }

    /// Set the metrics interval
    pub fn with_metrics_interval(mut self, interval: Duration) -> Self {
        self.metrics_interval = Some(interval);
        self
    }

    /// Set the key validation options
    pub fn with_key_validation(mut self, validation: KeyValidationOptions) -> Self {
        self.key_validation = validation;
        self
    }
}

impl From<&CacheConfig> for RedisCacheConfig {
    fn from(config: &CacheConfig) -> Self {
        Self {
            url: config.url.clone(),
            key_prefix: Some(config.prefix.clone()),
            max_connections: config.max_connections as usize,
            connection_timeout: config.connect_timeout,
            command_timeout: Duration::from_secs(2), // Default command timeout
            retry_commands: true,
            max_retries: 3,
            serialization_format: SerializationFormat::Json,
            default_ttl: Some(config.default_ttl),
            metrics_interval: None,
            key_validation: KeyValidationOptions::default(),
        }
    }
}

// Implement ConnectionPoolConfig for RedisCacheConfig
impl ConnectionPoolConfig for RedisCacheConfig {
    fn max_connections(&self) -> usize {
        self.max_connections
    }

    fn connection_timeout(&self) -> Duration {
        self.connection_timeout
    }

    fn command_timeout(&self) -> Duration {
        self.command_timeout
    }

    fn retry_commands(&self) -> bool {
        self.retry_commands
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }

    fn validate(&self) -> CacheResult<()> {
        // Convert RedisCacheResult to CacheResult
        self.validate().map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RedisCacheConfig::default();
        assert_eq!(config.url, "redis://127.0.0.1:6379");
        assert_eq!(config.key_prefix, None);
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.connection_timeout, Duration::from_secs(5));
        assert_eq!(config.command_timeout, Duration::from_secs(2));
        assert!(config.retry_commands);
        assert_eq!(config.max_retries, 3);
        assert!(matches!(
            config.serialization_format,
            SerializationFormat::Json
        ));
        assert_eq!(config.default_ttl, Some(Duration::from_secs(3600)));
        assert_eq!(config.metrics_interval, None);
        assert!(config.key_validation.is_default());
    }

    #[test]
    fn test_custom_config() {
        let config = RedisCacheConfig::new("redis://custom-host:6379")
            .with_key_prefix("custom:")
            .with_default_ttl(Duration::from_secs(60))
            .with_max_connections(5)
            .with_connection_timeout(Duration::from_secs(2))
            .with_command_timeout(Duration::from_secs(1))
            .with_retry_commands(false)
            .with_max_retries(2)
            .with_serialization_format(SerializationFormat::MsgPack)
            .with_metrics_interval(Duration::from_secs(30))
            .with_key_validation(KeyValidationOptions::new(true, true));

        assert_eq!(config.url, "redis://custom-host:6379");
        assert_eq!(config.key_prefix, Some("custom:".to_string()));
        assert_eq!(config.max_connections, 5);
        assert_eq!(config.connection_timeout, Duration::from_secs(2));
        assert_eq!(config.command_timeout, Duration::from_secs(1));
        assert!(!config.retry_commands);
        assert_eq!(config.max_retries, 2);
        assert!(matches!(
            config.serialization_format,
            SerializationFormat::MsgPack
        ));
        assert_eq!(config.default_ttl, Some(Duration::from_secs(60)));
        assert_eq!(config.metrics_interval, Some(Duration::from_secs(30)));
        assert!(config.key_validation.is_valid());
    }

    #[test]
    fn test_validation() {
        let config = RedisCacheConfig::default();
        assert!(config.validate().is_ok());

        let config = RedisCacheConfig {
            url: "".to_string(),
            ..RedisCacheConfig::default()
        };
        assert!(config.validate().is_err());

        let config = RedisCacheConfig {
            max_connections: 0,
            ..RedisCacheConfig::default()
        };
        assert!(config.validate().is_err());

        let config = RedisCacheConfig {
            connection_timeout: Duration::from_secs(0),
            ..RedisCacheConfig::default()
        };
        assert!(config.validate().is_err());

        let config = RedisCacheConfig {
            command_timeout: Duration::from_secs(0),
            ..RedisCacheConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
