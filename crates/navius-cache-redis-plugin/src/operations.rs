use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use redis::{AsyncCommands, Pipeline};
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::Mutex;

use navius_cache::{Cache, CacheKey, CacheOperations, CacheOptions, CacheResult};

use crate::config::RedisCacheConfig;
use crate::connection::RedisConnectionManager;
use crate::error::{RedisError, translate_redis_error};
use crate::util;

/// Redis implementation of the navius-cache Cache interface
#[derive(Clone)]
pub struct RedisCache {
    /// Redis connection manager
    connection_manager: Arc<RedisConnectionManager>,
    /// Configuration
    config: RedisCacheConfig,
}

impl RedisCache {
    /// Create a new Redis cache with the given configuration
    pub async fn new(config: RedisCacheConfig) -> Result<Self, RedisError> {
        let connection_manager = RedisConnectionManager::new(config.clone()).await?;

        Ok(Self {
            connection_manager: Arc::new(connection_manager),
            config,
        })
    }

    /// Get the Redis connection manager
    pub fn connection_manager(&self) -> &Arc<RedisConnectionManager> {
        &self.connection_manager
    }

    /// Get the configuration
    pub fn config(&self) -> &RedisCacheConfig {
        &self.config
    }

    /// Format a key with the configured prefix
    fn format_key<K>(&self, key: K) -> String
    where
        K: CacheKey,
    {
        util::format_key(&self.config.key_prefix, &key.to_string())
    }

    /// Get TTL from options or use default
    fn get_ttl(&self, options: Option<&CacheOptions>) -> Duration {
        options
            .and_then(|opts| opts.ttl)
            .unwrap_or(self.config.default_ttl)
    }

    /// Create a pipeline
    fn create_pipeline(&self) -> Pipeline {
        redis::pipe()
    }

    /// Execute a pipeline
    async fn execute_pipeline(&self, pipeline: Pipeline) -> Result<(), RedisError> {
        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| {
                RedisError::Connection(format!("Failed to get Redis connection: {}", e))
            })?;

        pipeline
            .query_async(&mut conn)
            .await
            .map_err(|e| RedisError::Operation(format!("Failed to execute pipeline: {}", e)))?;

        Ok(())
    }

    /// Get a raw Redis connection for direct Redis commands
    pub async fn raw_connection(&self) -> Result<redis::aio::Connection, RedisError> {
        self.connection_manager
            .get_async_connection()
            .await
            .map_err(|e| RedisError::Connection(format!("Failed to get Redis connection: {}", e)))
    }

    /// Perform a health check on the Redis connection
    pub async fn health_check(&self) -> Result<(), RedisError> {
        self.connection_manager.health_check().await
    }
}

#[async_trait]
impl CacheOperations for RedisCache {
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        let prefixed_key = self.format_key(key);

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let result: Option<String> = conn
            .get(&prefixed_key)
            .await
            .map_err(|e| translate_redis_error(e))?;

        match result {
            Some(data) => serde_json::from_str(&data)
                .map(Some)
                .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string())),
            None => Ok(None),
        }
    }

    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let prefixed_keys: Vec<String> = keys.into_iter().map(|k| self.format_key(k)).collect();

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let results: Vec<Option<String>> = conn
            .get(prefixed_keys)
            .await
            .map_err(|e| translate_redis_error(e))?;

        let mut parsed_results = Vec::with_capacity(results.len());

        for result in results {
            match result {
                Some(data) => {
                    let parsed = serde_json::from_str(&data).map_err(|e| {
                        navius_cache::error::CacheError::SerializationError(e.to_string())
                    })?;
                    parsed_results.push(Some(parsed));
                }
                None => parsed_results.push(None),
            }
        }

        Ok(parsed_results)
    }

    async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        let prefixed_key = self.format_key(key);
        let serialized = serde_json::to_string(value)
            .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;

        let ttl = self.get_ttl(options.as_ref());
        let ttl_seconds = util::format_ttl(ttl);

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        conn.set_ex(prefixed_key, serialized, ttl_seconds as u64)
            .await
            .map_err(|e| translate_redis_error(e))?;

        Ok(())
    }

    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<CacheOptions>,
    ) -> CacheResult<()>
    where
        K: CacheKey + Eq + std::hash::Hash + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        if entries.is_empty() {
            return Ok(());
        }

        let ttl = self.get_ttl(options.as_ref());
        let ttl_seconds = util::format_ttl(ttl);

        let mut pipe = self.create_pipeline();

        for (key, value) in entries {
            let prefixed_key = self.format_key(key);
            let serialized = serde_json::to_string(&value)
                .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;

            pipe.set_ex(prefixed_key, serialized, ttl_seconds as u64);
        }

        self.execute_pipeline(pipe).await.map_err(|e| e.into())
    }

    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        let prefixed_key = self.format_key(key);

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let result: i64 = conn
            .del(prefixed_key)
            .await
            .map_err(|e| translate_redis_error(e))?;

        Ok(result > 0)
    }

    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        if keys.is_empty() {
            return Ok(0);
        }

        let prefixed_keys: Vec<String> = keys.into_iter().map(|k| self.format_key(k)).collect();

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let result: i64 = conn
            .del(prefixed_keys)
            .await
            .map_err(|e| translate_redis_error(e))?;

        Ok(result as usize)
    }

    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        let prefixed_key = self.format_key(key);

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let result: bool = conn
            .exists(prefixed_key)
            .await
            .map_err(|e| translate_redis_error(e))?;

        Ok(result)
    }

    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        let prefixed_key = self.format_key(key);

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let result: i64 = conn
            .incr(prefixed_key, amount)
            .await
            .map_err(|e| translate_redis_error(e))?;

        Ok(result)
    }

    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        let prefixed_key = self.format_key(key);
        let ttl_seconds = ttl.as_secs() as usize;

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let result: bool = conn
            .expire(&prefixed_key, ttl_seconds)
            .await
            .map_err(|e| translate_redis_error(e))?;

        Ok(result)
    }

    async fn ttl<K>(&self, key: K) -> CacheResult<Option<Duration>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        let prefixed_key = self.format_key(key);

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let ttl_seconds: Option<i64> = conn
            .ttl(&prefixed_key)
            .await
            .map_err(|e| translate_redis_error(e))?;

        match ttl_seconds {
            Some(seconds) if seconds > 0 => Ok(Some(Duration::from_secs(seconds as u64))),
            _ => Ok(None),
        }
    }

    async fn clear(&self) -> CacheResult<()> {
        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        // Only clear keys with our prefix to avoid impacting other applications
        // using the same Redis instance
        if !self.config.key_prefix.is_empty() {
            let pattern = format!("{}*", self.config.key_prefix);
            let keys: Vec<String> = conn
                .keys(pattern)
                .await
                .map_err(|e| translate_redis_error(e))?;

            if !keys.is_empty() {
                conn.del(keys).await.map_err(|e| translate_redis_error(e))?;
            }
        } else {
            // If no prefix, use FLUSHDB - but this is risky as it will clear all keys
            conn.flushdb(false)
                .await
                .map_err(|e| translate_redis_error(e))?;
        }

        Ok(())
    }

    async fn health_check(&self) -> CacheResult<()> {
        self.connection_manager
            .health_check()
            .await
            .map_err(|e| e.into())
    }

    // List operations will be implemented in detail later

    async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        let prefixed_key = self.format_key(key);
        let serialized = serde_json::to_string(value)
            .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let result: i64 = conn
            .rpush(prefixed_key, serialized)
            .await
            .map_err(|e| translate_redis_error(e))?;

        Ok(result as usize)
    }

    async fn list_push_left<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        let prefixed_key = self.format_key(key);
        let serialized = serde_json::to_string(value)
            .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let result: i64 = conn
            .lpush(prefixed_key, serialized)
            .await
            .map_err(|e| translate_redis_error(e))?;

        Ok(result as usize)
    }

    async fn list_pop_right<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        let prefixed_key = self.format_key(key);

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let result: Option<String> = conn
            .rpop(prefixed_key, None)
            .await
            .map_err(|e| translate_redis_error(e))?;

        match result {
            Some(data) => serde_json::from_str(&data)
                .map(Some)
                .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string())),
            None => Ok(None),
        }
    }

    async fn list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        let prefixed_key = self.format_key(key);

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let result: Option<String> = conn
            .lpop(prefixed_key, None)
            .await
            .map_err(|e| translate_redis_error(e))?;

        match result {
            Some(data) => serde_json::from_str(&data)
                .map(Some)
                .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string())),
            None => Ok(None),
        }
    }

    async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        let prefixed_key = self.format_key(key);

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let results: Vec<String> = conn
            .lrange(prefixed_key, start, stop)
            .await
            .map_err(|e| translate_redis_error(e))?;

        let mut parsed_results = Vec::with_capacity(results.len());

        for data in results {
            let parsed = serde_json::from_str(&data)
                .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;
            parsed_results.push(parsed);
        }

        Ok(parsed_results)
    }

    async fn list_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        let prefixed_key = self.format_key(key);

        let mut conn = self
            .connection_manager
            .get_async_connection()
            .await
            .map_err(|e| translate_redis_error(e))?;

        let result: i64 = conn
            .llen(prefixed_key)
            .await
            .map_err(|e| translate_redis_error(e))?;

        Ok(result as usize)
    }

    // The rest of the collection operations can be implemented in future releases

    // Stub implementations for the remaining required methods to satisfy the trait
    // We'll add full implementations for these in Priority 2

    // List methods
    async fn list_push_right_many<K, V>(&self, _key: K, _values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Not implemented in Priority 1
        Err(navius_cache::error::CacheError::NotImplemented(
            "list_push_right_many".to_string(),
        ))
    }

    async fn list_push_left_many<K, V>(&self, _key: K, _values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Not implemented in Priority 1
        Err(navius_cache::error::CacheError::NotImplemented(
            "list_push_left_many".to_string(),
        ))
    }

    async fn list_remove<K, V>(&self, _key: K, _count: i32, _value: &V) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Not implemented in Priority 1
        Err(navius_cache::error::CacheError::NotImplemented(
            "list_remove".to_string(),
        ))
    }

    async fn list_trim<K>(&self, _key: K, _start: isize, _stop: isize) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        // Not implemented in Priority 1
        Err(navius_cache::error::CacheError::NotImplemented(
            "list_trim".to_string(),
        ))
    }

    async fn list_set<K, V>(&self, _key: K, _index: isize, _value: &V) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Not implemented in Priority 1
        Err(navius_cache::error::CacheError::NotImplemented(
            "list_set".to_string(),
        ))
    }

    // Hash methods
    async fn hash_set<K, F, V>(&self, key: K, field: F, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Clone + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "hash_set operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn hash_get<K, F, V>(&self, key: K, field: F) -> CacheResult<Option<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Clone + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "hash_get operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn hash_get_many<K, F, V>(&self, key: K, fields: Vec<F>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Eq + std::hash::Hash + Clone + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "hash_get_many operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn hash_set_many<K, F, V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Eq + std::hash::Hash + Clone + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "hash_set_many operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Clone + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "hash_exists operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Clone + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "hash_delete operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "hash_get_all operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn hash_keys<K>(&self, key: K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "hash_keys operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn hash_values<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "hash_values operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn hash_increment<K, F>(&self, key: K, field: F, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + Serialize + Clone + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "hash_increment operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn hash_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "hash_length operation not supported in current Redis implementation".to_string(),
        ))
    }

    // Set operations stubs
    async fn set_add<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "set_add operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "set_contains operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn set_intersection_store<K, D>(&self, keys: Vec<K>, destination: D) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        D: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "set_intersection_store operation not supported in current Redis implementation"
                .to_string(),
        ))
    }

    async fn set_union_store<K, D>(&self, keys: Vec<K>, destination: D) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        D: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "set_union_store operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn set_difference_store<K, D>(&self, keys: Vec<K>, destination: D) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        D: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "set_difference_store operation not supported in current Redis implementation"
                .to_string(),
        ))
    }

    async fn set_random_members<K, V>(&self, key: K, count: usize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "set_random_members operation not supported in current Redis implementation"
                .to_string(),
        ))
    }

    // Implement zset (sorted set) methods
    async fn zset_add<K, V>(&self, key: K, member: &V, score: f64) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_add operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn zset_remove<K, V>(&self, key: K, member: &V) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_remove operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn zset_score<K, V>(&self, key: K, member: &V) -> CacheResult<Option<f64>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_score operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn zset_increment_score<K, V>(
        &self,
        key: K,
        member: &V,
        increment: f64,
    ) -> CacheResult<f64>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_increment_score operation not supported in current Redis implementation"
                .to_string(),
        ))
    }

    async fn zset_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_range operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn zset_range_with_scores<K, V>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_range_with_scores operation not supported in current Redis implementation"
                .to_string(),
        ))
    }

    async fn zset_range_by_score<K, V>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_range_by_score operation not supported in current Redis implementation"
                .to_string(),
        ))
    }

    async fn zset_range_by_score_with_scores<K, V>(
        &self,
        key: K,
        min: f64,
        max: f64,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_range_by_score_with_scores operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn zset_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_rank operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn zset_rev_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_rev_rank operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn zset_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_length operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn zset_count<K>(&self, key: K, min: f64, max: f64) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_count operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn zset_rev_range_by_score<K, V>(&self, key: K, max: f64, min: f64) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_rev_range_by_score operation not supported in current Redis implementation"
                .to_string(),
        ))
    }

    async fn zset_rev_range_by_score_with_scores<K, V>(
        &self,
        key: K,
        max: f64,
        min: f64,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_rev_range_by_score_with_scores operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn zset_remove_range_by_rank<K>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_remove_range_by_rank operation not supported in current Redis implementation"
                .to_string(),
        ))
    }

    async fn zset_remove_range_by_score<K>(&self, key: K, min: f64, max: f64) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_remove_range_by_score operation not supported in current Redis implementation"
                .to_string(),
        ))
    }

    async fn zset_intersection_store<K, D>(
        &self,
        keys: Vec<K>,
        destination: D,
    ) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        D: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_intersection_store operation not supported in current Redis implementation"
                .to_string(),
        ))
    }

    async fn zset_union_store<K, D>(&self, keys: Vec<K>, destination: D) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        D: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "zset_union_store operation not supported in current Redis implementation".to_string(),
        ))
    }

    // Implement set methods required by the trait
    async fn set_members<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "set_members operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn set_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "set_length operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn set_intersection<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "set_intersection operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn set_union<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "set_union operation not supported in current Redis implementation".to_string(),
        ))
    }

    async fn set_difference<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        // Priority 2 feature - stub implementation
        Err(navius_cache::error::CacheError::OperationNotSupported(
            "set_difference operation not supported in current Redis implementation".to_string(),
        ))
    }
}

// Implement the marker trait
impl Cache for RedisCache {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // These tests are structured but will only run if a Redis server is available.
    // For CI environments, they should be marked #[ignore] or run conditionally.

    async fn create_test_cache() -> Option<RedisCache> {
        let config = RedisCacheConfig::new(
            "redis://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(60),
        );

        // Try to create the cache, but don't fail the test if Redis isn't available
        match RedisCache::new(config).await {
            Ok(cache) => Some(cache),
            Err(_) => None,
        }
    }

    #[tokio::test]
    async fn test_basic_operations() {
        if let Some(cache) = create_test_cache().await {
            // Test set and get
            let key = "test_key";
            let value = "test_value";

            assert!(cache.set(key, &value, None).await.is_ok());
            assert!(cache.exists(key).await.unwrap());

            let result: Option<String> = cache.get(key).await.unwrap();
            assert_eq!(result, Some(value.to_string()));

            // Test delete
            assert!(cache.delete(key).await.unwrap());
            assert!(!cache.exists(key).await.unwrap());

            // Clean up
            let _ = cache.clear().await;
        }
    }

    #[tokio::test]
    async fn test_ttl_operations() {
        if let Some(cache) = create_test_cache().await {
            let key = "ttl_test_key";
            let value = "test_value";

            // Set with explicit TTL
            let options = CacheOptions {
                ttl: Some(Duration::from_secs(10)),
            };

            assert!(cache.set(key, &value, Some(options)).await.is_ok());

            // Check TTL
            let ttl = cache.ttl(key).await.unwrap();
            assert!(ttl.is_some());

            let ttl_secs = ttl.unwrap().as_secs();
            assert!(ttl_secs > 0 && ttl_secs <= 10);

            // Test expire
            assert!(cache.expire(key, Duration::from_secs(20)).await.unwrap());

            let ttl = cache.ttl(key).await.unwrap();
            assert!(ttl.is_some());

            let ttl_secs = ttl.unwrap().as_secs();
            assert!(ttl_secs > 10 && ttl_secs <= 20);

            // Clean up
            let _ = cache.clear().await;
        }
    }

    #[tokio::test]
    async fn test_increment() {
        if let Some(cache) = create_test_cache().await {
            let key = "counter_key";

            // Initial increment
            let result = cache.increment(key, 1).await.unwrap();
            assert_eq!(result, 1);

            // Increment by 5
            let result = cache.increment(key, 5).await.unwrap();
            assert_eq!(result, 6);

            // Clean up
            let _ = cache.clear().await;
        }
    }

    #[tokio::test]
    async fn test_set_many_and_get_many() {
        if let Some(cache) = create_test_cache().await {
            let entries = vec![("key1", "value1"), ("key2", "value2"), ("key3", "value3")];

            assert!(cache.set_many(entries, None).await.is_ok());

            let keys = vec!["key1", "key2", "key3", "nonexistent"];
            let results: Vec<Option<String>> = cache.get_many(keys).await.unwrap();

            assert_eq!(results.len(), 4);
            assert_eq!(results[0], Some("value1".to_string()));
            assert_eq!(results[1], Some("value2".to_string()));
            assert_eq!(results[2], Some("value3".to_string()));
            assert_eq!(results[3], None);

            // Clean up
            let _ = cache.clear().await;
        }
    }

    #[tokio::test]
    async fn test_delete_many() {
        if let Some(cache) = create_test_cache().await {
            let entries = vec![
                ("del_key1", "value1"),
                ("del_key2", "value2"),
                ("del_key3", "value3"),
            ];

            assert!(cache.set_many(entries, None).await.is_ok());

            // Delete keys 1 and 3
            let delete_keys = vec!["del_key1", "del_key3"];
            let count = cache.delete_many(delete_keys).await.unwrap();
            assert_eq!(count, 2);

            // Verify key2 still exists
            assert!(cache.exists("del_key2").await.unwrap());

            // Verify key1 and key3 are gone
            assert!(!cache.exists("del_key1").await.unwrap());
            assert!(!cache.exists("del_key3").await.unwrap());

            // Clean up
            let _ = cache.clear().await;
        }
    }

    #[tokio::test]
    async fn test_list_operations() {
        if let Some(cache) = create_test_cache().await {
            let key = "list_test";

            // Test push operations
            assert_eq!(cache.list_push_right(key, &"value1").await.unwrap(), 1);
            assert_eq!(cache.list_push_right(key, &"value2").await.unwrap(), 2);
            assert_eq!(cache.list_push_left(key, &"value0").await.unwrap(), 3);

            // Test length
            assert_eq!(cache.list_length(key).await.unwrap(), 3);

            // Test range
            let range: Vec<String> = cache.list_range(key, 0, -1).await.unwrap();
            assert_eq!(range, vec!["value0", "value1", "value2"]);

            // Test pop operations
            let right_value: Option<String> = cache.list_pop_right(key).await.unwrap();
            assert_eq!(right_value, Some("value2".to_string()));

            let left_value: Option<String> = cache.list_pop_left(key).await.unwrap();
            assert_eq!(left_value, Some("value0".to_string()));

            // Verify length is now 1
            assert_eq!(cache.list_length(key).await.unwrap(), 1);

            // Pop the last item
            let last_value: Option<String> = cache.list_pop_left(key).await.unwrap();
            assert_eq!(last_value, Some("value1".to_string()));

            // Verify the list is empty
            assert_eq!(cache.list_length(key).await.unwrap(), 0);

            // Pop from empty list should return None
            let empty_value: Option<String> = cache.list_pop_left(key).await.unwrap();
            assert_eq!(empty_value, None);

            // Clean up
            let _ = cache.clear().await;
        }
    }
}
