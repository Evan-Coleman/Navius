use crate::config::CacheConfig;
use crate::error::{CacheError, CacheResult};
#[cfg(feature = "metrics")]
use crate::metrics::{CacheOperation, CacheTimer};
use crate::operations::{Cache, CacheKey, CacheOperations, CacheOptions};
use async_trait::async_trait;
use redis::{Client, RedisResult, aio::ConnectionManager};
use serde::{Serialize, de::DeserializeOwned};
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, instrument};

/// Redis-specific configuration
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,
    /// Key prefix
    pub prefix: String,
    /// Default TTL
    pub default_ttl: Duration,
    /// Maximum number of connections
    pub max_connections: usize,
}

impl From<&CacheConfig> for RedisConfig {
    fn from(config: &CacheConfig) -> Self {
        Self {
            url: config.url.clone(),
            prefix: config.prefix.clone(),
            default_ttl: config.default_ttl,
            max_connections: config.max_connections,
        }
    }
}

/// Redis cache implementation
#[derive(Clone)]
pub struct RedisCache {
    /// Redis connection manager
    connection: ConnectionManager,
    /// Cache configuration
    config: Arc<CacheConfig>,
}

impl std::fmt::Debug for RedisCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisCache")
            .field("config", &self.config)
            .finish()
    }
}

impl RedisCache {
    /// Create a new Redis cache
    #[instrument(skip(config))]
    pub async fn new(config: CacheConfig) -> CacheResult<Self> {
        debug!("Connecting to Redis at {}", config.url);

        let client = Client::open(config.url.as_str()).map_err(|e| {
            error!("Failed to create Redis client: {}", e);
            CacheError::ConnectionError(format!("Failed to create Redis client: {}", e))
        })?;

        let connection = ConnectionManager::new(client).await.map_err(|e| {
            error!("Failed to create Redis connection manager: {}", e);
            CacheError::ConnectionError(format!("Failed to connect to Redis: {}", e))
        })?;

        debug!("Successfully connected to Redis");

        Ok(Self {
            connection,
            config: Arc::new(config),
        })
    }

    /// Get the prefixed key
    fn prefixed_key<K: CacheKey>(&self, key: K) -> String {
        format!("{}{}", self.config.prefix, key.to_string())
    }

    /// Get TTL from options or default
    fn get_ttl(&self, options: Option<CacheOptions>) -> Option<Duration> {
        options
            .and_then(|opts| opts.ttl)
            .or(Some(self.config.default_ttl))
    }

    /// Serialize a value to a string
    fn serialize<V: Serialize>(&self, value: &V) -> CacheResult<String> {
        serde_json::to_string(value).map_err(|e| {
            error!("Failed to serialize value: {}", e);
            CacheError::SerializationError(format!("Failed to serialize value: {}", e))
        })
    }

    /// Deserialize a string to a value
    fn deserialize<V: DeserializeOwned>(&self, data: String) -> CacheResult<V> {
        serde_json::from_str(&data).map_err(|e| {
            error!("Failed to deserialize value: {}", e);
            CacheError::SerializationError(format!("Failed to deserialize value: {}", e))
        })
    }
}

impl Cache for RedisCache {}

#[async_trait]
impl CacheOperations for RedisCache {
    #[instrument(skip(self, key))]
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let prefixed_key = self.prefixed_key(key);
        debug!("Getting value for key: {}", prefixed_key);

        #[cfg(feature = "metrics")]
        let timer = CacheTimer::new(CacheOperation::Get, "redis");

        let result: RedisResult<Option<String>> = redis::cmd("GET")
            .arg(&prefixed_key)
            .query_async(&mut self.connection.clone())
            .await;

        match result {
            Ok(Some(data)) => {
                debug!("Found value for key: {}", prefixed_key);
                #[cfg(feature = "metrics")]
                timer.hit();
                self.deserialize(data).map(Some)
            }
            Ok(None) => {
                debug!("No value found for key: {}", prefixed_key);
                #[cfg(feature = "metrics")]
                timer.miss();
                Ok(None)
            }
            Err(e) => {
                error!("Failed to get value for key {}: {}", prefixed_key, e);
                #[cfg(feature = "metrics")]
                timer.error();
                Err(e.into())
            }
        }
    }

    #[instrument(skip(self, keys))]
    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let prefixed_keys: Vec<String> =
            keys.into_iter().map(|key| self.prefixed_key(key)).collect();

        debug!("Getting multiple values for keys: {:?}", prefixed_keys);

        let result: RedisResult<Vec<Option<String>>> = redis::cmd("MGET")
            .arg(&prefixed_keys)
            .query_async(&mut self.connection.clone())
            .await;

        match result {
            Ok(values) => {
                debug!("Found {} values", values.len());

                let mut result = Vec::with_capacity(values.len());

                for value in values {
                    match value {
                        Some(data) => match self.deserialize(data) {
                            Ok(deserialized) => result.push(Some(deserialized)),
                            Err(e) => {
                                error!("Failed to deserialize value: {}", e);
                                return Err(e);
                            }
                        },
                        None => result.push(None),
                    }
                }

                Ok(result)
            }
            Err(e) => {
                error!("Failed to get multiple values: {}", e);
                Err(e.into())
            }
        }
    }

    #[instrument(skip(self, key, value, options))]
    async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let prefixed_key = self.prefixed_key(key);
        debug!("Setting value for key: {}", prefixed_key);

        #[cfg(feature = "metrics")]
        let timer = CacheTimer::new(CacheOperation::Set, "redis");

        let serialized = self.serialize(value)?;
        let ttl = self.get_ttl(options);

        let result: RedisResult<()> = match ttl {
            Some(ttl) => {
                redis::cmd("SET")
                    .arg(&prefixed_key)
                    .arg(serialized)
                    .arg("PX")
                    .arg(ttl.as_millis() as u64)
                    .query_async(&mut self.connection.clone())
                    .await
            }
            None => {
                redis::cmd("SET")
                    .arg(&prefixed_key)
                    .arg(serialized)
                    .query_async(&mut self.connection.clone())
                    .await
            }
        };

        match result {
            Ok(_) => {
                debug!("Successfully set value for key: {}", prefixed_key);
                #[cfg(feature = "metrics")]
                timer.success();
                Ok(())
            }
            Err(e) => {
                error!("Failed to set value for key {}: {}", prefixed_key, e);
                #[cfg(feature = "metrics")]
                timer.error();
                Err(e.into())
            }
        }
    }

    #[instrument(skip(self, entries, options))]
    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<CacheOptions>,
    ) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if entries.is_empty() {
            return Ok(());
        }

        // For simplicity, we'll just set each entry individually for now
        // In a production implementation, you might want to use MSET for entries without TTL
        // and pipeline for entries with TTL
        for (key, value) in entries {
            self.set(key, &value, options.clone()).await?;
        }

        Ok(())
    }

    #[instrument(skip(self, key))]
    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        let prefixed_key = self.prefixed_key(key);
        debug!("Deleting key: {}", prefixed_key);

        let result: RedisResult<i64> = redis::cmd("DEL")
            .arg(&prefixed_key)
            .query_async(&mut self.connection.clone())
            .await;

        match result {
            Ok(count) => {
                debug!("Deleted {} instances of key: {}", count, prefixed_key);
                Ok(count > 0)
            }
            Err(e) => {
                error!("Failed to delete key {}: {}", prefixed_key, e);
                Err(e.into())
            }
        }
    }

    #[instrument(skip(self, keys))]
    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        if keys.is_empty() {
            return Ok(0);
        }

        let prefixed_keys: Vec<String> =
            keys.into_iter().map(|key| self.prefixed_key(key)).collect();

        debug!("Deleting multiple keys: {:?}", prefixed_keys);

        let result: RedisResult<i64> = redis::cmd("DEL")
            .arg(&prefixed_keys)
            .query_async(&mut self.connection.clone())
            .await;

        match result {
            Ok(count) => {
                debug!("Deleted {} keys", count);
                Ok(count as usize)
            }
            Err(e) => {
                error!("Failed to delete multiple keys: {}", e);
                Err(e.into())
            }
        }
    }

    #[instrument(skip(self, key))]
    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        let prefixed_key = self.prefixed_key(key);
        debug!("Checking if key exists: {}", prefixed_key);

        let result: RedisResult<i64> = redis::cmd("EXISTS")
            .arg(&prefixed_key)
            .query_async(&mut self.connection.clone())
            .await;

        match result {
            Ok(count) => {
                debug!("Key {} exists: {}", prefixed_key, count > 0);
                Ok(count > 0)
            }
            Err(e) => {
                error!("Failed to check if key {} exists: {}", prefixed_key, e);
                Err(e.into())
            }
        }
    }

    #[instrument(skip(self, key))]
    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
    {
        let prefixed_key = self.prefixed_key(key);
        debug!("Incrementing key {} by {}", prefixed_key, amount);

        let result: RedisResult<i64> = redis::cmd("INCRBY")
            .arg(&prefixed_key)
            .arg(amount)
            .query_async(&mut self.connection.clone())
            .await;

        match result {
            Ok(value) => {
                debug!("Key {} incremented to {}", prefixed_key, value);
                Ok(value)
            }
            Err(e) => {
                error!("Failed to increment key {}: {}", prefixed_key, e);
                Err(e.into())
            }
        }
    }

    #[instrument(skip(self, key))]
    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        let prefixed_key = self.prefixed_key(key);
        debug!(
            "Setting expiry for key {} to {} seconds",
            prefixed_key,
            ttl.as_secs()
        );

        let result: RedisResult<i64> = redis::cmd("EXPIRE")
            .arg(&prefixed_key)
            .arg(ttl.as_secs())
            .query_async(&mut self.connection.clone())
            .await;

        match result {
            Ok(status) => {
                debug!("Expiry set for key {}: {}", prefixed_key, status > 0);
                Ok(status > 0)
            }
            Err(e) => {
                error!("Failed to set expiry for key {}: {}", prefixed_key, e);
                Err(e.into())
            }
        }
    }

    #[instrument(skip(self))]
    async fn clear(&self) -> CacheResult<()> {
        debug!("Clearing all keys with prefix: {}", self.config.prefix);

        // We'll only clear keys with our prefix for safety
        let pattern = format!("{}*", self.config.prefix);

        // First, find all keys with our prefix
        let keys: RedisResult<Vec<String>> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut self.connection.clone())
            .await;

        match keys {
            Ok(keys) => {
                if keys.is_empty() {
                    debug!("No keys found with prefix {}", self.config.prefix);
                    return Ok(());
                }

                debug!(
                    "Found {} keys with prefix {}",
                    keys.len(),
                    self.config.prefix
                );

                // Then delete all those keys
                let result: RedisResult<i64> = redis::cmd("DEL")
                    .arg(&keys)
                    .query_async(&mut self.connection.clone())
                    .await;

                match result {
                    Ok(count) => {
                        debug!("Deleted {} keys with prefix {}", count, self.config.prefix);
                        Ok(())
                    }
                    Err(e) => {
                        error!(
                            "Failed to delete keys with prefix {}: {}",
                            self.config.prefix, e
                        );
                        Err(e.into())
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to find keys with prefix {}: {}",
                    self.config.prefix, e
                );
                Err(e.into())
            }
        }
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> CacheResult<()> {
        debug!("Performing Redis health check");

        #[cfg(feature = "metrics")]
        let timer = CacheTimer::new(CacheOperation::HealthCheck, "redis");

        let result: RedisResult<String> = redis::cmd("PING")
            .query_async(&mut self.connection.clone())
            .await;

        match result {
            Ok(response) => {
                if response == "PONG" {
                    debug!("Redis health check successful");
                    #[cfg(feature = "metrics")]
                    timer.success();
                    Ok(())
                } else {
                    error!(
                        "Redis health check failed: unexpected response: {}",
                        response
                    );
                    #[cfg(feature = "metrics")]
                    timer.error();
                    Err(CacheError::ConnectionError(
                        "Unexpected response from Redis".to_string(),
                    ))
                }
            }
            Err(e) => {
                error!("Redis health check failed: {}", e);
                #[cfg(feature = "metrics")]
                timer.error();
                Err(e.into())
            }
        }
    }
}
