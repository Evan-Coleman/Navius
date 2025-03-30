use crate::{
    config::RedisCacheConfig,
    error::{RedisCacheError, RedisCacheResult, error_helpers},
};
use redis::{Client, Connection, RedisError, aio::ConnectionManager};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument};

/// Redis connection manager for handling connection pooling
#[derive(Clone)]
pub struct RedisConnectionManager {
    /// Redis client
    client: Arc<Client>,
    /// Cache configuration
    config: RedisCacheConfig,
    /// Connection pool
    pool: Arc<Mutex<Vec<Connection>>>,
}

impl RedisConnectionManager {
    /// Create a new Redis connection manager
    #[instrument(skip(config), level = "debug")]
    pub async fn new(config: RedisCacheConfig) -> RedisCacheResult<Self> {
        info!("Initializing Redis connection manager");

        let redis_url = match &config.password {
            Some(password) => format!(
                "redis://{}:{}@{}:{}/{}",
                "",
                password, // username is empty for Redis
                config.url.trim_start_matches("redis://"),
                config.database
            ),
            None => format!(
                "redis://{}/{}",
                config.url.trim_start_matches("redis://"),
                config.database
            ),
        };

        debug!("Connecting to Redis at: {}", redis_url);

        let client = match Client::open(redis_url) {
            Ok(client) => client,
            Err(err) => {
                error!("Failed to create Redis client: {}", err);
                return Err(RedisCacheError::ConnectionError(err.to_string()));
            }
        };

        // Test connection
        let connection = match client.get_connection() {
            Ok(conn) => conn,
            Err(err) => {
                error!("Failed to connect to Redis: {}", err);
                return Err(RedisCacheError::ConnectionError(format!(
                    "Failed to connect to Redis: {}",
                    err
                )));
            }
        };

        let manager = Self {
            client: Arc::new(client),
            config,
            pool: Arc::new(Mutex::new(vec![connection])),
        };

        debug!("Redis connection manager initialized");
        Ok(manager)
    }

    /// Get a connection from the pool or create a new one
    pub async fn get_connection(&self) -> RedisCacheResult<ConnectionManager> {
        debug!("Getting Redis connection manager");
        match self.client.get_async_connection_manager().await {
            Ok(conn) => Ok(conn),
            Err(err) => {
                error!("Failed to get Redis connection: {}", err);
                Err(RedisCacheError::ConnectionError(format!(
                    "Failed to get Redis connection: {}",
                    err
                )))
            }
        }
    }

    /// Execute a Redis command and handle errors
    pub async fn execute_command<F, T>(
        &self,
        key: &str,
        operation: &str,
        func: F,
    ) -> RedisCacheResult<T>
    where
        F: FnOnce(ConnectionManager) -> Result<T, RedisError>,
    {
        let conn = self.get_connection().await?;
        let result = func(conn);

        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                error!(
                    "Redis operation '{}' failed on key '{}': {}",
                    operation, key, err
                );
                Err(RedisCacheError::OperationError(format!(
                    "Redis operation '{}' failed: {}",
                    operation, err
                )))
            }
        }
    }

    /// Create a prefixed key
    pub fn prefixed_key<K: AsRef<str>>(&self, key: K) -> String {
        if self.config.key_prefix.is_empty() {
            key.as_ref().to_string()
        } else {
            format!("{}:{}", self.config.key_prefix, key.as_ref())
        }
    }

    /// Get the key prefix
    pub fn key_prefix(&self) -> &str {
        &self.config.key_prefix
    }

    /// Check if Redis is available
    pub async fn ping(&self) -> RedisCacheResult<()> {
        self.execute_command("ping", "PING", |mut conn| {
            redis::cmd("PING")
                .query_async(&mut conn)
                .map(|_: String| ())
        })
        .await
    }

    /// Get the Redis configuration
    pub fn config(&self) -> &RedisCacheConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_connection_manager() {
        let config = RedisCacheConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            key_prefix: "test".to_string(),
            default_ttl: Duration::from_secs(60),
            max_connections: 5,
            database: 0,
            password: None,
            use_tls: false,
            connection_timeout_seconds: 5,
            command_timeout_seconds: 2,
            retry_commands: true,
            max_retries: 3,
        };

        let manager = RedisConnectionManager::new(config.clone()).await;
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert_eq!(manager.key_prefix(), "test");
        assert_eq!(manager.prefixed_key("key"), "test:key");
        assert_eq!(manager.config().url, "redis://127.0.0.1:6379");
    }
}
