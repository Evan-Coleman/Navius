use async_trait::async_trait;
use redis::{cmd, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Duration;

use navius_cache::{
    config::CacheConfig,
    operations::{Cache, CacheError, CacheResult},
};

use crate::{
    config::RedisCacheConfig,
    connection::RedisConnectionManager,
    error::{error_helpers, RedisCacheError, RedisCacheResult},
};

/// Redis cache implementation
#[derive(Clone)]
pub struct RedisCache {
    /// Connection manager
    connection_manager: RedisConnectionManager,
}

impl RedisCache {
    /// Create a new Redis cache
    pub async fn new(config: RedisCacheConfig) -> RedisCacheResult<Self> {
        let connection_manager = RedisConnectionManager::new(config).await?;

        Ok(Self { connection_manager })
    }

    /// Get the connection manager
    pub fn connection_manager(&self) -> &RedisConnectionManager {
        &self.connection_manager
    }

    /// Serialize a value to JSON
    fn serialize<T: Serialize>(&self, value: &T) -> RedisCacheResult<String> {
        serde_json::to_string(value).map_err(|e| {
            RedisCacheError::Serialization(format!("Failed to serialize value: {}", e))
        })
    }

    /// Deserialize a value from JSON
    fn deserialize<T: DeserializeOwned>(&self, data: &str) -> RedisCacheResult<T> {
        serde_json::from_str(data).map_err(|e| {
            RedisCacheError::Serialization(format!("Failed to deserialize value: {}", e))
        })
    }

    /// Get the prefixed key
    fn prefixed_key<K: AsRef<str>>(&self, key: K) -> String {
        self.connection_manager.prefixed_key(key)
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> CacheResult<Option<T>> {
        let prefixed_key = self.prefixed_key(key);

        let result: Option<String> = self
            .connection_manager
            .execute_command(&prefixed_key, "GET", |mut conn| {
                redis::cmd("GET").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            Some(data) => match self.deserialize(&data) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(CacheError::from(e)),
            },
            None => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let prefixed_key = self.prefixed_key(key);
        let serialized = self.serialize(value)?;

        match ttl {
            Some(ttl) => self
                .connection_manager
                .execute_command(&prefixed_key, "SETEX", |mut conn| {
                    redis::cmd("SETEX")
                        .arg(&prefixed_key)
                        .arg(ttl.as_secs())
                        .arg(&serialized)
                        .query(&mut conn)
                })
                .await
                .map_err(|e| CacheError::from(e)),
            None => self
                .connection_manager
                .execute_command(&prefixed_key, "SET", |mut conn| {
                    redis::cmd("SET")
                        .arg(&prefixed_key)
                        .arg(&serialized)
                        .query(&mut conn)
                })
                .await
                .map_err(|e| CacheError::from(e)),
        }
    }

    async fn delete(&self, key: &str) -> CacheResult<bool> {
        let prefixed_key = self.prefixed_key(key);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "DEL", |mut conn| {
                redis::cmd("DEL").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result > 0)
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let prefixed_key = self.prefixed_key(key);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "EXISTS", |mut conn| {
                redis::cmd("EXISTS").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result > 0)
    }

    async fn increment(&self, key: &str, amount: i64) -> CacheResult<i64> {
        let prefixed_key = self.prefixed_key(key);

        self.connection_manager
            .execute_command(&prefixed_key, "INCRBY", |mut conn| {
                redis::cmd("INCRBY")
                    .arg(&prefixed_key)
                    .arg(amount)
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))
    }

    async fn decrement(&self, key: &str, amount: i64) -> CacheResult<i64> {
        let prefixed_key = self.prefixed_key(key);

        self.connection_manager
            .execute_command(&prefixed_key, "DECRBY", |mut conn| {
                redis::cmd("DECRBY")
                    .arg(&prefixed_key)
                    .arg(amount)
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))
    }

    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
        let prefixed_key = self.prefixed_key(key);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "EXPIRE", |mut conn| {
                redis::cmd("EXPIRE")
                    .arg(&prefixed_key)
                    .arg(ttl.as_secs())
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result > 0)
    }

    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        let prefixed_key = self.prefixed_key(key);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "TTL", |mut conn| {
                redis::cmd("TTL").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            -2 => Ok(None), // Key does not exist
            -1 => Ok(None), // Key exists but has no associated expire
            ttl if ttl >= 0 => Ok(Some(Duration::from_secs(ttl as u64))),
            _ => Err(CacheError::Provider(format!(
                "Unexpected TTL result: {}",
                result
            ))),
        }
    }

    async fn clear(&self) -> CacheResult<()> {
        let pattern = self.prefixed_key("*");

        // First get all keys matching the pattern
        let keys: Vec<String> = self
            .connection_manager
            .execute_command(&pattern, "KEYS", |mut conn| {
                redis::cmd("KEYS").arg(&pattern).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        if keys.is_empty() {
            return Ok(());
        }

        // Then delete them all
        self.connection_manager
            .execute_command("multiple", "DEL", |mut conn| {
                redis::cmd("DEL").arg(keys).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(())
    }

    async fn health_check(&self) -> CacheResult<bool> {
        match self.connection_manager.ping().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use navius_cache::operations::Cache;

    #[tokio::test]
    async fn test_basic_operations() {
        let config = RedisCacheConfig::new(
            "redis://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(3600),
        );

        // Skip test if Redis is not available
        let cache = RedisCache::new(config).await;
        if cache.is_err() {
            println!("Skipping test_basic_operations - Redis not available");
            return;
        }

        let cache = cache.unwrap();

        // Clear any existing data
        let _ = cache.clear().await;

        // Test set and get
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct TestData {
            id: i32,
            name: String,
        }

        let data = TestData {
            id: 1,
            name: "Test".to_string(),
        };

        let key = "test_key";

        // Test set
        let set_result = cache.set(key, &data, None).await;
        assert!(set_result.is_ok(), "Failed to set: {:?}", set_result);

        // Test exists
        let exists_result = cache.exists(key).await;
        assert!(
            exists_result.is_ok(),
            "Failed to check exists: {:?}",
            exists_result
        );
        assert!(exists_result.unwrap(), "Key should exist");

        // Test get
        let get_result: CacheResult<Option<TestData>> = cache.get(key).await;
        assert!(get_result.is_ok(), "Failed to get: {:?}", get_result);
        assert_eq!(get_result.unwrap(), Some(data));

        // Test delete
        let delete_result = cache.delete(key).await;
        assert!(
            delete_result.is_ok(),
            "Failed to delete: {:?}",
            delete_result
        );
        assert!(delete_result.unwrap(), "Delete should return true");

        // Verify key is gone
        let exists_result = cache.exists(key).await;
        assert!(
            exists_result.is_ok(),
            "Failed to check exists: {:?}",
            exists_result
        );
        assert!(!exists_result.unwrap(), "Key should not exist after delete");
    }

    #[tokio::test]
    async fn test_ttl_operations() {
        let config = RedisCacheConfig::new(
            "redis://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(3600),
        );

        // Skip test if Redis is not available
        let cache = RedisCache::new(config).await;
        if cache.is_err() {
            println!("Skipping test_ttl_operations - Redis not available");
            return;
        }

        let cache = cache.unwrap();

        // Clear any existing data
        let _ = cache.clear().await;

        let key = "ttl_test";
        let ttl = Duration::from_secs(10);

        // Set with TTL
        let set_result = cache.set(key, &"value", Some(ttl)).await;
        assert!(
            set_result.is_ok(),
            "Failed to set with TTL: {:?}",
            set_result
        );

        // Check TTL
        let ttl_result = cache.ttl(key).await;
        assert!(ttl_result.is_ok(), "Failed to get TTL: {:?}", ttl_result);

        match ttl_result {
            Ok(Some(remaining)) => {
                assert!(
                    remaining <= ttl,
                    "TTL should be less than or equal to original"
                );
                assert!(
                    remaining > Duration::from_secs(0),
                    "TTL should be greater than 0"
                );
            }
            _ => panic!("Expected Some TTL duration"),
        }
    }
}
