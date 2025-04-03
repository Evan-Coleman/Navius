use crate::config::RedisCacheConfig;
use crate::error::{RedisCacheError, RedisCacheResult};
use bb8::Pool;
use bb8_redis::{RedisConnectionManager, bb8::PooledConnection};
use redis::aio::MultiplexedConnection;
use redis::{Client, RedisError};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, instrument, warn};

/// Statistics for connection pool monitoring
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total number of connections created
    pub total_connections_created: usize,
    /// Total number of connections closed
    pub total_connections_closed: usize,
    /// Current size of the pool
    pub current_size: usize,
    /// Number of connections currently in use
    pub active_connections: usize,
    /// Number of idle connections
    pub idle_connections: usize,
}

/// Connection pool for Redis
#[derive(Clone)]
pub struct RedisConnectionPool {
    /// Connection configuration
    config: RedisCacheConfig,
    /// Internal connection pool using bb8
    pub(crate) pool: bb8::Pool<RedisConnectionManager>,
    /// Redis URL
    url: String,
    /// Optional key prefix
    pub(crate) prefix: Option<String>,
}

impl RedisConnectionPool {
    /// Create a new connection pool with the given configuration
    pub async fn new(config: RedisCacheConfig) -> RedisCacheResult<Self> {
        let url = config.url.clone();

        // Validate configuration
        config.validate()?;

        // Create connection manager
        let manager = RedisConnectionManager::new(url.clone()).map_err(|e| {
            RedisCacheError::Connection(format!("Failed to create Redis connection manager: {}", e))
        })?;

        // Create connection pool
        let pool = Pool::builder()
            .max_size(config.max_connections as u32)
            .connection_timeout(config.connection_timeout)
            .build(manager)
            .await
            .map_err(|e| {
                RedisCacheError::Connection(format!(
                    "Failed to create Redis connection pool: {}",
                    e
                ))
            })?;

        // Test the connection
        let pool_clone = pool.clone();
        let _conn = timeout(config.connection_timeout, pool_clone.get())
            .await
            .map_err(|_| {
                RedisCacheError::Timeout(
                    "Connection pool timed out during initialization".to_string(),
                )
            })?
            .map_err(|e| {
                RedisCacheError::Connection(format!(
                    "Failed to get Redis connection during initialization: {}",
                    e
                ))
            })?;

        debug!("Redis connection pool initialized successfully");

        let config_clone = config.clone();
        Ok(Self {
            config,
            pool,
            url,
            prefix: config_clone.key_prefix,
        })
    }

    /// Get a connection from the pool
    #[instrument(skip(self), level = "debug")]
    pub async fn get_connection(
        &self,
    ) -> RedisCacheResult<PooledConnection<'_, RedisConnectionManager>> {
        let conn = timeout(self.config.connection_timeout, self.pool.get())
            .await
            .map_err(|_| RedisCacheError::Timeout("Connection pool timed out".to_string()))?
            .map_err(|e| {
                RedisCacheError::Connection(format!("Failed to get Redis connection: {}", e))
            })?;

        Ok(conn)
    }

    /// Health check for the Redis connection
    pub async fn health_check(&self) -> RedisCacheResult<()> {
        let mut conn = self.get_connection().await?;

        // Use proper query_async with correct connection type
        let ping_result: redis::RedisResult<String> = timeout(
            self.config.command_timeout,
            redis::cmd("PING").query_async(&mut *conn),
        )
        .await
        .map_err(|_| RedisCacheError::Timeout("Health check timed out".to_string()))?;

        match ping_result {
            Ok(response) if response == "PONG" => Ok(()),
            Ok(response) => Err(RedisCacheError::Command(format!(
                "Health check failed: unexpected response '{}'",
                response
            ))),
            Err(e) => Err(RedisCacheError::Command(format!(
                "Health check failed: {}",
                e
            ))),
        }
    }

    /// Raw PING command that returns the response string
    pub async fn ping_raw(&self) -> RedisCacheResult<String> {
        let mut conn = self.get_connection().await?;

        // Use proper query_async with correct connection type
        let response: redis::RedisResult<String> = timeout(
            self.config.command_timeout,
            redis::cmd("PING").query_async(&mut *conn),
        )
        .await
        .map_err(|_| RedisCacheError::Timeout("Ping timed out".to_string()))?;

        response.map_err(|e| RedisCacheError::Command(format!("Ping failed: {}", e)))
    }

    /// Get connection pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        let state = self.pool.state();

        PoolStats {
            // Use the fields that actually exist in bb8::State
            total_connections_created: state.connections as usize,
            total_connections_closed: 0, // Not provided by bb8
            current_size: state.connections as usize,
            active_connections: (state.connections - state.idle_connections) as usize,
            idle_connections: state.idle_connections as usize,
        }
    }

    /// Execute a Redis command with timeout and retry logic
    pub async fn execute<F, T>(&self, f: F) -> RedisCacheResult<T>
    where
        F: FnOnce(&mut redis::aio::MultiplexedConnection) -> redis::RedisFuture<'_, T>
            + Clone
            + Send
            + 'static,
        T: Send + 'static,
    {
        let mut attempts = 0;
        let max_retries = self.config.max_retries;
        let retry_enabled = self.config.retry_commands;

        loop {
            let mut conn = self.get_connection().await?;
            let f = f.clone(); // Clone the function for each retry attempt

            let result = timeout(self.config.command_timeout, f(&mut *conn)).await;

            match result {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(err)) => {
                    if retry_enabled && Self::should_retry(&err) && attempts < max_retries {
                        attempts += 1;
                        warn!(
                            "Redis command failed, retrying ({}/{})...: {}",
                            attempts, max_retries, err
                        );
                        continue;
                    }
                    return Err(RedisCacheError::from(err));
                }
                Err(_) => {
                    if retry_enabled && attempts < max_retries {
                        attempts += 1;
                        warn!(
                            "Redis command timed out, retrying ({}/{})...",
                            attempts, max_retries
                        );
                        continue;
                    }
                    return Err(RedisCacheError::Timeout(
                        "Redis command timed out".to_string(),
                    ));
                }
            }
        }
    }

    /// Get the Redis connection configuration
    pub fn config(&self) -> &RedisCacheConfig {
        &self.config
    }

    /// Get the Redis URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Apply the key prefix to a key
    pub fn prefixed_key<K: AsRef<str>>(&self, key: K) -> String {
        match &self.prefix {
            Some(prefix) => format!("{}{}", prefix, key.as_ref()),
            None => key.as_ref().to_string(),
        }
    }

    /// Check if a Redis error is retriable
    fn should_retry(err: &RedisError) -> bool {
        err.is_io_error() || err.is_timeout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    #[ignore]
    async fn test_connection_pool() {
        let config = RedisCacheConfig::default();

        // Try to create a pool
        match RedisConnectionPool::new(config).await {
            Ok(pool) => {
                // If Redis is available, test the connection
                let result = pool.health_check().await;
                assert!(
                    result.is_ok(),
                    "Health check should succeed when Redis is available"
                );

                // Test get_stats
                let stats = pool.get_stats().await;
                assert!(
                    stats.current_size > 0,
                    "Pool should have at least one connection"
                );
            }
            Err(err) => {
                // It's okay if Redis is not available for testing
                println!("Redis not available for testing: {}", err);
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_invalid_config() {
        // Test with empty URL
        let config = RedisCacheConfig {
            url: "".to_string(),
            ..RedisCacheConfig::default()
        };
        let result = RedisConnectionPool::new(config).await;
        assert!(result.is_err(), "Pool creation should fail with empty URL");

        // Test with zero max connections
        let config = RedisCacheConfig {
            max_connections: 0,
            ..RedisCacheConfig::default()
        };
        let result = RedisConnectionPool::new(config).await;
        assert!(
            result.is_err(),
            "Pool creation should fail with zero max connections"
        );

        // Test with zero connection timeout
        let config = RedisCacheConfig {
            connection_timeout: Duration::from_secs(0),
            ..RedisCacheConfig::default()
        };
        let result = RedisConnectionPool::new(config).await;
        assert!(
            result.is_err(),
            "Pool creation should fail with zero connection timeout"
        );
    }
}
