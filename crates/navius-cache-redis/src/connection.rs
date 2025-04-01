use crate::metrics;
use crate::{
    config::RedisCacheConfig,
    error::{error_helpers, RedisCacheError, RedisCacheResult},
};
use futures::Future;
use futures::TryFutureExt;
use redis::aio::ConnectionLike;
use redis::aio::ConnectionManager;
use redis::aio::MultiplexedConnection;
use redis::{aio::Connection, AsyncCommands};
use redis::{Client, RedisError};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::Mutex, time::timeout};
use tracing::{debug, error, info, instrument, warn};

/// Statistics for connection pool monitoring
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total number of connections created
    pub total_connections_created: usize,
    /// Total number of connections closed
    pub total_connections_closed: usize,
    /// Total number of successful connection acquisitions
    pub total_acquires: usize,
    /// Total number of failed connection acquisitions
    pub total_acquire_failures: usize,
    /// Total number of timeouts during connection acquisition
    pub total_acquire_timeouts: usize,
    /// Current number of active connections
    pub current_active_connections: usize,
    /// Current number of idle connections in the pool
    pub current_idle_connections: usize,
}

/// Health status of a connection
#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionHealth {
    /// Connection is healthy
    Healthy,
    /// Connection is degraded (still usable but with issues)
    Degraded(String),
    /// Connection is unhealthy
    Unhealthy(String),
}

/// A connection with metadata for pool management
#[derive(Debug)]
struct PooledConnection {
    /// The actual Redis connection
    connection: Connection,
    /// When the connection was created
    created_at: Instant,
    /// When the connection was last used
    last_used: Instant,
    /// Number of times this connection has been used
    use_count: usize,
}

impl PooledConnection {
    /// Create a new pooled connection
    fn new(connection: Connection) -> Self {
        let now = Instant::now();
        Self {
            connection,
            created_at: now,
            last_used: now,
            use_count: 0,
        }
    }

    /// Update the last used timestamp
    fn mark_used(&mut self) {
        self.last_used = Instant::now();
        self.use_count += 1;
    }

    /// Check if the connection has expired based on max lifetime
    fn is_expired(&self, max_lifetime: Duration) -> bool {
        self.created_at.elapsed() > max_lifetime
    }

    /// Check if the connection has been idle for too long
    fn is_idle_timeout(&self, idle_timeout: Duration) -> bool {
        self.last_used.elapsed() > idle_timeout
    }
}

/// Redis connection manager for handling connection pooling
#[derive(Clone)]
pub struct RedisConnectionManager {
    client: redis::Client,
    pub command_timeout: Duration,
    key_prefix: Option<String>,
}

// Manual Debug implementation since redis::Client doesn't implement Debug
impl std::fmt::Debug for RedisConnectionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisConnectionManager")
            .field("command_timeout", &self.command_timeout)
            .field("key_prefix", &self.key_prefix)
            .field("client", &"<redis::Client>")
            .finish()
    }
}

impl RedisConnectionManager {
    #[instrument(skip(url), level = "debug")]
    pub fn new(
        url: &str,
        key_prefix: Option<String>,
        command_timeout: Duration,
    ) -> Result<Self, RedisCacheError> {
        let client = redis::Client::open(url)
            .map_err(|e| RedisCacheError::ConnectionError(e.to_string()))?;

        Ok(Self {
            client,
            key_prefix,
            command_timeout,
        })
    }

    pub async fn get_connection(&self) -> Result<MultiplexedConnection, RedisCacheError> {
        let conn = timeout(
            self.command_timeout,
            self.client.get_multiplexed_async_connection(),
        )
        .await
        .map_err(|_| RedisCacheError::Timeout("Connection timeout".to_string()))?
        .map_err(|e| RedisCacheError::ConnectionError(e.to_string()))?;

        Ok(conn)
    }

    pub async fn check_health(&self) -> Result<(), RedisCacheError> {
        let mut conn = self.get_connection().await?;
        let result: Result<String, redis::RedisError> = timeout(
            self.command_timeout,
            redis::cmd("PING").query_async(&mut conn),
        )
        .await
        .map_err(|_| RedisCacheError::Timeout("Health check timeout".to_string()))?;

        match result {
            Ok(response) if response == "PONG" => Ok(()),
            Ok(_) => Err(RedisCacheError::OperationError(
                "Invalid PING response".to_string(),
            )),
            Err(err) => Err(RedisCacheError::OperationError(err.to_string())),
        }
    }

    pub fn prefix_key(&self, key: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }

    pub fn get_command_timeout(&self) -> Duration {
        self.command_timeout
    }

    /// Execute a Redis command with timeout and metrics
    #[instrument(skip(self, operation_fn), level = "debug")]
    pub async fn execute_command<F, Fut, T>(
        &self,
        key: &str,
        operation: &str,
        operation_fn: F,
    ) -> RedisCacheResult<T>
    where
        F: FnOnce(MultiplexedConnection) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, RedisError>> + Send + 'static,
        T: Send + 'static,
    {
        let start = Instant::now();
        let conn = self.get_connection().await?;

        // Track metrics for this operation
        metrics::increment_operation_count(operation);

        let result = timeout(self.command_timeout, operation_fn(conn)).await;

        // Record operation duration
        let duration = start.elapsed();
        metrics::record_operation_duration(operation, duration);

        match result {
            Ok(Ok(value)) => {
                if duration > Duration::from_millis(100) {
                    warn!(
                        "Redis operation '{}' on key '{}' took {:?}",
                        operation, key, duration
                    );
                }
                Ok(value)
            }
            Ok(Err(e)) => {
                error!(
                    "Redis operation '{}' on key '{}' failed: {}",
                    operation, key, e
                );
                metrics::increment_error_count(operation);
                Err(RedisCacheError::OperationError(ToString::to_string(&e)))
            }
            Err(_) => {
                error!(
                    "Redis operation '{}' on key '{}' timed out after {:?}",
                    operation, key, self.command_timeout
                );
                metrics::increment_timeout_count(operation);
                Err(RedisCacheError::Timeout(ToString::to_string(&format!(
                    "Operation '{}' timed out",
                    operation
                ))))
            }
        }
    }

    /// Serialize a value to a Redis-compatible format
    pub async fn serialize<T>(&self, value: &T) -> RedisCacheResult<Vec<u8>>
    where
        T: Serialize + Send + Sync + 'static,
    {
        serde_json::to_vec(value).map_err(|e| RedisCacheError::SerializationError(e))
    }

    /// Deserialize a value from Redis
    pub async fn deserialize<T>(&self, data: &[u8]) -> RedisCacheResult<T>
    where
        T: DeserializeOwned + Send + 'static,
    {
        serde_json::from_slice(data).map_err(|e| RedisCacheError::DeserializationError(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisCacheConfig;
    use std::time::Duration;

    #[tokio::test]
    async fn test_connection_manager() {
        let config = RedisCacheConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            key_prefix: Some("test:".to_string()),
            command_timeout_seconds: 1,
            ..Default::default()
        };

        let manager = match RedisConnectionManager::new(
            &config.url,
            config.key_prefix,
            Duration::from_secs(config.command_timeout_seconds),
        )
        .await
        {
            Ok(manager) => manager,
            Err(_) => {
                println!("Skipping test_connection_manager - Redis not available");
                return;
            }
        };

        // Test ping
        manager.check_health().await.expect("Ping should succeed");
    }
}
