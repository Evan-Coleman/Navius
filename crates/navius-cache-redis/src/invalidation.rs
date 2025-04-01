use async_trait::async_trait;
use navius_cache::{
    error::{CacheError, CacheResult},
    invalidation::{CacheEntityTracker, CacheEventInvalidator, CacheInvalidator, CacheTtlManager},
};
use redis::{aio::ConnectionManager, cmd};
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashSet, sync::Arc, time::Duration};
use tracing::{debug, error, instrument};

use crate::{
    connection::RedisConnectionManager,
    error::{RedisCacheError, RedisCacheResult, error_helpers},
};

/// Redis-specific cache invalidation implementation
#[derive(Clone)]
pub struct RedisInvalidator {
    /// Redis connection manager
    connection_manager: RedisConnectionManager,
}

impl RedisInvalidator {
    /// Create a new Redis invalidator
    pub fn new(connection_manager: RedisConnectionManager) -> Self {
        Self { connection_manager }
    }

    /// Get prefixed key
    fn prefixed_key(&self, key: &str) -> String {
        self.connection_manager.prefixed_key(key)
    }

    /// Get the tag key for a given tag
    fn tag_key(&self, tag: &str) -> String {
        self.prefixed_key(&format!("tag:{}", tag))
    }

    /// Get the entity key for tracking entity-related cache keys
    fn entity_key(&self, entity_type: &str, entity_id: &str) -> String {
        self.prefixed_key(&format!("entity:{}:{}", entity_type, entity_id))
    }

    /// Get a connection from the pool
    async fn get_connection(&self) -> RedisCacheResult<ConnectionManager> {
        self.connection_manager.get_connection().await
    }
}

#[async_trait]
impl CacheInvalidator for RedisInvalidator {
    #[instrument(skip(self), level = "debug")]
    async fn invalidate_key(&self, key: &str) -> CacheResult<bool> {
        let prefixed_key = self.prefixed_key(key);
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        let result: i64 = redis::cmd("DEL")
            .arg(&prefixed_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to invalidate key: {}", e))
            })?;

        Ok(result > 0)
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_keys(&self, keys: &[&str]) -> CacheResult<u64> {
        if keys.is_empty() {
            return Ok(0);
        }

        let prefixed_keys: Vec<String> = keys.iter().map(|k| self.prefixed_key(k)).collect();
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        let result: i64 = redis::cmd("DEL")
            .arg(prefixed_keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to invalidate keys: {}", e))
            })?;

        Ok(result as u64)
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_by_pattern(&self, pattern: &str) -> CacheResult<u64> {
        let prefixed_pattern = self.prefixed_key(&format!("{}*", pattern));
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        // First, scan for all keys matching the pattern
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&prefixed_pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to find keys by pattern: {}", e))
            })?;

        if keys.is_empty() {
            return Ok(0);
        }

        // Then delete all those keys
        let count: i64 = redis::cmd("DEL")
            .arg(keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to delete keys by pattern: {}", e))
            })?;

        Ok(count as u64)
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_all(&self) -> CacheResult<bool> {
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        // Get all keys with our prefix
        let prefix_pattern = format!("{}*", self.connection_manager.key_prefix());
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&prefix_pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to find all keys: {}", e))
            })?;

        if keys.is_empty() {
            return Ok(true);
        }

        // Delete all those keys
        let _: () = redis::cmd("DEL")
            .arg(keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to delete all keys: {}", e))
            })?;

        Ok(true)
    }

    #[instrument(skip(self), level = "debug")]
    async fn tag_key(&self, key: &str, tags: &[&str]) -> CacheResult<()> {
        if tags.is_empty() {
            return Ok(());
        }

        let prefixed_key = self.prefixed_key(key);
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        // For each tag, add the key to the tag's set
        for tag in tags {
            let tag_key = self.tag_key(tag);
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
        let prefixed_key = self.prefixed_key(key);
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        // Find all tag keys
        let tag_pattern = self.prefixed_key("tag:*");
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
                    .strip_prefix(&self.connection_manager.key_prefix())
                    .and_then(|s| s.strip_prefix("tag:"))
                    .unwrap_or("");

                tags.insert(tag.to_string());
            }
        }

        Ok(tags)
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_by_tag(&self, tag: &str) -> CacheResult<u64> {
        let tag_key = self.tag_key(tag);
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
impl CacheTtlManager for RedisInvalidator {
    #[instrument(skip(self), level = "debug")]
    async fn set_ttl(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
        let prefixed_key = self.prefixed_key(key);
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        let result: bool = redis::cmd("EXPIRE")
            .arg(&prefixed_key)
            .arg(ttl.as_secs())
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::InvalidationError(format!("Failed to set TTL: {}", e)))?;

        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    async fn get_ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        let prefixed_key = self.prefixed_key(key);
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        let ttl: i64 = redis::cmd("TTL")
            .arg(&prefixed_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::InvalidationError(format!("Failed to get TTL: {}", e)))?;

        // Redis returns -2 if the key does not exist and -1 if it has no expiration
        match ttl {
            -2 => Ok(None),                         // Key does not exist
            -1 => Ok(Some(Duration::from_secs(0))), // No expiration (infinite TTL)
            ttl if ttl >= 0 => Ok(Some(Duration::from_secs(ttl as u64))),
            _ => Err(CacheError::InvalidationError(format!(
                "Unexpected TTL value: {}",
                ttl
            ))),
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn remove_ttl(&self, key: &str) -> CacheResult<bool> {
        let prefixed_key = self.prefixed_key(key);
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        let result: bool = redis::cmd("PERSIST")
            .arg(&prefixed_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::InvalidationError(format!("Failed to remove TTL: {}", e)))?;

        Ok(result)
    }
}

#[async_trait]
impl CacheEventInvalidator for RedisInvalidator {
    #[instrument(skip(self), level = "debug")]
    async fn subscribe_to_invalidation_events(&self) -> CacheResult<()> {
        // For simplicity, this demonstration doesn't set up a separate subscription connection
        // A real implementation would use a dedicated connection in a separate task for pub/sub
        debug!("Subscribed to cache invalidation events");
        Ok(())
    }

    #[instrument(skip(self, payload), level = "debug")]
    async fn publish_invalidation_event(&self, event_type: &str, payload: &str) -> CacheResult<()> {
        let channel = self.prefixed_key(&format!("invalidation:{}", event_type));
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        let _: () = redis::cmd("PUBLISH")
            .arg(&channel)
            .arg(payload)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!(
                    "Failed to publish invalidation event: {}",
                    e
                ))
            })?;

        debug!("Published cache invalidation event: {}", event_type);
        Ok(())
    }

    #[instrument(skip(self, payload), level = "debug")]
    async fn process_invalidation_event(&self, event_type: &str, payload: &str) -> CacheResult<()> {
        debug!(
            "Processing cache invalidation event: {} with payload: {}",
            event_type, payload
        );

        // Process different event types
        match event_type {
            "key" => {
                self.invalidate_key(payload).await?;
            }
            "tag" => {
                self.invalidate_by_tag(payload).await?;
            }
            "pattern" => {
                self.invalidate_by_pattern(payload).await?;
            }
            "all" => {
                self.invalidate_all().await?;
            }
            _ => {
                debug!("Unknown invalidation event type: {}", event_type);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl CacheEntityTracker for RedisInvalidator {
    #[instrument(skip(self, entity), level = "debug")]
    async fn track_entity_change<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        entity_type: &str,
        entity_id: &str,
        entity: &T,
    ) -> CacheResult<()> {
        let entity_key = self.entity_key(entity_type, entity_id);
        let cache_key = self.prefixed_key(&format!("entity:{}:{}", entity_type, entity_id));

        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        // Store the serialized entity in the cache
        let serialized = serde_json::to_string(entity).map_err(|e| {
            CacheError::SerializationError(format!("Failed to serialize entity: {}", e))
        })?;

        let _: () = redis::cmd("SET")
            .arg(&cache_key)
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
        let entity_key = self.entity_key(entity_type, entity_id);
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
        let prefix = self.connection_manager.key_prefix();
        let result: Vec<String> = cache_keys
            .into_iter()
            .filter_map(|key| key.strip_prefix(prefix).map(ToString::to_string))
            .collect();

        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_entity(&self, entity_type: &str, entity_id: &str) -> CacheResult<u64> {
        let entity_key = self.entity_key(entity_type, entity_id);
        let cache_key = self.prefixed_key(&format!("entity:{}:{}", entity_type, entity_id));
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
                .arg(&cache_key)
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
            .arg(&cache_key)
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
