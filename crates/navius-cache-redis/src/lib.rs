// Navius Cache Redis Implementation
//
// This crate provides a Redis-specific implementation of the Navius cache system.

#![deny(missing_docs)]
#![warn(clippy::all)]

/// Redis cache configuration
pub mod config;
/// Redis cache invalidation
pub mod invalidation;
/// Redis cache lua scripting
pub mod lua;
/// Redis cache metrics
pub mod metrics;
/// Redis cache operations
pub mod operations;
/// Redis cache pipelining
pub mod pipeline;

// Re-export important types
pub use config::RedisCacheConfig;
pub use invalidation::RedisInvalidator;
pub use lua::{initialize_common_scripts, RedisLuaManager, RedisLuaScripting};
pub use metrics::{
    record_connection_acquisition, record_connection_health, record_connection_pool_stats,
    TimedOperation,
};
pub use operations::RedisCache;
pub use pipeline::{RedisPipeline, RedisPipelineBuilder, RedisPipelineManager};
pub use redis::Pipeline;

// Error types
pub mod error;
pub use error::{RedisCacheError, RedisCacheResult};

// Connection management
pub mod connection;
pub use connection::{ConnectionHealth, PoolStats, RedisConnectionManager};

use std::{fmt::Debug, sync::Arc, time::Duration};

use navius_cache::{Cache, CacheError, CacheKey, CacheOperations, CacheResult};
use redis::RedisError;
use serde::{de::DeserializeOwned, Serialize};
use tracing::instrument;

// Skip duplicate module declarations
// These are already declared above with pub visibility
// mod connection;
// mod error;
// mod lua;
// mod operations;
// mod pipeline;

use connection::RedisConnectionManager;
use error::RedisCacheError;
use lua::RedisLuaManager;
use operations::RedisOperations;
use pipeline::RedisPipeline;

#[derive(Debug, Clone)]
pub struct RedisCacheConfig {
    pub url: String,
    pub key_prefix: Option<String>,
    pub command_timeout_seconds: u64,
}

#[derive(Clone)]
pub struct RedisCache {
    operations: Arc<RedisOperations<String, Vec<u8>>>,
}

impl RedisCache {
    #[instrument(skip(redis_url), level = "debug")]
    pub async fn new(
        redis_url: String,
        command_timeout: Duration,
        key_prefix: String,
    ) -> Result<Self, CacheError> {
        let connection_manager =
            RedisConnectionManager::new(&redis_url, Some(key_prefix), command_timeout)
                .map_err(|e| RedisOperations::into_cache_error(e))?;

        let operations = Arc::new(RedisOperations::new(Arc::new(connection_manager)));

        Ok(Self { operations })
    }
}

#[async_trait::async_trait]
impl CacheOperations for RedisCache {
    async fn get<T>(&self, key: &K) -> Result<Option<T>, CacheError>
    where
        T: DeserializeOwned + 'static,
    {
        self.operations
            .get::<T>(key.as_ref())
            .await
            .map_err(|err| RedisOperations::into_cache_error(err))
    }

    async fn get_many<T>(&self, keys: Vec<K>) -> Result<Vec<Option<T>>, CacheError>
    where
        T: DeserializeOwned + 'static,
        V: DeserializeOwned + 'static,
    {
        let str_keys: Vec<&str> = keys.iter().map(|k| k.as_ref()).collect();
        self.operations
            .get_many::<T>(&str_keys)
            .await
            .map_err(|err| RedisOperations::into_cache_error(err))
    }

    async fn set<K, V>(
        &self,
        key: K,
        value: &V,
        options: Option<navius_cache::CacheOptions>,
    ) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .set(key, value, options)
            .await
            .map_err(|err| RedisOperations::into_cache_error(err))
    }

    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<navius_cache::CacheOptions>,
    ) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .set_many(entries, options)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .delete(key)
            .await
            .map_err(|err| RedisOperations::into_cache_error(err))
    }

    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .delete_many(keys)
            .await
            .map_err(|err| RedisOperations::into_cache_error(err))
    }

    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .exists(key)
            .await
            .map_err(|err| RedisOperations::into_cache_error(err))
    }

    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .increment(key, amount)
            .await
            .map_err(|err| RedisOperations::into_cache_error(err))
    }

    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .expire(key, ttl)
            .await
            .map_err(|err| RedisOperations::into_cache_error(err))
    }

    async fn clear(&self) -> CacheResult<()> {
        self.operations
            .clear()
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn health_check(&self) -> CacheResult<()> {
        self.operations
            .health_check()
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    // List Operations
    async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .list_push_right(key, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn list_push_right_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .list_push_right_many(key, values)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn list_push_left<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .list_push_left(key, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn list_push_left_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .list_push_left_many(key, values)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn list_pop_right<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .list_pop_right(key)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .list_pop_left(key)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .list_range(key, start, stop)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn list_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .list_length(key)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn list_remove<K, V>(&self, key: K, count: isize, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .list_remove(key, count, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn list_trim<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .list_trim(key, start, stop)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn list_set<K, V>(&self, key: K, index: isize, value: &V) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .list_set(key, index, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    // Hash Operations
    async fn hash_get<K, F, V>(&self, key: K, field: F) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .hash_get(key, field)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn hash_set<K, F, V>(&self, key: K, field: F, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .hash_set(key, field, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn hash_get_many<K, F, V>(&self, key: K, fields: Vec<F>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .hash_get_many(key, fields)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn hash_set_many<K, F, V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .hash_set_many(key, entries)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        self.operations
            .hash_exists(key, field)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn hash_delete<K, F>(&self, key: K, field: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        self.operations
            .hash_delete(key, field)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .hash_get_all(key)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn hash_keys<K>(&self, key: K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .hash_keys(key)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn hash_values<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .hash_values(key)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn hash_increment<K, F>(&self, key: K, field: F, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        self.operations
            .hash_increment(key, field, amount)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn hash_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .hash_length(key)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    // Set Operations
    async fn set_add<K, V>(&self, key: K, value: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .set_add(key, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn set_remove<K, V>(&self, key: K, value: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .set_remove(key, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .set_contains(key, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn set_members<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .set_members(key)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn set_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .set_length(key)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn set_intersection<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .set_intersection(keys)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn set_intersection_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        self.operations
            .set_intersection_store(destination, keys)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn set_union<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .set_union(keys)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn set_union_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        self.operations
            .set_union_store(destination, keys)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn set_difference<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .set_difference(keys)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn set_difference_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        self.operations
            .set_difference_store(destination, keys)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn set_random_members<K, V>(&self, key: K, count: usize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .set_random_members(key, count)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    // Sorted Set Operations
    async fn zset_add<K, V>(&self, key: K, values: Vec<(f64, V)>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .zset_add(key, values)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_remove<K, V>(&self, key: K, value: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .zset_remove(key, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_score<K, V>(&self, key: K, value: &V) -> CacheResult<Option<f64>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .zset_score(key, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_increment_score<K, V>(
        &self,
        key: K,
        value: &V,
        increment: f64,
    ) -> CacheResult<f64>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .zset_increment_score(key, value, increment)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .zset_range(key, start, stop)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_range_with_scores<K, V>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .zset_range_with_scores(key, start, stop)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_range_by_score<K, V>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .zset_range_by_score(key, min, max)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_range_by_score_with_scores<K, V>(
        &self,
        key: K,
        min: f64,
        max: f64,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.operations
            .zset_range_by_score_with_scores(key, min, max)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_rank<K, V>(&self, key: K, value: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .zset_rank(key, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_reverse_rank<K, V>(&self, key: K, value: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.operations
            .zset_reverse_rank(key, value)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .zset_length(key)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_count<K>(&self, key: K, min: f64, max: f64) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        self.operations
            .zset_count(key, min, max)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_intersection_store<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
        weights: Option<Vec<f64>>,
        aggregate: Option<String>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        self.operations
            .zset_intersection_store(destination, keys, weights, aggregate)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }

    async fn zset_union_store<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
        weights: Option<Vec<f64>>,
        aggregate: Option<String>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        self.operations
            .zset_union_store(destination, keys, weights, aggregate)
            .await
            .map_err(|e| RedisOperations::into_cache_error(e))
    }
}

impl Cache for RedisCache {}
