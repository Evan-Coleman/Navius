use async_trait::async_trait;
use navius_cache::{
    error::{CacheError, CacheResult},
    invalidation::CacheInvalidator as NaviusCacheInvalidator,
    key::CacheKey,
};
use redis::aio::ConnectionManager;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashSet, sync::Arc, time::Duration};
use tracing::{debug, error, instrument};

use crate::{
    connection::RedisConnectionManager,
    error::{error_helpers, RedisCacheError, RedisCacheResult},
    operations::RedisCache,
};

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

impl NaviusCacheInvalidator for RedisInvalidator {
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

#[async_trait]
impl CacheOperations for RedisInvalidator {
    #[instrument(skip(self, entity), level = "debug")]
    async fn track_entity_change<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        entity_type: &str,
        entity_id: &str,
        entity: &T,
    ) -> CacheResult<()> {
        let entity_key = self
            .cache
            .prefixed_key(&format!("entity:{}:{}", entity_type, entity_id));
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        // Store the serialized entity in the cache
        let serialized = serde_json::to_string(entity).map_err(|e| {
            CacheError::SerializationError(format!("Failed to serialize entity: {}", e))
        })?;

        let _: () = redis::cmd("SET")
            .arg(&entity_key)
            .arg(serialized)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::InvalidationError(format!("Failed to store entity: {}", e)))?;

        debug!("Tracked entity change: {}:{}", entity_type, entity_id);
        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn get_entity_cache_keys(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> CacheResult<Vec<String>> {
        let entity_key = self
            .cache
            .prefixed_key(&format!("entity:{}:{}", entity_type, entity_id));
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        // Get all cache keys related to this entity
        let cache_keys: Vec<String> = redis::cmd("SMEMBERS")
            .arg(&entity_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to get entity cache keys: {}", e))
            })?;

        // Remove the prefix from the keys to return the original key names
        let prefix = self.cache.key_prefix();
        let result: Vec<String> = cache_keys
            .into_iter()
            .filter_map(|key| key.strip_prefix(prefix).map(ToString::to_string))
            .collect();

        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_entity(&self, entity_type: &str, entity_id: &str) -> CacheResult<u64> {
        let entity_key = self
            .cache
            .prefixed_key(&format!("entity:{}:{}", entity_type, entity_id));
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        // Get all cache keys related to this entity
        let keys: Vec<String> = redis::cmd("SMEMBERS")
            .arg(&entity_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to get entity cache keys: {}", e))
            })?;

        if keys.is_empty() {
            // Just delete the entity itself
            let _: () = redis::cmd("DEL")
                .arg(&entity_key)
                .query_async(&mut conn)
                .await
                .map_err(|e| {
                    CacheError::InvalidationError(format!("Failed to delete entity: {}", e))
                })?;

            return Ok(1);
        }

        // Delete all related cache keys
        let count: i64 = redis::cmd("DEL")
            .arg(&keys)
            .arg(&entity_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to invalidate entity: {}", e))
            })?;

        debug!(
            "Invalidated entity {}:{} with {} related cache keys",
            entity_type,
            entity_id,
            keys.len()
        );
        Ok(count as u64)
    }
}

#[cfg(test)]
mod tests {
    // Tests would be here in a real implementation
    // They would use a real Redis instance or a mock
}
