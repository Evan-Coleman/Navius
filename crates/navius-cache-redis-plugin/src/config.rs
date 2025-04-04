use navius_cache::CacheConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the Redis cache implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisCacheConfig {
    /// Redis connection URL (e.g., "redis://127.0.0.1:6379/")
    pub url: String,

    /// Key prefix to use for all cache keys
    pub key_prefix: String,

    /// Default Time-To-Live for cache entries
    pub default_ttl: Duration,

    /// Pool configuration
    pub pool_config: PoolConfig,

    /// TLS configuration
    pub tls_config: TlsConfig,

    /// Cluster configuration
    pub cluster_config: ClusterConfig,
}

/// Pool configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum number of connections in the pool
    pub min_connections: Option<u32>,

    /// Maximum number of connections in the pool
    pub max_connections: Option<u32>,

    /// Connection timeout
    pub connect_timeout: Option<Duration>,

    /// Command timeout
    pub command_timeout: Option<Duration>,

    /// Maximum number of connection retries
    pub connect_retries: u32,

    /// Whether to enable connection recycling
    pub enable_connection_recycling: bool,

    /// Time between health checks
    pub health_check_interval: Duration,
}

/// TLS configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Whether TLS is enabled
    pub enabled: bool,

    /// Server name for TLS verification
    pub server_name: Option<String>,

    /// Path to CA certificate file
    pub ca_cert_path: Option<String>,

    /// Path to client certificate file
    pub client_cert_path: Option<String>,

    /// Path to client key file
    pub client_key_path: Option<String>,
}

/// Cluster configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Whether cluster support is enabled
    pub enabled: bool,

    /// Number of retries for cluster operations
    pub retry_count: u32,

    /// Use read-only replicas for read operations
    pub read_from_replicas: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: Some(1),
            max_connections: Some(10),
            connect_timeout: Some(Duration::from_secs(5)),
            command_timeout: Some(Duration::from_secs(3)),
            connect_retries: 3,
            enable_connection_recycling: true,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            server_name: None,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
        }
    }
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            retry_count: 3,
            read_from_replicas: false,
        }
    }
}

impl RedisCacheConfig {
    /// Create a new Redis cache configuration
    pub fn new(url: String, key_prefix: String, default_ttl: Duration) -> Self {
        Self {
            url,
            key_prefix,
            default_ttl,
            pool_config: PoolConfig::default(),
            tls_config: TlsConfig::default(),
            cluster_config: ClusterConfig::default(),
        }
    }

    /// Create a Redis cache configuration from a navius-cache CacheConfig
    pub fn from_cache_config(cache_config: &CacheConfig) -> Result<Self, String> {
        let url = extract_string_value(cache_config, "url")
            .map_err(|e| format!("Missing Redis URL: {}", e))?;

        let key_prefix =
            extract_string_value(cache_config, "key_prefix").unwrap_or_else(|_| "".to_string());

        let default_ttl = extract_duration_value(cache_config, "default_ttl")
            .unwrap_or_else(|_| Duration::from_secs(60 * 60)); // Default 1 hour

        let mut redis_config = Self::new(url, key_prefix, default_ttl);

        // Extract optional pool configuration
        if let Some(min_connections) = extract_u32_value(cache_config, "pool.min_connections").ok()
        {
            redis_config.pool_config.min_connections = Some(min_connections);
        }

        if let Some(max_connections) = extract_u32_value(cache_config, "pool.max_connections").ok()
        {
            redis_config.pool_config.max_connections = Some(max_connections);
        }

        if let Some(connect_timeout) =
            extract_duration_value(cache_config, "pool.connect_timeout").ok()
        {
            redis_config.pool_config.connect_timeout = Some(connect_timeout);
        }

        if let Some(command_timeout) =
            extract_duration_value(cache_config, "pool.command_timeout").ok()
        {
            redis_config.pool_config.command_timeout = Some(command_timeout);
        }

        if let Some(connect_retries) = extract_u32_value(cache_config, "pool.connect_retries").ok()
        {
            redis_config.pool_config.connect_retries = connect_retries;
        }

        if let Some(enable_connection_recycling) =
            extract_bool_value(cache_config, "pool.enable_connection_recycling").ok()
        {
            redis_config.pool_config.enable_connection_recycling = enable_connection_recycling;
        }

        if let Some(health_check_interval) =
            extract_duration_value(cache_config, "pool.health_check_interval").ok()
        {
            redis_config.pool_config.health_check_interval = health_check_interval;
        }

        // Extract optional TLS configuration
        if let Some(enabled) = extract_bool_value(cache_config, "tls.enabled").ok() {
            redis_config.tls_config.enabled = enabled;

            if enabled {
                if let Some(server_name) =
                    extract_string_value(cache_config, "tls.server_name").ok()
                {
                    redis_config.tls_config.server_name = Some(server_name);
                }

                if let Some(ca_cert_path) =
                    extract_string_value(cache_config, "tls.ca_cert_path").ok()
                {
                    redis_config.tls_config.ca_cert_path = Some(ca_cert_path);
                }

                if let Some(client_cert_path) =
                    extract_string_value(cache_config, "tls.client_cert_path").ok()
                {
                    redis_config.tls_config.client_cert_path = Some(client_cert_path);
                }

                if let Some(client_key_path) =
                    extract_string_value(cache_config, "tls.client_key_path").ok()
                {
                    redis_config.tls_config.client_key_path = Some(client_key_path);
                }
            }
        }

        // Extract optional cluster configuration
        if let Some(enabled) = extract_bool_value(cache_config, "cluster.enabled").ok() {
            redis_config.cluster_config.enabled = enabled;

            if enabled {
                if let Some(retry_count) =
                    extract_u32_value(cache_config, "cluster.retry_count").ok()
                {
                    redis_config.cluster_config.retry_count = retry_count;
                }

                if let Some(read_from_replicas) =
                    extract_bool_value(cache_config, "cluster.read_from_replicas").ok()
                {
                    redis_config.cluster_config.read_from_replicas = read_from_replicas;
                }
            }
        }

        Ok(redis_config)
    }

    /// Convert this configuration to a navius-cache CacheConfig
    pub fn to_cache_config(&self) -> CacheConfig {
        let mut config = CacheConfig::new();

        config.insert("url".to_string(), self.url.clone());
        config.insert("key_prefix".to_string(), self.key_prefix.clone());
        config.insert(
            "default_ttl".to_string(),
            self.default_ttl.as_secs().to_string(),
        );

        // Add pool configuration
        if let Some(min_connections) = self.pool_config.min_connections {
            config.insert(
                "pool.min_connections".to_string(),
                min_connections.to_string(),
            );
        }

        if let Some(max_connections) = self.pool_config.max_connections {
            config.insert(
                "pool.max_connections".to_string(),
                max_connections.to_string(),
            );
        }

        if let Some(connect_timeout) = self.pool_config.connect_timeout {
            config.insert(
                "pool.connect_timeout".to_string(),
                connect_timeout.as_secs().to_string(),
            );
        }

        if let Some(command_timeout) = self.pool_config.command_timeout {
            config.insert(
                "pool.command_timeout".to_string(),
                command_timeout.as_secs().to_string(),
            );
        }

        config.insert(
            "pool.connect_retries".to_string(),
            self.pool_config.connect_retries.to_string(),
        );
        config.insert(
            "pool.enable_connection_recycling".to_string(),
            self.pool_config.enable_connection_recycling.to_string(),
        );
        config.insert(
            "pool.health_check_interval".to_string(),
            self.pool_config.health_check_interval.as_secs().to_string(),
        );

        // Add TLS configuration
        config.insert(
            "tls.enabled".to_string(),
            self.tls_config.enabled.to_string(),
        );

        if self.tls_config.enabled {
            if let Some(ref server_name) = self.tls_config.server_name {
                config.insert("tls.server_name".to_string(), server_name.clone());
            }

            if let Some(ref ca_cert_path) = self.tls_config.ca_cert_path {
                config.insert("tls.ca_cert_path".to_string(), ca_cert_path.clone());
            }

            if let Some(ref client_cert_path) = self.tls_config.client_cert_path {
                config.insert("tls.client_cert_path".to_string(), client_cert_path.clone());
            }

            if let Some(ref client_key_path) = self.tls_config.client_key_path {
                config.insert("tls.client_key_path".to_string(), client_key_path.clone());
            }
        }

        // Add cluster configuration
        config.insert(
            "cluster.enabled".to_string(),
            self.cluster_config.enabled.to_string(),
        );

        if self.cluster_config.enabled {
            config.insert(
                "cluster.retry_count".to_string(),
                self.cluster_config.retry_count.to_string(),
            );
            config.insert(
                "cluster.read_from_replicas".to_string(),
                self.cluster_config.read_from_replicas.to_string(),
            );
        }

        config
    }

    /// Parse a Redis URL and validate it
    pub fn validate_url(&self) -> Result<(), String> {
        if !self.url.starts_with("redis://") && !self.url.starts_with("rediss://") {
            return Err(format!(
                "Redis URL must start with 'redis://' or 'rediss://'"
            ));
        }
        Ok(())
    }

    /// Create a new configuration with a pool configuration
    pub fn with_pool(mut self, pool_config: PoolConfig) -> Self {
        self.pool_config = pool_config;
        self
    }

    /// Create a new configuration with TLS settings
    pub fn with_tls(mut self, tls_config: TlsConfig) -> Self {
        self.tls_config = tls_config;
        self
    }

    /// Create a new configuration with cluster settings
    pub fn with_cluster(mut self, cluster_config: ClusterConfig) -> Self {
        self.cluster_config = cluster_config;
        self
    }
}

/// Extract a string value from a CacheConfig
fn extract_string_value(config: &CacheConfig, key: &str) -> Result<String, String> {
    config
        .get(key)
        .ok_or_else(|| format!("Missing config key: {}", key))
        .map(|v| v.to_string())
}

/// Extract a u32 value from a CacheConfig
fn extract_u32_value(config: &CacheConfig, key: &str) -> Result<u32, String> {
    let value = extract_string_value(config, key)?;
    value
        .parse::<u32>()
        .map_err(|e| format!("Invalid u32 value for key: {}, got: {}", key, e))
}

/// Extract a boolean value from a CacheConfig
fn extract_bool_value(config: &CacheConfig, key: &str) -> Result<bool, String> {
    let value = extract_string_value(config, key)?;
    value
        .parse::<bool>()
        .map_err(|e| format!("Invalid boolean value for key: {}, got: {}", key, e))
}

/// Extract a Duration value (in seconds) from a CacheConfig
fn extract_duration_value(config: &CacheConfig, key: &str) -> Result<Duration, String> {
    let seconds = extract_u32_value(config, key)?;
    Ok(Duration::from_secs(seconds as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pool_config() {
        let pool_config = PoolConfig::default();
        assert_eq!(pool_config.min_connections, Some(1));
        assert_eq!(pool_config.max_connections, Some(10));
        assert_eq!(pool_config.connect_timeout, Some(Duration::from_secs(5)));
        assert_eq!(pool_config.command_timeout, Some(Duration::from_secs(3)));
        assert_eq!(pool_config.connect_retries, 3);
        assert!(pool_config.enable_connection_recycling);
        assert_eq!(pool_config.health_check_interval, Duration::from_secs(30));
    }

    #[test]
    fn test_default_tls_config() {
        let tls_config = TlsConfig::default();
        assert!(!tls_config.enabled);
        assert_eq!(tls_config.server_name, None);
        assert_eq!(tls_config.ca_cert_path, None);
        assert_eq!(tls_config.client_cert_path, None);
        assert_eq!(tls_config.client_key_path, None);
    }

    #[test]
    fn test_default_cluster_config() {
        let cluster_config = ClusterConfig::default();
        assert!(!cluster_config.enabled);
        assert_eq!(cluster_config.retry_count, 3);
        assert!(!cluster_config.read_from_replicas);
    }

    #[test]
    fn test_create_redis_config() {
        let config = RedisCacheConfig::new(
            "redis://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(60),
        );

        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.key_prefix, "test:");
        assert_eq!(config.default_ttl.as_secs(), 60);

        // Default Pool config
        assert_eq!(config.pool_config.min_connections, Some(1));
        assert_eq!(config.pool_config.max_connections, Some(10));

        // TLS disabled by default
        assert!(!config.tls_config.enabled);

        // Cluster disabled by default
        assert!(!config.cluster_config.enabled);
    }

    #[test]
    fn test_config_builder_pattern() {
        let pool_config = PoolConfig {
            min_connections: Some(5),
            max_connections: Some(20),
            ..Default::default()
        };

        let tls_config = TlsConfig {
            enabled: true,
            server_name: Some("redis.example.com".to_string()),
            ..Default::default()
        };

        let config = RedisCacheConfig::new(
            "redis://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(60),
        )
        .with_pool(pool_config)
        .with_tls(tls_config);

        assert_eq!(config.pool_config.min_connections, Some(5));
        assert_eq!(config.pool_config.max_connections, Some(20));
        assert!(config.tls_config.enabled);
        assert_eq!(
            config.tls_config.server_name,
            Some("redis.example.com".to_string())
        );
    }

    #[test]
    fn test_validate_url() {
        let valid_config = RedisCacheConfig::new(
            "redis://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(60),
        );

        assert!(valid_config.validate_url().is_ok());

        let valid_tls_config = RedisCacheConfig::new(
            "rediss://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(60),
        );

        assert!(valid_tls_config.validate_url().is_ok());

        let invalid_config = RedisCacheConfig::new(
            "invalid://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(60),
        );

        assert!(invalid_config.validate_url().is_err());
    }

    #[test]
    fn test_to_cache_config() {
        let redis_config = RedisCacheConfig::new(
            "redis://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(60),
        );

        let cache_config = redis_config.to_cache_config();

        assert_eq!(
            cache_config.get("url"),
            Some(&"redis://localhost:6379".to_string())
        );
        assert_eq!(cache_config.get("key_prefix"), Some(&"test:".to_string()));
        assert_eq!(cache_config.get("default_ttl"), Some(&"60".to_string()));
    }
}
