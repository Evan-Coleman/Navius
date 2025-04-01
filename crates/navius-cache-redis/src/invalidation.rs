use async_trait::async_trait;
use navius_cache::{
    error::{CacheError, CacheResult},
    invalidation::CacheInvalidation,
    operations::Cache,
    CacheKey, InvalidationStrategy,
};
use redis::{aio::ConnectionManager, cmd};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashSet, sync::Arc, time::Duration};
use tracing::{debug, error, instrument};

use crate::{
    connection::RedisConnectionManager,
    error::{error_helpers, RedisCacheError, RedisCacheResult},
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
impl<C: Cache> CacheInvalidation<C> for RedisInvalidator {
    #[instrument(skip(self), level = "debug")]
    async fn invalidate<K: CacheKey + 'static>(&self, key: K) -> CacheResult<bool> {
        let prefixed_key = self.prefixed_key(&key.to_string());
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
    async fn invalidate_by_pattern(&self, pattern: &str) -> CacheResult<usize> {
        let prefixed_pattern = self.prefixed_key(&format!("{}*", pattern));
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

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

        let count: i64 = redis::cmd("DEL")
            .arg(keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to delete keys by pattern: {}", e))
            })?;

        Ok(count as usize)
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_all(&self) -> CacheResult<()> {
        let mut conn = self
            .get_connection()
            .await
            .map_err(|e| CacheError::InvalidationError(e.to_string()))?;

        let prefix_pattern = format!("{}*", self.connection_manager.key_prefix());
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&prefix_pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to find all keys: {}", e))
            })?;

        if keys.is_empty() {
            return Ok(());
        }

        let _: () = redis::cmd("DEL")
            .arg(keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                CacheError::InvalidationError(format!("Failed to delete all keys: {}", e))
            })?;

        Ok(())
    }

    fn set_strategy(&mut self, _strategy: InvalidationStrategy) {
        unimplemented!("RedisInvalidator does not support setting invalidation strategy");
    }

    fn strategy(&self) -> &InvalidationStrategy {
        unimplemented!("RedisInvalidator does not currently track invalidation strategy");
    }
}

#[cfg(test)]
mod tests {
    // Tests would be here in a real implementation
    // They would use a real Redis instance or a mock
}
