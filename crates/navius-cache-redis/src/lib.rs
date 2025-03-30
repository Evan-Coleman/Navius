/*!
Navius Cache Redis - Redis implementation for the Navius cache framework

This crate provides Redis connectivity for the Navius cache framework.
It implements the core cache interfaces defined in navius-cache.
*/

// Re-export redis for convenience
pub use redis;

// Internal modules
mod config;
mod connection;
mod error;
mod invalidation;
mod operations;

// Public exports
pub use config::RedisCacheConfig;
pub use connection::RedisConnectionManager;
pub use error::RedisCacheError;
pub use invalidation::RedisInvalidator;
pub use operations::RedisCache;

/// Redis provider for navius-cache
pub struct RedisProvider;

/// Implementation of the CacheProvider trait for Redis
impl navius_cache::CacheProvider for RedisProvider {
    fn name(&self) -> &'static str {
        "redis"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

/// Helper functions for common Redis operations
pub mod helpers {
    use crate::config::RedisCacheConfig;
    use crate::connection::RedisConnectionManager;
    use navius_cache::error::CacheResult;
    use redis::Client;

    /// Initialize a Redis connection with the given configuration
    ///
    /// This function will:
    /// 1. Create a Redis client from the configuration
    /// 2. Test the connection to ensure it's valid
    /// 3. Create a connection manager for the client
    ///
    /// # Arguments
    ///
    /// * `config` - The Redis cache configuration
    ///
    /// # Returns
    ///
    /// * `Ok(manager)` - The Redis connection manager
    /// * `Err(e)` - An error if any operation fails
    pub async fn initialize_redis(config: RedisCacheConfig) -> CacheResult<RedisConnectionManager> {
        // Create a Redis client from the configuration
        let client = match Client::open(config.connection_string()) {
            Ok(client) => client,
            Err(err) => {
                return Err(navius_cache::error::CacheError::ConnectionError(format!(
                    "Failed to create Redis client: {}",
                    err
                )));
            }
        };

        // Create a connection manager for the client
        let manager = RedisConnectionManager::new(client, config).await?;

        // Test the connection
        manager.health_check().await?;

        Ok(manager)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
