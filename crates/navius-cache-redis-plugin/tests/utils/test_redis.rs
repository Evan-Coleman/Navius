/// A utility module for Redis testing
///
/// This module provides tools to easily test Redis functionality:
/// 1. TestRedisServer - A simple wrapper for a Redis connection for tests
/// 2. RedisTestContext - A context for running Redis tests with isolated namespaces
use navius_cache::{Cache, CacheKey, CacheOperations};
use navius_cache_redis_plugin::{RedisCache, RedisCacheConfig, RedisError, RedisResult};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

/// Static counter to generate unique namespace prefixes for concurrent tests
static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// A simple wrapper around a Redis cache connection for testing
pub struct TestRedisServer {
    cache: RedisCache,
    pub prefix: String,
}

impl TestRedisServer {
    /// Create a new Redis test server
    ///
    /// # Arguments
    /// * `url` - The Redis URL to connect to (defaults to localhost:6379)
    /// * `prefix_override` - Optional custom prefix (if not provided, a unique prefix is generated)
    ///
    /// # Returns
    /// A TestRedisServer instance
    pub async fn new(
        url: Option<String>,
        prefix_override: Option<String>,
    ) -> Result<Self, RedisError> {
        // Generate a unique prefix to isolate test data
        let prefix = prefix_override.unwrap_or_else(|| {
            let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
            format!("test:{}:", count)
        });

        // Create Redis configuration
        let config = RedisCacheConfig::new(
            url.unwrap_or_else(|| "redis://localhost:6379".to_string()),
            prefix.clone(),
            Duration::from_secs(60),
        );

        // Create the Redis cache instance
        let cache = RedisCache::new(config).await?;

        Ok(Self { cache, prefix })
    }

    /// Provides direct access to the RedisCache instance
    pub fn cache(&self) -> &RedisCache {
        &self.cache
    }

    /// Clean up all test keys with the current prefix
    pub async fn cleanup(&self) -> RedisResult<()> {
        // Get all keys with the test prefix and delete them
        // Note: This uses a direct Redis command as it's a test utility
        let raw_conn = self.cache.raw_connection().await?;

        // Get all keys with the current prefix
        let pattern = format!("{}*", self.prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut raw_conn.clone())
            .await?;

        if !keys.is_empty() {
            // Delete all found keys
            redis::cmd("DEL")
                .arg(keys)
                .query_async(&mut raw_conn.clone())
                .await?;
        }

        Ok(())
    }

    /// Check if Redis server is available
    pub async fn is_available() -> bool {
        match TestRedisServer::new(None, None).await {
            Ok(server) => {
                if let Ok(_) = server.cache.health_check().await {
                    return true;
                }
                false
            }
            Err(_) => false,
        }
    }
}

/// A test context for Redis tests that ensures isolation and cleanup
pub struct RedisTestContext {
    server: TestRedisServer,
}

impl RedisTestContext {
    /// Create a new Redis test context with a unique namespace
    pub async fn new() -> Result<Self, RedisError> {
        let server = TestRedisServer::new(None, None).await?;
        Ok(Self { server })
    }

    /// Get the Redis cache instance
    pub fn cache(&self) -> &RedisCache {
        self.server.cache()
    }

    /// Get the prefix being used by this test context
    pub fn prefix(&self) -> &str {
        &self.server.prefix
    }
}

impl Drop for RedisTestContext {
    fn drop(&mut self) {
        // Run cleanup when the context is dropped
        let server = self.server.cache().clone();
        let prefix = self.server.prefix.clone();

        // Spawn a task to clean up keys in the background
        tokio::spawn(async move {
            // Get all keys with test prefix and delete them
            if let Ok(mut raw_conn) = server.raw_connection().await {
                let pattern = format!("{}*", prefix);
                if let Ok(keys) = redis::cmd("KEYS")
                    .arg(&pattern)
                    .query_async::<_, Vec<String>>(&mut raw_conn)
                    .await
                {
                    if !keys.is_empty() {
                        let _ = redis::cmd("DEL")
                            .arg(keys)
                            .query_async::<_, ()>(&mut raw_conn)
                            .await;
                    }
                }
            }
        });
    }
}

/// Helper macro for testing if Redis is available
/// This allows tests to be skipped if Redis is not running
#[macro_export]
macro_rules! skip_if_no_redis {
    () => {
        if !$crate::utils::test_redis::TestRedisServer::is_available().await {
            eprintln!("Redis server is not available, skipping test");
            return;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_test_redis_server_creation() {
        if !TestRedisServer::is_available().await {
            // Skip test if Redis is not available
            return;
        }

        // Test with default values
        let server1 = TestRedisServer::new(None, None).await.unwrap();
        assert!(server1.prefix.starts_with("test:"));

        // Test with custom prefix
        let prefix = "custom:test:".to_string();
        let server2 = TestRedisServer::new(None, Some(prefix.clone()))
            .await
            .unwrap();
        assert_eq!(server2.prefix, prefix);

        // Clean up
        let _ = server1.cleanup().await;
        let _ = server2.cleanup().await;
    }

    #[tokio::test]
    async fn test_redis_test_context() {
        if !TestRedisServer::is_available().await {
            // Skip test if Redis is not available
            return;
        }

        // Create context
        let context = RedisTestContext::new().await.unwrap();

        // Test basic cache operations
        let key = "test-key";
        let value = "test-value";

        context.cache().set(key, &value, None).await.unwrap();
        let result: Option<String> = context.cache().get(key).await.unwrap();

        assert_eq!(result, Some(value.to_string()));

        // Context will automatically clean up when dropped
    }
}
