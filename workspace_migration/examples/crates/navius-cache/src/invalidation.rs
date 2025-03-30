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
    async fn invalidate<K: CacheKey + 'static>(&self, key: K) -> CacheResult<bool>;

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
    #[instrument(skip(self, key))]
    async fn invalidate<T: CacheKey + 'static>(&self, key: T) -> CacheResult<bool> {
        debug!("Invalidating cache entry: {}", key.to_string());

        match &self.strategy {
            InvalidationStrategy::Immediate => {
                // Simply delete the key
                self.cache.delete(key).await
            }
            InvalidationStrategy::TimeToLive(ttl) => {
                // Set the key to expire
                self.cache.expire(key, *ttl).await
            }
            InvalidationStrategy::EntityBased => {
                // Delete the specific entity key
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

        // For Redis, we'd typically use SCAN + DEL
        // For simplicity, we'll implement a basic version here
        // In a real implementation, you'd want to optimize this for your specific cache backend

        // In our Redis implementation, we're already using prefixed keys
        // So we can just use the pattern as-is with our prefix and a wildcard
        let pattern_with_wildcard = format!("{}*", pattern);

        // This is a simplified implementation that just clears all keys matching the pattern
        // In a production implementation, you'd want to use a more efficient approach
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
