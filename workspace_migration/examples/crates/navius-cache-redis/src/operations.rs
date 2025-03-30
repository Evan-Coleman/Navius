use async_trait::async_trait;
use redis::{AsyncCommands, cmd};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::time::Duration;

use navius_cache::{
    config::CacheConfig,
    error::{CacheError, CacheResult},
    operations::Cache,
    serialization::{CacheSerializer, JsonSerializer},
};

use crate::{
    config::RedisCacheConfig,
    connection::RedisConnectionManager,
    error::{RedisCacheError, RedisCacheResult, error_helpers},
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

    /// Get the prefixed key
    fn prefixed_key<K: AsRef<str>>(&self, key: K) -> String {
        self.connection_manager.prefixed_key(key)
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> CacheResult<Option<T>> {
        let prefixed_key = self.prefixed_key(key);

        let result: Option<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "GET", |mut conn| {
                redis::cmd("GET").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            Some(data) => match self.serializer.deserialize(&data).await {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let prefixed_key = self.prefixed_key(key);
        let serialized = self.serializer.serialize(value).await?;

        match ttl {
            Some(ttl) => self
                .connection_manager
                .execute_command(&prefixed_key, "SETEX", |mut conn| {
                    redis::cmd("SETEX")
                        .arg(&prefixed_key)
                        .arg(ttl.as_secs())
                        .arg(&serialized)
                        .query(&mut conn)
                })
                .await
                .map_err(|e| CacheError::from(e)),
            None => self
                .connection_manager
                .execute_command(&prefixed_key, "SET", |mut conn| {
                    redis::cmd("SET")
                        .arg(&prefixed_key)
                        .arg(&serialized)
                        .query(&mut conn)
                })
                .await
                .map_err(|e| CacheError::from(e)),
        }
    }

    // ... existing code ...
}
