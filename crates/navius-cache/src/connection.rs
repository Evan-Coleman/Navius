use crate::config::CacheConfig;
use crate::error::CacheResult;
#[cfg(feature = "redis")]
use crate::operations::redis::RedisCache;
use crate::operations::{Cache, CacheOperations};
use std::cmp::Eq;
use std::hash::Hash;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Wrapper for cache connections that handles initialization and provides a unified interface
#[derive(Debug, Clone)]
pub struct CacheConnectionManager<C: Cache> {
    /// The actual cache implementation
    cache: C,
    /// Cache configuration
    config: Arc<CacheConfig>,
}

impl<C: Cache> CacheConnectionManager<C> {
    /// Create a new cache connection manager with an existing cache instance
    pub fn new(cache: C, config: CacheConfig) -> Self {
        Self {
            cache,
            config: Arc::new(config),
        }
    }

    /// Get a reference to the underlying cache implementation
    pub fn cache(&self) -> &C {
        &self.cache
    }

    /// Get a reference to the cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

/// Factory method for creating a Redis cache connection manager
#[cfg(feature = "redis")]
impl CacheConnectionManager<RedisCache> {
    /// Create a new Redis cache connection manager
    #[instrument(skip(config))]
    pub async fn new_redis(config: CacheConfig) -> CacheResult<Self> {
        debug!("Creating Redis cache connection manager");

        let cache = RedisCache::new(config.clone()).await?;

        debug!("Redis cache connection manager created successfully");

        Ok(Self {
            cache,
            config: Arc::new(config),
        })
    }
}

/// Delegate Cache trait methods to the underlying cache implementation
#[async_trait::async_trait]
impl<C: Cache> CacheOperations for CacheConnectionManager<C> {
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        self.cache.get(key).await
    }

    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        self.cache.get_many(keys).await
    }

    async fn set<K, V>(
        &self,
        key: K,
        value: &V,
        options: Option<crate::operations::CacheOptions>,
    ) -> CacheResult<()>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + Clone + 'static,
    {
        self.cache.set(key, value, options).await
    }

    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<crate::operations::CacheOptions>,
    ) -> CacheResult<()>
    where
        K: crate::operations::CacheKey + Eq + Hash + 'static,
        V: serde::Serialize + Send + Sync + Clone + 'static,
    {
        self.cache.set_many(entries, options).await
    }

    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: crate::operations::CacheKey + 'static,
    {
        self.cache.delete(key).await
    }

    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
    {
        self.cache.delete_many(keys).await
    }

    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: crate::operations::CacheKey + 'static,
    {
        self.cache.exists(key).await
    }

    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: crate::operations::CacheKey + 'static,
    {
        self.cache.increment(key, amount).await
    }

    async fn expire<K>(&self, key: K, ttl: std::time::Duration) -> CacheResult<bool>
    where
        K: crate::operations::CacheKey + 'static,
    {
        self.cache.expire(key, ttl).await
    }

    async fn clear(&self) -> CacheResult<()> {
        self.cache.clear().await
    }

    async fn health_check(&self) -> CacheResult<()> {
        self.cache.health_check().await
    }

    // List operations
    async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.list_push_right(key, value).await
    }

    async fn list_push_right_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.list_push_right_many(key, values).await
    }

    async fn list_push_left<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.list_push_left(key, value).await
    }

    async fn list_push_left_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.list_push_left_many(key, values).await
    }

    async fn list_pop_right<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + 'static,
    {
        self.cache.list_pop_right(key).await
    }

    async fn list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + 'static,
    {
        self.cache.list_pop_left(key).await
    }

    async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        self.cache.list_range(key, start, stop).await
    }

    async fn list_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
    {
        self.cache.list_length(key).await
    }

    async fn list_remove<K, V>(&self, key: K, count: i32, value: &V) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + Clone + 'static,
    {
        self.cache.list_remove(key, count, value).await
    }

    async fn list_trim<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
    where
        K: crate::operations::CacheKey + 'static,
    {
        self.cache.list_trim(key, start, stop).await
    }

    async fn list_set<K, V>(&self, key: K, index: isize, value: &V) -> CacheResult<()>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.list_set(key, index, value).await
    }

    // Hash operations
    async fn hash_get<K, F, V>(&self, key: K, field: F) -> CacheResult<Option<V>>
    where
        K: crate::operations::CacheKey + 'static,
        F: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        self.cache.hash_get(key, field).await
    }

    async fn hash_set<K, F, V>(&self, key: K, field: F, value: &V) -> CacheResult<bool>
    where
        K: crate::operations::CacheKey + 'static,
        F: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.hash_set(key, field, value).await
    }

    async fn hash_get_many<K, F, V>(&self, key: K, fields: Vec<F>) -> CacheResult<Vec<Option<V>>>
    where
        K: crate::operations::CacheKey + 'static,
        F: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        self.cache.hash_get_many(key, fields).await
    }

    async fn hash_set_many<K, F, V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<()>
    where
        K: crate::operations::CacheKey + 'static,
        F: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.hash_set_many(key, entries).await
    }

    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: crate::operations::CacheKey + 'static,
        F: crate::operations::CacheKey + 'static,
    {
        self.cache.hash_exists(key, field).await
    }

    async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        F: crate::operations::CacheKey + 'static,
    {
        self.cache.hash_delete(key, fields).await
    }

    async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        self.cache.hash_get_all(key).await
    }

    async fn hash_keys<K>(&self, key: K) -> CacheResult<Vec<String>>
    where
        K: crate::operations::CacheKey + 'static,
    {
        self.cache.hash_keys(key).await
    }

    async fn hash_values<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        self.cache.hash_values(key).await
    }

    async fn hash_increment<K, F>(&self, key: K, field: F, amount: i64) -> CacheResult<i64>
    where
        K: crate::operations::CacheKey + 'static,
        F: crate::operations::CacheKey + 'static,
    {
        self.cache.hash_increment(key, field, amount).await
    }

    async fn hash_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
    {
        self.cache.hash_length(key).await
    }

    // Set operations
    async fn set_add<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.set_add(key, values).await
    }

    async fn set_remove<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.set_remove(key, values).await
    }

    async fn set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.set_contains(key, value).await
    }

    async fn set_members<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: crate::operations::CacheKey + std::fmt::Debug + 'static,
        V: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        self.cache.set_members(key).await
    }

    async fn set_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + std::fmt::Debug + 'static,
    {
        self.cache.set_length(key).await
    }

    async fn set_intersection<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + 'static,
    {
        self.cache.set_intersection(keys).await
    }

    async fn set_intersection_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        D: crate::operations::CacheKey + 'static,
    {
        self.cache.set_intersection_store(destination, keys).await
    }

    async fn set_union<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + 'static,
    {
        self.cache.set_union(keys).await
    }

    async fn set_union_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        D: crate::operations::CacheKey + 'static,
    {
        self.cache.set_union_store(destination, keys).await
    }

    async fn set_difference<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + 'static,
    {
        self.cache.set_difference(keys).await
    }

    async fn set_difference_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        D: crate::operations::CacheKey + 'static,
    {
        self.cache.set_difference_store(destination, keys).await
    }

    async fn set_random_members<K, V>(&self, key: K, count: usize) -> CacheResult<Vec<V>>
    where
        K: crate::operations::CacheKey + std::fmt::Debug + 'static,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        self.cache.set_random_members(key, count).await
    }

    // Sorted set operations
    async fn zset_add<K, V>(&self, key: K, items: Vec<(f64, V)>) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.zset_add(key, items).await
    }

    async fn zset_remove<K, V>(&self, key: K, members: Vec<V>) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.zset_remove(key, members).await
    }

    async fn zset_score<K, V>(&self, key: K, member: &V) -> CacheResult<Option<f64>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.zset_score(key, member).await
    }

    async fn zset_increment_score<K, V>(&self, key: K, member: &V, amount: f64) -> CacheResult<f64>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.zset_increment_score(key, member, amount).await
    }

    async fn zset_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + 'static,
    {
        self.cache.zset_range(key, start, stop).await
    }

    async fn zset_range_with_scores<K, V>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + 'static,
    {
        self.cache.zset_range_with_scores(key, start, stop).await
    }

    async fn zset_range_by_score<K, V>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<V>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + 'static,
    {
        self.cache.zset_range_by_score(key, min, max).await
    }

    async fn zset_range_by_score_with_scores<K, V>(
        &self,
        key: K,
        min: f64,
        max: f64,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + 'static,
    {
        self.cache
            .zset_range_by_score_with_scores(key, min, max)
            .await
    }

    async fn zset_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.zset_rank(key, member).await
    }

    async fn zset_reverse_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.zset_reverse_rank(key, member).await
    }

    async fn zset_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
    {
        self.cache.zset_length(key).await
    }

    async fn zset_count<K>(&self, key: K) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
    {
        self.cache.zset_count(key).await
    }

    async fn zset_intersection_store<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
    ) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        D: crate::operations::CacheKey + 'static,
    {
        self.cache.zset_intersection_store(destination, keys).await
    }

    async fn zset_union_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: crate::operations::CacheKey + 'static,
        D: crate::operations::CacheKey + 'static,
    {
        self.cache.zset_union_store(destination, keys).await
    }
}
