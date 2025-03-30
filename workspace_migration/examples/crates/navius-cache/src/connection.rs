use crate::config::CacheConfig;
use crate::error::CacheResult;
#[cfg(feature = "redis")]
use crate::operations::redis::RedisCache;
use crate::operations::{Cache, CacheOperations};
use std::sync::Arc;
use tracing::{debug, error, instrument};

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
        V: serde::de::DeserializeOwned + 'static,
    {
        self.cache.get(key).await
    }

    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::de::DeserializeOwned + 'static,
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
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.cache.set(key, value, options).await
    }

    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<crate::operations::CacheOptions>,
    ) -> CacheResult<()>
    where
        K: crate::operations::CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
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
}
