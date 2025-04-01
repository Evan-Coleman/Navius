//! Redis implementation for cache invalidation.

use crate::connection::RedisConnectionManager;
use async_trait::async_trait;
use navius_cache::CacheKey;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashSet, sync::Arc, time::Duration};
use tracing::{debug, instrument};

use navius_cache::error::{CacheError, CacheResult};
use navius_cache::invalidation::CacheInvalidator;
use navius_cache::CacheOperations;

/// Redis implementation of CacheInvalidator
pub struct RedisInvalidator {
    /// The Redis cache instance to use for operations
    cache: Arc<RedisCache>,
}

impl RedisInvalidator {
    /// Create a new Redis invalidator with the given cache
    pub fn new(cache: Arc<RedisCache>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl CacheInvalidator for RedisInvalidator {
    #[instrument(skip(self, key), level = "debug")]
    async fn invalidate<K: CacheKey>(&self, key: K) -> CacheResult<bool> {
        self.cache.delete(key).await
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn invalidate_many<K: CacheKey>(&self, keys: Vec<K>) -> CacheResult<usize> {
        self.cache.delete_many(keys).await
    }

    #[instrument(skip(self, pattern), level = "debug")]
    async fn invalidate_pattern(&self, pattern: &str) -> CacheResult<usize> {
        let keys = self.cache.keys(pattern).await?;
        if keys.is_empty() {
            return Ok(0);
        }
        self.cache.delete_many(keys).await
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_all(&self) -> CacheResult<()> {
        self.cache.clear().await
    }

    #[instrument(skip(self, key, ttl), level = "debug")]
    async fn set_ttl<K: CacheKey>(&self, key: K, ttl: Duration) -> CacheResult<bool> {
        self.cache.expire(key, ttl).await
    }

    #[instrument(skip(self), level = "debug")]
    async fn tag_key(&self, key: &str, tags: &[&str]) -> CacheResult<()> {
        if tags.is_empty() {
            return Ok(());
        }

        let prefixed_key = self.cache.prefixed_key(key);
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        // For each tag, add the key to the tag's set
        for tag in tags {
            let tag_key = self.cache.prefixed_key(&format!("tag:{}", tag));
            let _: () = redis::cmd("SADD")
                .arg(&tag_key)
                .arg(&prefixed_key)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::InvalidationError(format!("Failed to tag key: {}", e)))?;
        }

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn get_key_tags(&self, key: &str) -> CacheResult<HashSet<String>> {
        let prefixed_key = self.cache.prefixed_key(key);
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        // Find all tag keys
        let tag_pattern = self.cache.prefixed_key("tag:*");
        let tag_keys: Vec<String> = redis::cmd("KEYS")
            .arg(&tag_pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to find tag keys: {}", e))
            })?;

        let mut tags = HashSet::new();

        // For each tag, check if our key is in its set
        for tag_key in tag_keys {
            let is_member: bool = redis::cmd("SISMEMBER")
                .arg(&tag_key)
                .arg(&prefixed_key)
                .query_async(&mut conn)
                .await
                .map_err(|e| {
                    CacheError::InvalidationError(format!("Failed to check tag membership: {}", e))
                })?;

            if is_member {
                // Extract the tag name from the tag key (removing the prefix and "tag:" part)
                let tag = tag_key
                    .strip_prefix(&self.cache.key_prefix())
                    .and_then(|s| s.strip_prefix("tag:"))
                    .unwrap_or("");

                tags.insert(tag.to_string());
            }
        }

        Ok(tags)
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_by_tag(&self, tag: &str) -> CacheResult<u64> {
        let tag_key = self.cache.prefixed_key(&format!("tag:{}", tag));
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        // Get all keys for this tag
        let keys: Vec<String> = redis::cmd("SMEMBERS")
            .arg(&tag_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to get keys for tag: {}", e))
            })?;

        if keys.is_empty() {
            return Ok(0);
        }

        // Delete all those keys
        let count: i64 = redis::cmd("DEL")
            .arg(&keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to delete keys for tag: {}", e))
            })?;

        // Also delete the tag set itself
        let _: () = redis::cmd("DEL")
            .arg(&tag_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to delete tag set: {}", e))
            })?;

        Ok(count as u64)
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_by_tags(&self, tags: &[&str]) -> CacheResult<u64> {
        if tags.is_empty() {
            return Ok(0);
        }

        let mut total_count = 0;

        // For each tag, invalidate its keys
        for tag in tags {
            let count = self.invalidate_by_tag(tag).await?;
            total_count += count;
        }

        Ok(total_count)
    }
}

#[cfg(test)]
mod tests {
    // Tests would be here in a real implementation
    // They would use a real Redis instance or a mock
}
