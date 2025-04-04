use crate::error::CacheResult;
use crate::operations::{Cache, CacheKey};
use async_trait::async_trait;
use std::marker::PhantomData;
use std::time::Duration;
use tracing::{debug, instrument};

/// Cache invalidation strategy types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidationStrategy {
    /// Invalidate immediately
    Immediate,
    /// Time-based expiration
    TimeToLive(Duration),
    /// Per-entity invalidation
    EntityBased,
    /// Pattern-based invalidation
    PatternBased(String),
}

/// Cache invalidation trait for managing cache invalidation
#[async_trait]
pub trait CacheInvalidation<C: Cache> {
    /// Invalidate cache entries by key
    async fn invalidate<K: CacheKey + std::fmt::Debug + 'static>(
        &self,
        key: K,
    ) -> CacheResult<bool>;

    /// Invalidate cache entries by pattern
    async fn invalidate_by_pattern(&self, pattern: &str) -> CacheResult<usize>;

    /// Invalidate all cache entries
    async fn invalidate_all(&self) -> CacheResult<()>;

    /// Set the invalidation strategy
    fn set_strategy(&mut self, strategy: InvalidationStrategy);

    /// Get the current invalidation strategy
    fn strategy(&self) -> &InvalidationStrategy;
}

/// Cache invalidator implementation
#[derive(Debug, Clone)]
pub struct CacheInvalidator<C: Cache, K: CacheKey + 'static> {
    /// The cache to invalidate
    cache: C,
    /// Invalidation strategy
    strategy: InvalidationStrategy,
    /// Phantom type for the key
    _key_type: PhantomData<K>,
}

impl<C: Cache, K: CacheKey + 'static> CacheInvalidator<C, K> {
    /// Create a new cache invalidator
    pub fn new(cache: C, strategy: InvalidationStrategy) -> Self {
        Self {
            cache,
            strategy,
            _key_type: PhantomData,
        }
    }

    /// Convert a key to a pattern for pattern-based invalidation
    fn key_to_pattern<T: CacheKey>(&self, key: T) -> String {
        match &self.strategy {
            InvalidationStrategy::PatternBased(pattern) => {
                let key_str = key.to_string();
                pattern.replace("{key}", &key_str)
            }
            _ => key.to_string(),
        }
    }
}

#[async_trait]
impl<C: Cache, K: CacheKey + 'static> CacheInvalidation<C> for CacheInvalidator<C, K> {
    /// Invalidate a cache entry. If a TTL is set, the entry will be set to expire after the TTL.
    /// Otherwise, the entry will be deleted immediately.
    ///
    /// # Arguments
    /// * `key` - The key to invalidate
    ///
    /// # Returns
    /// * `CacheResult<bool>` - Whether the invalidation was successful
    #[instrument(level = "debug", skip(self), err)]
    async fn invalidate<T: CacheKey + std::fmt::Debug + 'static>(
        &self,
        key: T,
    ) -> CacheResult<bool> {
        match self.strategy {
            InvalidationStrategy::Immediate => {
                debug!("Invalidating cache entry by deleting");
                self.cache.delete(key).await
            }
            InvalidationStrategy::TimeToLive(ref ttl) => {
                debug!("Invalidating cache entry by setting TTL");
                self.cache.expire(key, *ttl).await
            }
            InvalidationStrategy::EntityBased => {
                debug!("Invalidating cache entry with custom strategy");
                self.cache.delete(key).await
            }
            InvalidationStrategy::PatternBased(_) => {
                // Convert the key to a pattern and invalidate by pattern
                let pattern = self.key_to_pattern(key);
                let count = self.invalidate_by_pattern(&pattern).await?;
                Ok(count > 0)
            }
        }
    }

    #[instrument(skip(self))]
    async fn invalidate_by_pattern(&self, pattern: &str) -> CacheResult<usize> {
        debug!("Invalidating cache entries by pattern: {}", pattern);

        // Add a wildcard to the pattern to match all keys with the given prefix
        let pattern_with_wildcard = format!("{}*", pattern);

        // Delete all keys matching the pattern
        self.cache.delete_many(vec![pattern_with_wildcard]).await
    }

    #[instrument(skip(self))]
    async fn invalidate_all(&self) -> CacheResult<()> {
        debug!("Invalidating all cache entries");

        self.cache.clear().await
    }

    fn set_strategy(&mut self, strategy: InvalidationStrategy) {
        debug!("Setting invalidation strategy: {:?}", strategy);
        self.strategy = strategy;
    }

    fn strategy(&self) -> &InvalidationStrategy {
        &self.strategy
    }
}
