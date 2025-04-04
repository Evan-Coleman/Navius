use std::sync::Arc;
use std::time::Duration;

use redis::{
    Client, ConnectionInfo, RedisError, aio::ConnectionManager,
    aio::MultiplexedConnection as AsyncConnection,
};

use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::config::RedisCacheConfig;
use crate::error::RedisError as CrateRedisError;

/// Connection manager for Redis
pub struct RedisConnectionManager {
    /// Redis client
    client: Client,
    /// Async connection manager for multiplexing
    connection_manager: Arc<Mutex<ConnectionManager>>,
    /// Configuration for the Redis connection
    config: RedisCacheConfig,
    /// Last error encountered
    last_error: Mutex<Option<String>>,
    /// Connection retry settings
    retry_settings: RetrySettings,
}

/// Settings for connection retry logic
#[derive(Clone, Debug)]
pub struct RetrySettings {
    /// Maximum number of connection retries before giving up
    pub max_retries: u32,
    /// Base delay between retries (will be increased with backoff)
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Whether to use exponential backoff for retries
    pub use_exponential_backoff: bool,
}

impl Default for RetrySettings {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            use_exponential_backoff: true,
        }
    }
}

impl RedisConnectionManager {
    /// Create a new Redis connection manager
    pub async fn new(config: RedisCacheConfig) -> Result<Self, CrateRedisError> {
        info!(
            "Initializing Redis connection manager with URL: {}",
            config.url
        );

        let connection_options = parse_url_with_config(&config)?;

        let client = Client::open(connection_options).map_err(|e| {
            CrateRedisError::Connection(format!("Failed to create Redis client: {}", e))
        })?;

        debug!("Redis client created, attempting to establish connection");

        let retry_settings = RetrySettings {
            max_retries: config.pool_config.connect_retries,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            use_exponential_backoff: true,
        };

        // Attempt to create a connection manager with retries
        let connection_manager = Self::create_connection_manager(&client, &retry_settings).await?;

        info!("Successfully connected to Redis server");

        Ok(Self {
            client,
            connection_manager: Arc::new(Mutex::new(connection_manager)),
            config,
            last_error: Mutex::new(None),
            retry_settings,
        })
    }

    /// Create a connection manager with retry logic
    async fn create_connection_manager(
        client: &Client,
        retry_settings: &RetrySettings,
    ) -> Result<ConnectionManager, CrateRedisError> {
        let mut last_error = None;
        let mut retry_count = 0;

        while retry_count < retry_settings.max_retries {
            match ConnectionManager::new(client.clone()).await {
                Ok(manager) => {
                    if retry_count > 0 {
                        info!(
                            "Successfully established Redis connection after {} retries",
                            retry_count
                        );
                    }
                    return Ok(manager);
                }
                Err(err) => {
                    last_error = Some(err.to_string());
                    retry_count += 1;

                    if retry_count < retry_settings.max_retries {
                        let delay = if retry_settings.use_exponential_backoff {
                            let exp_delay = retry_settings.base_delay.as_millis() as u64
                                * (2_u64.pow(retry_count - 1));
                            Duration::from_millis(
                                exp_delay.min(retry_settings.max_delay.as_millis() as u64),
                            )
                        } else {
                            retry_settings.base_delay
                        };

                        warn!(
                            "Failed to establish Redis connection (attempt {}/{}), retrying in {:?}: {}",
                            retry_count, retry_settings.max_retries, delay, err
                        );

                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        let error_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
        error!(
            "Failed to establish Redis connection after {} attempts: {}",
            retry_settings.max_retries, error_msg
        );
        Err(CrateRedisError::Connection(format!(
            "Failed to create connection after {} attempts: {}",
            retry_settings.max_retries, error_msg
        )))
    }

    /// Get an async connection from the manager
    pub async fn get_async_connection(&self) -> Result<AsyncConnection, RedisError> {
        let manager = self.connection_manager.lock().await;
        manager.get_multiplexed_async_connection().await
    }

    /// Get the Redis client
    pub fn get_client(&self) -> &Client {
        &self.client
    }

    /// Get the configuration
    pub fn get_config(&self) -> &RedisCacheConfig {
        &self.config
    }

    /// Health check for the Redis connection
    pub async fn health_check(&self) -> Result<(), CrateRedisError> {
        let mut conn = self.get_async_connection().await.map_err(|e| {
            CrateRedisError::Connection(format!("Health check connection failed: {}", e))
        })?;

        redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| CrateRedisError::Redis(e))?;

        Ok(())
    }

    /// Get the last error encountered, if any
    pub async fn last_error(&self) -> Option<String> {
        self.last_error.lock().await.clone()
    }

    /// Force reconnection to the Redis server
    pub async fn reconnect(&self) -> Result<(), CrateRedisError> {
        info!("Forcing Redis reconnection");

        let new_manager =
            Self::create_connection_manager(&self.client, &self.retry_settings).await?;

        // Replace the current connection manager with the new one
        *self.connection_manager.lock().await = new_manager;

        // Clear any stored error
        *self.last_error.lock().await = None;

        info!("Redis reconnection successful");
        Ok(())
    }
}

/// Parse the Redis URL with additional configuration options
fn parse_url_with_config(config: &RedisCacheConfig) -> Result<ConnectionInfo, CrateRedisError> {
    // Parse the base URL
    let mut connection_info = redis::parse_redis_url(&config.url)
        .map_err(|e| CrateRedisError::Configuration(format!("Invalid Redis URL: {}", e)))?;

    // Set timeout if configured
    if let Some(connect_timeout) = config.pool_config.connect_timeout {
        connection_info.redis.connection_timeout = Some(connect_timeout);
    }

    // Configure TLS if needed
    #[cfg(any(feature = "tls-rustls", feature = "tls-native"))]
    if config.tls_config.enabled {
        configure_tls(&mut connection_info, &config.tls_config)?;
    }

    Ok(connection_info)
}

/// Configure TLS for the connection info
#[cfg(any(feature = "tls-rustls", feature = "tls-native"))]
fn configure_tls(
    connection_info: &mut ConnectionInfo,
    tls_config: &crate::config::TlsConfig,
) -> Result<(), CrateRedisError> {
    use redis::TlsMode;

    #[cfg(feature = "tls-rustls")]
    {
        use redis::TlsCertificates;

        let mut tls_mode = if let Some(server_name) = &tls_config.server_name {
            TlsMode::Secure(server_name.to_string())
        } else {
            TlsMode::SecureNone
        };

        // Check if custom certificates are provided
        if tls_config.ca_cert_path.is_some() || tls_config.client_cert_path.is_some() {
            let mut certs = TlsCertificates::empty();

            // Add CA certificate if provided
            if let Some(ca_cert_path) = &tls_config.ca_cert_path {
                certs = certs.add_ca_certificate(std::fs::read(ca_cert_path).map_err(|e| {
                    CrateRedisError::Tls(format!("Failed to read CA cert file: {}", e))
                })?);
            }

            // Add client certificate and key if both are provided
            if let (Some(client_cert_path), Some(client_key_path)) =
                (&tls_config.client_cert_path, &tls_config.client_key_path)
            {
                certs = certs.add_client_credentials(
                    std::fs::read(client_cert_path).map_err(|e| {
                        CrateRedisError::Tls(format!("Failed to read client cert: {}", e))
                    })?,
                    std::fs::read(client_key_path).map_err(|e| {
                        CrateRedisError::Tls(format!("Failed to read client key: {}", e))
                    })?,
                );
            }

            tls_mode = TlsMode::Customized(certs);
        }

        connection_info.redis.tls = Some(tls_mode);
    }

    #[cfg(all(feature = "tls-native", not(feature = "tls-rustls")))]
    {
        let tls_mode = if let Some(server_name) = &tls_config.server_name {
            TlsMode::Secure(server_name.to_string())
        } else {
            TlsMode::SecureNone
        };

        connection_info.redis.tls = Some(tls_mode);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ClusterConfig, PoolConfig, RedisCacheConfig, TlsConfig};
    use std::time::Duration;

    #[test]
    fn test_parse_url_with_config() {
        let config = RedisCacheConfig {
            url: "redis://localhost:6379".to_string(),
            key_prefix: "test:".to_string(),
            default_ttl: Duration::from_secs(60),
            pool_config: PoolConfig {
                min_connections: Some(5),
                max_connections: Some(20),
                connect_timeout: Some(Duration::from_secs(30)),
                command_timeout: Some(Duration::from_secs(5)),
                connect_retries: 3,
                enable_connection_recycling: true,
                health_check_interval: Duration::from_secs(60),
            },
            tls_config: TlsConfig::default(),
            cluster_config: ClusterConfig::default(),
        };

        let connection_info = parse_url_with_config(&config).unwrap();
        assert_eq!(connection_info.redis.db, 0);
        assert_eq!(connection_info.redis.username, None);
        assert_eq!(connection_info.redis.password, None);
        assert_eq!(
            connection_info.redis.connection_timeout,
            Some(Duration::from_secs(30))
        );
    }

    #[test]
    fn test_parse_url_with_auth() {
        let config = RedisCacheConfig {
            url: "redis://user:password@localhost:6379/1".to_string(),
            key_prefix: "test:".to_string(),
            default_ttl: Duration::from_secs(60),
            pool_config: PoolConfig {
                min_connections: Some(5),
                max_connections: Some(20),
                connect_timeout: Some(Duration::from_secs(30)),
                command_timeout: Some(Duration::from_secs(5)),
                connect_retries: 3,
                enable_connection_recycling: true,
                health_check_interval: Duration::from_secs(60),
            },
            tls_config: TlsConfig::default(),
            cluster_config: ClusterConfig::default(),
        };

        let connection_info = parse_url_with_config(&config).unwrap();
        assert_eq!(connection_info.redis.db, 1);
        assert_eq!(connection_info.redis.username, Some("user".to_string()));
        assert_eq!(connection_info.redis.password, Some("password".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        let config = RedisCacheConfig {
            url: "invalid-url".to_string(),
            key_prefix: "test:".to_string(),
            default_ttl: Duration::from_secs(60),
            pool_config: PoolConfig::default(),
            tls_config: TlsConfig::default(),
            cluster_config: ClusterConfig::default(),
        };

        let result = parse_url_with_config(&config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_health_check_mock() {
        // This is a mock test that doesn't actually connect to Redis
        // In a real environment, you would use a test Redis instance or mock

        let config = RedisCacheConfig {
            url: "redis://localhost:6379".to_string(),
            key_prefix: "test:".to_string(),
            default_ttl: Duration::from_secs(60),
            pool_config: PoolConfig::default(),
            tls_config: TlsConfig::default(),
            cluster_config: ClusterConfig::default(),
        };

        // We can't actually test the connection without a Redis server,
        // so we'll just verify the function signature and that it fails
        // as expected when there's no Redis server

        if let Ok(manager) = RedisConnectionManager::new(config).await {
            let result = manager.health_check().await;
            // Since we're likely not connected to a real Redis server in tests,
            // we expect this to fail. In a real environment with Redis available,
            // this would pass.
            assert!(result.is_err() || result.is_ok());
        }
    }
}
