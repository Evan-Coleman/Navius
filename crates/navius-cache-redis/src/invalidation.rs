//! Redis implementation for cache invalidation.

use crate::connection::RedisConnectionManager;
use crate::error::RedisCacheError;
use async_trait::async_trait;
use navius_cache::{
    error::{CacheError, CacheResult},
    invalidation::{CacheInvalidation, InvalidationStrategy},
    operations::CacheKey,
    RedisCache,
};
use redis::FromRedisValue;
use std::sync::Arc;
use tracing::instrument;

/// Redis invalidator implementation
pub struct RedisInvalidator<T> {
    connection_manager: Arc<RedisConnectionManager<T>>,
    strategy: InvalidationStrategy,
}

impl<T> RedisInvalidator<T> {
    /// Create a new Redis invalidator
    pub fn new(connection_manager: Arc<RedisConnectionManager<T>>) -> Self {
        Self {
            connection_manager,
            strategy: InvalidationStrategy::Immediate,
        }
    }
}

#[async_trait::async_trait]
impl<T> CacheInvalidation for RedisInvalidator<T>
where
    T: Send + Sync + 'static,
{
    #[instrument(skip(self), level = "debug")]
    async fn invalidate<K>(&self, key: K) -> Result<(), CacheError>
    where
        K: CacheKey + Send + 'static,
    {
        let key_str = key.to_string();
        match self
            .connection_manager
            .execute_command(&key_str, "DEL", |mut conn| async move {
                conn.del(&key_str).await
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(RedisCacheError::Connection(format!(
                "Failed to invalidate key {}: {}",
                key_str, err
            ))
            .into()),
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_by_pattern(&self, pattern: String) -> Result<(), CacheError> {
        let pattern_str = pattern.clone();
        match self
            .connection_manager
            .execute_command(&pattern, "KEYS", |mut conn| async move {
                let keys: Vec<String> = conn.keys(&pattern_str).await?;
                if !keys.is_empty() {
                    conn.del(keys).await?;
                }
                Ok(())
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(RedisCacheError::Connection(format!(
                "Failed to invalidate pattern {}: {}",
                pattern, err
            ))
            .into()),
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn invalidate_all(&self) -> Result<(), CacheError> {
        match self
            .connection_manager
            .execute_command("*", "FLUSHDB", |mut conn| async move {
                conn.flushdb(true).await
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(RedisCacheError::Connection(format!(
                "Failed to invalidate all keys: {}",
                err
            ))
            .into()),
        }
    }

    fn set_strategy(&mut self, strategy: InvalidationStrategy) {
        self.strategy = strategy;
    }

    fn strategy(&self) -> InvalidationStrategy {
        self.strategy
    }
}

#[cfg(test)]
mod tests {
    // Tests would be here in a real implementation
    // They would use a real Redis instance or a mock
}
