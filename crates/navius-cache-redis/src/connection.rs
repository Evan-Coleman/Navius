use async_trait::async_trait;
use redis::{Client, Connection, ConnectionInfo, IntoConnectionInfo};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::RedisCacheConfig;
use crate::error::{error_helpers, RedisCacheError, RedisCacheResult};

/// Manager for Redis connections
#[derive(Clone)]
pub struct RedisConnectionManager {
    /// Redis client
    client: Arc<Client>,
    /// Configuration
    config: RedisCacheConfig,
    /// Connection pool mutex
    pool: Arc<Mutex<Vec<Connection>>>,
}

impl RedisConnectionManager {
    /// Create a new Redis connection manager
    pub async fn new(config: RedisCacheConfig) -> RedisCacheResult<Self> {
        let connection_info = match config.connection_string().into_connection_info() {
            Ok(info) => {
                let mut info = info;
                if let Some(pass) = &config.password {
                    match info {
                        ConnectionInfo { addr, .. } => {
                            info = ConnectionInfo {
                                addr,
                                redis: redis::RedisConnectionInfo {
                                    db: config.database,
                                    username: None,
                                    password: Some(pass.clone()),
                                },
                            };
                        }
                    }
                }
                info
            }
            Err(err) => {
                return Err(RedisCacheError::Configuration(format!(
                    "Invalid Redis connection URL: {}",
                    err
                )))
            }
        };

        let client = match Client::open(connection_info) {
            Ok(client) => client,
            Err(err) => {
                return Err(RedisCacheError::Configuration(format!(
                    "Failed to create Redis client: {}",
                    err
                )))
            }
        };

        // Test connection
        let connection = match client.get_connection() {
            Ok(conn) => conn,
            Err(err) => {
                return Err(RedisCacheError::Unavailable(format!(
                    "Failed to connect to Redis: {}",
                    err
                )))
            }
        };

        let manager = Self {
            client: Arc::new(client),
            config: config.clone(),
            pool: Arc::new(Mutex::new(vec![connection])),
        };

        Ok(manager)
    }

    /// Get a connection from the pool or create a new one
    pub async fn get_connection(&self) -> RedisCacheResult<Connection> {
        let mut pool = self.pool.lock().await;

        if let Some(conn) = pool.pop() {
            return Ok(conn);
        }

        match self.client.get_connection() {
            Ok(conn) => Ok(conn),
            Err(err) => Err(RedisCacheError::Unavailable(format!(
                "Failed to get Redis connection: {}",
                err
            ))),
        }
    }

    /// Return a connection to the pool
    pub async fn return_connection(&self, conn: Connection) {
        let mut pool = self.pool.lock().await;

        if pool.len() < self.config.max_connections as usize {
            pool.push(conn);
        }
    }

    /// Get the Redis client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the Redis configuration
    pub fn config(&self) -> &RedisCacheConfig {
        &self.config
    }

    /// Execute a function with a connection from the pool
    pub async fn with_connection<F, T>(&self, func: F) -> RedisCacheResult<T>
    where
        F: FnOnce(Connection) -> RedisCacheResult<T>,
    {
        let conn = self.get_connection().await?;

        let result = func(conn);

        // Only return the connection to the pool if the operation succeeded
        if result.is_ok() {
            if let Ok(conn) = self.client.get_connection() {
                self.return_connection(conn).await;
            }
        }

        result
    }

    /// Execute a Redis command and handle errors
    pub async fn execute_command<F, T>(
        &self,
        key: &str,
        operation: &str,
        func: F,
    ) -> RedisCacheResult<T>
    where
        F: FnOnce(Connection) -> Result<T, redis::RedisError>,
    {
        let conn = self.get_connection().await?;

        let result = func(conn);

        match result {
            Ok(value) => {
                // Return a connection to the pool only on successful operations
                if let Ok(conn) = self.client.get_connection() {
                    self.return_connection(conn).await;
                }
                Ok(value)
            }
            Err(err) => {
                // Don't return the connection on error
                error_helpers::handle_redis_result(Err(err), key, operation)
            }
        }
    }

    /// Create a prefixed key
    pub fn prefixed_key<K: AsRef<str>>(&self, key: K) -> String {
        self.config.prefixed_key(key)
    }

    /// Check if Redis is available
    pub async fn ping(&self) -> RedisCacheResult<()> {
        self.execute_command("ping", "PING", |mut conn| {
            redis::cmd("PING").query::<String>(&mut conn).map(|_| ())
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_connection_manager() {
        let config = RedisCacheConfig::new(
            "redis://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(3600),
        );

        // This test is designed to be skipped if Redis is not available
        let manager = RedisConnectionManager::new(config).await;
        if manager.is_err() {
            println!("Skipping test_connection_manager - Redis not available");
            return;
        }

        let manager = manager.unwrap();

        // Test prefixed key
        assert_eq!(manager.prefixed_key("user:123"), "test:user:123");

        // Test ping
        let ping_result = manager.ping().await;
        assert!(ping_result.is_ok(), "Ping failed: {:?}", ping_result);
    }
}
