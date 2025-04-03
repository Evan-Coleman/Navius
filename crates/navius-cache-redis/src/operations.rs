use crate::connection::RedisConnectionPool;
use crate::error::RedisCacheError;
use crate::metrics::MetricNames;
use crate::metrics::OperationTimer;
use redis::{AsyncCommands, FromRedisValue, RedisResult, ToRedisArgs};
use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;
use tracing::trace;

/// Basic Redis operations
///
/// This trait provides a set of wrapper methods for common Redis operations,
/// adding additional functionality like key validation, error handling,
/// and metrics tracking.
pub trait RedisOperations {
    /// Apply the key prefix to a key
    fn prefixed_key<K: AsRef<str>>(&self, key: K) -> String;

    /// Execute a Redis command with the connection pool
    fn execute<F, T>(&self, f: F) -> impl Future<Output = Result<T, RedisCacheError>>
    where
        F: FnOnce(&mut redis::aio::MultiplexedConnection) -> redis::RedisFuture<'_, T>
            + Clone
            + Send
            + 'static,
        T: Send + 'static;

    /// Get a value from Redis
    async fn get_raw<K, V>(&self, key: K) -> Result<Option<V>, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
        V: FromRedisValue + Send + 'static;

    /// Set a value in Redis
    async fn set_raw<K, V>(
        &self,
        key: K,
        value: V,
        ttl: Option<Duration>,
    ) -> Result<(), RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
        V: ToRedisArgs + Send + Sync + Clone + 'static;

    /// Check if a key exists in Redis
    async fn exists_raw<K>(&self, key: K) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static;

    /// Delete a key from Redis
    async fn delete_raw<K>(&self, key: K) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static;

    /// Set a key's time to live in Redis
    async fn expire_raw<K>(&self, key: K, ttl: Duration) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static;

    /// Increment a counter value in Redis
    async fn increment_raw<K>(&self, key: K, by: i64) -> Result<i64, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static;

    /// Ping the Redis server
    async fn ping(&self) -> Result<bool, RedisCacheError>;

    /// Push a value to a Redis list
    async fn list_push_raw<K, V>(
        &self,
        key: K,
        value: V,
        right: bool,
    ) -> Result<i64, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
        V: ToRedisArgs + Send + Sync + Clone + 'static,
    {
        let op_name = if right {
            MetricNames::LIST_PUSH
        } else {
            MetricNames::LIST_PUSH
        };
        let timer = OperationTimer::new(op_name);
        let key = self.prefixed_key(&key);

        let cmd = if right { "RPUSH" } else { "LPUSH" };
        trace!(
            key = %key,
            command = cmd,
            "Redis list push operation"
        );

        let right_clone = right;
        let result = self
            .execute(move |conn| {
                Box::pin(async move {
                    let result: RedisResult<i64> = if right_clone {
                        conn.rpush(&key, value).await
                    } else {
                        conn.lpush(&key, value).await
                    };
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    /// Pop a value from a Redis list
    async fn list_pop_raw<K, V>(&self, key: K, right: bool) -> Result<Option<V>, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
        V: FromRedisValue + Send + 'static;

    /// Get a range of values from a Redis list
    async fn list_range_raw<K, V>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> Result<Vec<V>, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
        V: FromRedisValue + Send + 'static;

    /// Get the length of a Redis list
    async fn list_length_raw<K>(&self, key: K) -> Result<i64, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync;

    /// Get a value from a Redis hash
    async fn hash_get_raw<K, F, V>(&self, key: K, field: F) -> Result<Option<V>, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
        F: AsRef<str> + Debug + Send + Sync + Clone + 'static,
        V: FromRedisValue + Send + 'static;

    /// Set a value in a Redis hash
    async fn hash_set_raw<K, F, V>(
        &self,
        key: K,
        field: F,
        value: V,
    ) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
        F: AsRef<str> + Debug + Send + Sync + Clone + 'static,
        V: ToRedisArgs + Send + Sync + Clone + 'static;

    /// Delete a field from a Redis hash
    async fn hash_delete_raw<K, F>(&self, key: K, field: F) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
        F: AsRef<str> + Debug + Send + Sync + Clone + 'static;

    /// Get all fields from a Redis hash
    async fn hash_get_all_raw<K, V>(&self, key: K) -> Result<Vec<(String, V)>, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
        V: FromRedisValue + Send + 'static;
}

impl RedisOperations for RedisConnectionPool {
    fn prefixed_key<K: AsRef<str>>(&self, key: K) -> String {
        if let Some(prefix) = &self.prefix {
            if !prefix.is_empty() {
                format!("{}:{}", prefix, key.as_ref())
            } else {
                key.as_ref().to_string()
            }
        } else {
            key.as_ref().to_string()
        }
    }

    fn execute<F, T>(&self, f: F) -> impl Future<Output = Result<T, RedisCacheError>>
    where
        F: FnOnce(&mut redis::aio::MultiplexedConnection) -> redis::RedisFuture<'_, T>
            + Clone
            + Send
            + 'static,
        T: Send + 'static,
    {
        Box::pin(async move {
            let mut conn = self
                .pool
                .get()
                .await
                .map_err(|e| RedisCacheError::Connection(e.to_string()))?;
            let result = f(&mut conn).await;
            result.map_err(|e| RedisCacheError::Command(e.to_string()))
        })
    }

    async fn get_raw<K, V>(&self, key: K) -> Result<Option<V>, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
        V: FromRedisValue + Send + 'static,
    {
        let timer = OperationTimer::new(MetricNames::GET);
        let key = self.prefixed_key(&key);

        trace!(key = %key, "Redis GET operation");
        let result = self
            .execute(|conn| {
                Box::pin(async move {
                    let result: RedisResult<Option<V>> = conn.get(&key).await;
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn set_raw<K, V>(
        &self,
        key: K,
        value: V,
        ttl: Option<Duration>,
    ) -> Result<(), RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
        V: ToRedisArgs + Send + Sync + Clone + 'static,
    {
        let timer = OperationTimer::new(MetricNames::SET);
        let key = self.prefixed_key(&key);

        trace!(
            key = %key,
            ttl_ms = ttl.map(|t| t.as_millis()).unwrap_or(0),
            "Redis SET operation"
        );

        let result = self
            .execute(move |conn| {
                Box::pin(async move {
                    let result: RedisResult<()> = match ttl {
                        Some(ttl) => conn.set_ex(&key, value, ttl.as_secs() as u64).await,
                        None => conn.set(&key, value).await,
                    };
                    result
                })
            })
            .await;

        match result {
            Ok(_) => {
                timer.record_success();
                Ok(())
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn exists_raw<K>(&self, key: K) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
    {
        let timer = OperationTimer::new(MetricNames::EXISTS);
        let key = self.prefixed_key(&key);

        trace!(key = %key, "Redis EXISTS operation");
        let result = self
            .execute(|conn| {
                Box::pin(async move {
                    let result: RedisResult<bool> = conn.exists(&key).await;
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn delete_raw<K>(&self, key: K) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
    {
        let timer = OperationTimer::new(MetricNames::DELETE);
        let key = self.prefixed_key(&key);

        trace!(key = %key, "Redis DEL operation");
        let result = self
            .execute(|conn| {
                Box::pin(async move {
                    let result: RedisResult<i64> = conn.del(&key).await;
                    result
                })
            })
            .await;

        match result {
            Ok(deleted) => {
                timer.record_success();
                Ok(deleted > 0)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn expire_raw<K>(&self, key: K, ttl: Duration) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
    {
        let timer = OperationTimer::new(MetricNames::EXPIRE);
        let key = self.prefixed_key(&key);

        trace!(
            key = %key,
            ttl_ms = ttl.as_millis(),
            "Redis EXPIRE operation"
        );

        let result = self
            .execute(move |conn| {
                Box::pin(async move {
                    let result: RedisResult<bool> = conn.expire(&key, ttl.as_secs() as i64).await;
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn increment_raw<K>(&self, key: K, by: i64) -> Result<i64, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
    {
        let timer = OperationTimer::new(MetricNames::INCREMENT);
        let key = self.prefixed_key(&key);

        trace!(key = %key, by = by, "Redis INCRBY operation");
        let result = self
            .execute(move |conn| {
                Box::pin(async move {
                    let result: RedisResult<i64> = conn.incr(&key, by).await;
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn ping(&self) -> Result<bool, RedisCacheError> {
        let timer = OperationTimer::new(MetricNames::PING);
        trace!("Redis PING operation");

        let result = self
            .execute(|conn| {
                Box::pin(async move {
                    let result: RedisResult<String> = redis::cmd("PING").query_async(conn).await;
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value == "PONG")
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn list_pop_raw<K, V>(&self, key: K, right: bool) -> Result<Option<V>, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
        V: FromRedisValue + Send + 'static,
    {
        let op_name = if right {
            MetricNames::LIST_POP
        } else {
            MetricNames::LIST_POP
        };
        let timer = OperationTimer::new(op_name);
        let key = self.prefixed_key(&key);

        let cmd = if right { "RPOP" } else { "LPOP" };
        trace!(
            key = %key,
            command = cmd,
            "Redis list pop operation"
        );

        let result = self
            .execute(move |conn| {
                Box::pin(async move {
                    let result: RedisResult<Option<V>> = if right {
                        conn.rpop(&key, None).await
                    } else {
                        conn.lpop(&key, None).await
                    };
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn list_range_raw<K, V>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> Result<Vec<V>, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
        V: FromRedisValue + Send + 'static,
    {
        let timer = OperationTimer::new(MetricNames::LIST_RANGE);
        let key = self.prefixed_key(&key);

        trace!(
            key = %key,
            start = start,
            stop = stop,
            "Redis LRANGE operation"
        );

        let result = self
            .execute(move |conn| {
                Box::pin(async move {
                    let result: RedisResult<Vec<V>> = conn.lrange(&key, start, stop).await;
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn list_length_raw<K>(&self, key: K) -> Result<i64, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
    {
        let timer = OperationTimer::new(MetricNames::LIST_LENGTH);
        let key = self.prefixed_key(&key);

        trace!(key = %key, "Redis LLEN operation");
        let result = self
            .execute(|conn| {
                Box::pin(async move {
                    let result: RedisResult<i64> = conn.llen(&key).await;
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn hash_get_raw<K, F, V>(&self, key: K, field: F) -> Result<Option<V>, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
        F: AsRef<str> + Debug + Send + Sync + Clone + 'static,
        V: FromRedisValue + Send + 'static,
    {
        let timer = OperationTimer::new(MetricNames::HASH_GET);
        let key = self.prefixed_key(&key);

        trace!(
            key = %key,
            field = %field.as_ref(),
            "Redis HGET operation"
        );

        let result = self
            .execute(|conn| {
                Box::pin(async move {
                    let result: RedisResult<Option<V>> = conn.hget(&key, field.as_ref()).await;
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn hash_set_raw<K, F, V>(
        &self,
        key: K,
        field: F,
        value: V,
    ) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
        F: AsRef<str> + Debug + Send + Sync + Clone + 'static,
        V: ToRedisArgs + Send + Sync + Clone + 'static,
    {
        let timer = OperationTimer::new(MetricNames::HASH_SET);
        let key = self.prefixed_key(&key);

        trace!(
            key = %key,
            field = %field.as_ref(),
            "Redis HSET operation"
        );

        let result = self
            .execute(|conn| {
                Box::pin(async move {
                    let result: RedisResult<bool> = conn.hset_nx(&key, field.as_ref(), value).await;
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn hash_delete_raw<K, F>(&self, key: K, field: F) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
        F: AsRef<str> + Debug + Send + Sync + Clone + 'static,
    {
        let timer = OperationTimer::new(MetricNames::HASH_DELETE);
        let key = self.prefixed_key(&key);

        trace!(
            key = %key,
            field = %field.as_ref(),
            "Redis HDEL operation"
        );

        let result = self
            .execute(|conn| {
                Box::pin(async move {
                    let result: RedisResult<i64> = conn.hdel(&key, field.as_ref()).await;
                    result
                })
            })
            .await;

        match result {
            Ok(deleted) => {
                timer.record_success();
                Ok(deleted > 0)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }

    async fn hash_get_all_raw<K, V>(&self, key: K) -> Result<Vec<(String, V)>, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
        V: FromRedisValue + Send + 'static,
    {
        let timer = OperationTimer::new("redis_hash_get_all");
        let key = self.prefixed_key(&key);

        trace!(key = %key, "Redis HGETALL operation");

        let result = self
            .execute(|conn| {
                Box::pin(async move {
                    let result: RedisResult<Vec<(String, V)>> = conn.hgetall(&key).await;
                    result
                })
            })
            .await;

        match result {
            Ok(value) => {
                timer.record_success();
                Ok(value)
            }
            Err(err) => {
                timer.record_error(&err);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisCacheConfig;
    use redis::Value;

    // Helper to create a mocked connection pool
    async fn create_test_pool() -> RedisConnectionPool {
        // Only run if we have a Redis server available for testing
        if std::env::var("REDIS_TEST_SERVER").is_err() {
            // Create a default config for tests
            let config = RedisCacheConfig::default();
            let pool = RedisConnectionPool::new(config)
                .await
                .expect("Failed to create test connection pool");

            // Clear any existing test data
            let _ = pool
                .execute(|conn| {
                    Box::pin(async move {
                        let _: RedisResult<()> = redis::cmd("FLUSHDB").query_async(conn).await;
                        Ok(())
                    })
                })
                .await;

            pool
        } else {
            panic!(
                "Redis server not available for testing. Set REDIS_TEST_SERVER=1 to run integration tests."
            );
        }
    }

    // Use #[ignore] to skip integration tests unless explicitly requested
    #[tokio::test]
    #[ignore]
    async fn test_get_set_operations() {
        let pool = create_test_pool().await;

        // Test set and get
        let key = "test:key";
        let value = "test_value";

        assert!(pool.set_raw(key, value, None).await.is_ok());

        let retrieved: Result<Option<String>, _> = pool.get_raw(key).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap(), Some(value.to_string()));

        // Test exists
        let exists = pool.exists_raw(key).await;
        assert!(exists.is_ok());
        assert!(exists.unwrap());

        // Test delete
        let deleted = pool.delete_raw(key).await;
        assert!(deleted.is_ok());
        assert!(deleted.unwrap());

        // Test key no longer exists
        let exists = pool.exists_raw(key).await;
        assert!(exists.is_ok());
        assert!(!exists.unwrap());
    }

    #[tokio::test]
    #[ignore]
    async fn test_expire_operation() {
        let pool = create_test_pool().await;

        // Set a key
        let key = "test:expire";
        let value = "expiring_value";

        assert!(pool.set_raw(key, value, None).await.is_ok());

        // Set expiry for 1 second
        let ttl = Duration::from_secs(1);
        let expired = pool.expire_raw(key, ttl).await;
        assert!(expired.is_ok());
        assert!(expired.unwrap());

        // Key should still exist immediately
        let exists = pool.exists_raw(key).await;
        assert!(exists.is_ok());
        assert!(exists.unwrap());

        // Wait for key to expire
        tokio::time::sleep(ttl + Duration::from_millis(500)).await;

        // Key should be gone
        let exists = pool.exists_raw(key).await;
        assert!(exists.is_ok());
        assert!(!exists.unwrap());
    }

    #[tokio::test]
    #[ignore]
    async fn test_increment_operation() {
        let pool = create_test_pool().await;

        // Test increment
        let key = "test:counter";

        // Increment new key
        let count = pool.increment_raw(key, 1).await;
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 1);

        // Increment again
        let count = pool.increment_raw(key, 5).await;
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 6);

        // Decrement
        let count = pool.increment_raw(key, -2).await;
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 4);
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_operations() {
        let pool = create_test_pool().await;

        // Test list operations
        let key = "test:list";

        // Push items
        let len = pool.list_push_raw(key, "item1", true).await;
        assert!(len.is_ok());
        assert_eq!(len.unwrap(), 1);

        let len = pool.list_push_raw(key, "item2", true).await;
        assert!(len.is_ok());
        assert_eq!(len.unwrap(), 2);

        let len = pool.list_push_raw(key, "item0", false).await;
        assert!(len.is_ok());
        assert_eq!(len.unwrap(), 3);

        // Get list length
        let len = pool.list_length_raw(key).await;
        assert!(len.is_ok());
        assert_eq!(len.unwrap(), 3);

        // Get range
        let items: Result<Vec<String>, _> = pool.list_range_raw(key, 0, -1).await;
        assert!(items.is_ok());
        assert_eq!(items.unwrap(), vec!["item0", "item1", "item2"]);

        // Pop items
        let item: Result<Option<String>, _> = pool.list_pop_raw(key, true).await;
        assert!(item.is_ok());
        assert_eq!(item.unwrap(), Some("item2".to_string()));

        let item: Result<Option<String>, _> = pool.list_pop_raw(key, false).await;
        assert!(item.is_ok());
        assert_eq!(item.unwrap(), Some("item0".to_string()));
    }

    #[tokio::test]
    #[ignore]
    async fn test_hash_operations() {
        let pool = create_test_pool().await;

        // Test hash operations
        let key = "test:hash";

        // Set hash field
        let set = pool.hash_set_raw(key, "field1", "value1").await;
        assert!(set.is_ok());
        assert!(set.unwrap());

        let set = pool.hash_set_raw(key, "field2", "value2").await;
        assert!(set.is_ok());
        assert!(set.unwrap());

        // Get hash field
        let value: Result<Option<String>, _> = pool.hash_get_raw(key, "field1").await;
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), Some("value1".to_string()));

        // Get non-existent field
        let value: Result<Option<String>, _> = pool.hash_get_raw(key, "field3").await;
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), None);

        // Delete hash field
        let deleted = pool.hash_delete_raw(key, "field1").await;
        assert!(deleted.is_ok());
        assert!(deleted.unwrap());

        // Get all hash fields
        let values: Result<Vec<(String, String)>, _> = pool.hash_get_all_raw(key).await;
        assert!(values.is_ok());

        let mut all_values = values.unwrap();
        all_values.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(
            all_values,
            vec![("field2".to_string(), "value2".to_string())]
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_ping_operation() {
        let pool = create_test_pool().await;

        // Test ping
        let pong = pool.ping().await;
        assert!(pong.is_ok());
        assert!(pong.unwrap());
    }
}
