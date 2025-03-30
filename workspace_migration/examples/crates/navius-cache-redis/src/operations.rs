use async_trait::async_trait;
use redis::{AsyncCommands, cmd};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, instrument};

use navius_cache::{
    error::{CacheError, CacheResult},
    operations::Cache,
    serialization::{CacheSerializer, JsonSerializer},
};

use crate::{
    config::RedisCacheConfig,
    connection::RedisConnectionManager,
    error::{RedisCacheError, RedisCacheResult},
};

/// Redis cache implementation
#[derive(Clone)]
pub struct RedisCache {
    /// Connection manager
    connection_manager: RedisConnectionManager,
    /// Serializer for data conversion
    serializer: Arc<dyn CacheSerializer>,
}

impl RedisCache {
    /// Create a new Redis cache
    pub fn new(connection_manager: RedisConnectionManager) -> RedisCacheResult<Self> {
        Ok(Self {
            connection_manager,
            serializer: Arc::new(JsonSerializer),
        })
    }

    /// Create a new Redis cache with a custom serializer
    pub fn with_serializer(
        connection_manager: RedisConnectionManager,
        serializer: impl CacheSerializer + 'static,
    ) -> RedisCacheResult<Self> {
        Ok(Self {
            connection_manager,
            serializer: Arc::new(serializer),
        })
    }

    /// Get the connection manager
    pub fn connection_manager(&self) -> &RedisConnectionManager {
        &self.connection_manager
    }

    /// Get the serializer
    pub fn serializer(&self) -> &Arc<dyn CacheSerializer> {
        &self.serializer
    }
}

#[async_trait]
impl Cache for RedisCache {
    #[instrument(skip(self), level = "debug")]
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> CacheResult<Option<T>> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
        debug!("Getting value for key: {}", prefixed_key);

        let result: Option<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "GET", |mut conn| {
                redis::cmd("GET").arg(&prefixed_key).query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            Some(data) => {
                debug!("Found value for key: {}", prefixed_key);
                match self.serializer.deserialize(&data).await {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => {
                        error!(
                            "Failed to deserialize value for key {}: {:?}",
                            prefixed_key, e
                        );
                        Err(e)
                    }
                }
            }
            None => {
                debug!("No value found for key: {}", prefixed_key);
                Ok(None)
            }
        }
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
        debug!("Setting value for key: {}", prefixed_key);

        let serialized = self.serializer.serialize(value).await?;

        match ttl {
            Some(ttl) => self
                .connection_manager
                .execute_command(&prefixed_key, "SETEX", |mut conn| {
                    redis::cmd("SETEX")
                        .arg(&prefixed_key)
                        .arg(ttl.as_secs())
                        .arg(&serialized)
                        .query_async(&mut conn)
                })
                .await
                .map_err(|e| CacheError::from(e)),
            None => self
                .connection_manager
                .execute_command(&prefixed_key, "SET", |mut conn| {
                    redis::cmd("SET")
                        .arg(&prefixed_key)
                        .arg(&serialized)
                        .query_async(&mut conn)
                })
                .await
                .map_err(|e| CacheError::from(e)),
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn delete(&self, key: &str) -> CacheResult<bool> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
        debug!("Deleting key: {}", prefixed_key);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "DEL", |mut conn| {
                redis::cmd("DEL").arg(&prefixed_key).query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result > 0)
    }

    #[instrument(skip(self), level = "debug")]
    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
        debug!("Checking if key exists: {}", prefixed_key);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "EXISTS", |mut conn| {
                redis::cmd("EXISTS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result > 0)
    }

    #[instrument(skip(self), level = "debug")]
    async fn increment(&self, key: &str, amount: i64) -> CacheResult<i64> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
        debug!("Incrementing key: {} by {}", prefixed_key, amount);

        self.connection_manager
            .execute_command(&prefixed_key, "INCRBY", |mut conn| {
                redis::cmd("INCRBY")
                    .arg(&prefixed_key)
                    .arg(amount)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))
    }

    #[instrument(skip(self), level = "debug")]
    async fn decrement(&self, key: &str, amount: i64) -> CacheResult<i64> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
        debug!("Decrementing key: {} by {}", prefixed_key, amount);

        self.connection_manager
            .execute_command(&prefixed_key, "DECRBY", |mut conn| {
                redis::cmd("DECRBY")
                    .arg(&prefixed_key)
                    .arg(amount)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))
    }

    #[instrument(skip(self), level = "debug")]
    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
        debug!("Getting TTL for key: {}", prefixed_key);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "TTL", |mut conn| {
                redis::cmd("TTL").arg(&prefixed_key).query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            -2 => Ok(None), // Key does not exist
            -1 => Ok(None), // Key exists but has no TTL
            ttl => Ok(Some(Duration::from_secs(ttl as u64))),
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn set_ttl(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
        debug!("Setting TTL for key: {} to {:?}", prefixed_key, ttl);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "EXPIRE", |mut conn| {
                redis::cmd("EXPIRE")
                    .arg(&prefixed_key)
                    .arg(ttl.as_secs())
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result > 0)
    }
}
