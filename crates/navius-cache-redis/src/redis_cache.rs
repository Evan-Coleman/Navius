use crate::config::RedisCacheConfig;
use crate::connection::RedisConnectionPool;
use crate::error::{RedisCacheError, RedisCacheResult as CrateRedisCacheResult};
use crate::key::{KeyValidationOptions, validate_key};
use crate::metrics::OperationTimer;
use crate::operations::RedisOperations;
use crate::serialization::{SerializationFormat, SerializerImpl};
use async_trait::async_trait;
use metrics;
use navius_cache::cache::{Cache, CacheKey, CacheOperations, CacheOptions, CacheResult};
use navius_cache::error::CacheError;
use redis::{AsyncCommands, FromRedisValue, RedisError, RedisResult, ToRedisArgs};
use serde::{Serialize, de::DeserializeOwned};
use std::cmp::Eq;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, field::display, info, instrument, trace, warn};

/// Redis cache implementation
#[derive(Clone)]
pub struct RedisCache {
    /// The Redis connection pool
    pub(crate) pool: Arc<RedisConnectionPool>,
    /// The Redis cache configuration
    pub(crate) config: RedisCacheConfig,
    /// The serializer for values
    pub(crate) serializer: SerializerImpl,
    /// Key validation options
    key_validation: KeyValidationOptions,
    /// The serialization format
    format: SerializationFormat,
}

/// Check if a Redis server is available at the given URL
///
/// # Returns
///
/// `true` if the Redis server is available, `false` otherwise
pub async fn check_redis_connection(url: &str) -> bool {
    let client = match redis::Client::open(url) {
        Ok(client) => client,
        Err(_) => return false,
    };

    let connection = match client.get_async_connection().await {
        Ok(conn) => conn,
        Err(_) => return false,
    };

    let mut connection = connection;
    let ping: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut connection).await;

    match ping {
        Ok(response) => response == "PONG",
        Err(_) => false,
    }
}

impl RedisCache {
    /// Create a new Redis cache with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The Redis cache configuration
    ///
    /// # Returns
    ///
    /// A new `RedisCache` instance wrapped in a `Result`
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis server is not available or if the configuration is invalid
    pub async fn new(config: RedisCacheConfig) -> Result<Self, RedisCacheError> {
        // Validate the configuration
        config.validate()?;

        // Create the connection pool
        let pool = RedisConnectionPool::new(config.clone()).await?;

        // Create the serializer
        let serializer = SerializerImpl::new(config.serialization_format);

        // Create the cache
        let cache = Self {
            pool: Arc::new(pool),
            config: config.clone(),
            serializer,
            key_validation: KeyValidationOptions::default(),
            format: config.serialization_format,
        };

        // Test the connection
        match cache.health_check().await {
            Ok(_) => Ok(cache),
            Err(e) => Err(RedisCacheError::Connection(format!(
                "Failed to connect to Redis: {}",
                e
            ))),
        }
    }

    /// Get the Redis cache configuration
    pub fn config(&self) -> &RedisCacheConfig {
        &self.config
    }

    /// Get the Redis connection pool
    pub fn pool(&self) -> &RedisConnectionPool {
        &self.pool
    }

    /// Get the serializer for values
    pub fn serializer(&self) -> &SerializerImpl {
        &self.serializer
    }

    /// Validate a cache key
    ///
    /// # Arguments
    ///
    /// * `key` - The key to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if the key is valid, `Err(RedisCacheError)` otherwise
    fn validate_key<K>(&self, key: &K) -> CrateRedisCacheResult<()>
    where
        K: AsRef<str> + Debug + std::fmt::Display + Send + Sync + 'static,
    {
        validate_key(key, &self.key_validation)
    }

    /// Get the time-to-live from options or default
    pub(crate) fn get_ttl(&self, options: Option<CacheOptions>) -> Option<Duration> {
        options
            .and_then(|opts| opts.ttl)
            .or_else(|| self.config.default_ttl)
    }

    /// Convert and validate a cache key, applying prefix
    pub(crate) fn key_to_string<K: CacheKey>(&self, key: K) -> Result<String, RedisCacheError> {
        let key_str = key.to_string();
        let key_opts = self.key_validation.clone();

        match validate_key(&key_str, &key_opts) {
            Ok(_) => {
                // Apply prefix here
                if let Some(prefix) = &self.config.key_prefix {
                    if !prefix.is_empty() {
                        Ok(format!("{}:{}", prefix, key_str))
                    } else {
                        Ok(key_str)
                    }
                } else {
                    Ok(key_str)
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Helper method to clear keys with a specific prefix
    pub(crate) async fn clear_with_prefix(&self) -> CacheResult<()> {
        let timer = OperationTimer::new("cache_clear_with_prefix");
        debug!("Clearing all keys with prefix");

        // Use FLUSHDB or delete all keys with the prefix
        if self
            .config
            .key_prefix
            .as_ref()
            .map_or(true, |p| p.is_empty())
        {
            // Use FLUSHDB if no prefix is set (dangerous operation)
            let result: Result<redis::RedisResult<String>, RedisCacheError> = self
                .pool
                .execute(|conn| Box::pin(redis::cmd("FLUSHDB").query_async(conn)))
                .await;

            match result {
                Ok(Ok(_)) => {
                    timer.record_success();
                    Ok(())
                }
                Ok(Err(redis_err)) => {
                    let err = RedisCacheError::from(redis_err);
                    timer.record_error(&err);
                    Err(err.into())
                }
                Err(cache_err) => {
                    timer.record_error(&cache_err);
                    Err(cache_err.into())
                }
            }
        } else {
            // Use SCAN + DEL to delete keys with prefix
            let pattern = if let Some(prefix) = &self.config.key_prefix {
                if !prefix.is_empty() {
                    format!("{}:*", prefix)
                } else {
                    "*".to_string()
                }
            } else {
                "*".to_string()
            };

            let batch_size = 100;
            let mut cursor: u64 = 0;
            let mut total_deleted: usize = 0;

            loop {
                // Use SCAN to find keys with prefix
                let result: Result<redis::RedisResult<(u64, Vec<String>)>, RedisCacheError> = self
                    .pool
                    .execute({
                        let pattern = pattern.clone();
                        move |conn| {
                            let mut cmd = redis::cmd("SCAN");
                            Box::pin(
                                cmd.arg(cursor)
                                    .arg("MATCH")
                                    .arg(&pattern)
                                    .arg("COUNT")
                                    .arg(batch_size)
                                    .query_async(conn),
                            )
                        }
                    })
                    .await;

                match result {
                    Ok(Ok((next_cursor, keys))) => {
                        cursor = next_cursor;

                        // Delete the found keys if any
                        if !keys.is_empty() {
                            let del_result: Result<redis::RedisResult<usize>, RedisCacheError> =
                                self.pool
                                    .execute(move |conn| {
                                        let keys_clone = keys.clone();
                                        Box::pin(
                                            redis::cmd("DEL").arg(&keys_clone).query_async(conn),
                                        )
                                    })
                                    .await;

                            match del_result {
                                Ok(Ok(count)) => {
                                    total_deleted += count;
                                    debug!(
                                        deleted = %count,
                                        "Deleted keys in batch"
                                    );
                                }
                                Ok(Err(redis_err)) => {
                                    let err = RedisCacheError::from(redis_err);
                                    timer.record_error(&err);
                                    return Err(CacheError::OperationError(format!(
                                        "Failed to delete keys: {}",
                                        err
                                    )));
                                }
                                Err(cache_err) => {
                                    timer.record_error(&cache_err);
                                    return Err(CacheError::OperationError(format!(
                                        "Failed to execute delete keys: {}",
                                        cache_err
                                    )));
                                }
                            }
                        }

                        // Break when we've processed all keys
                        if cursor == 0 {
                            break;
                        }
                    }
                    Ok(Err(redis_err)) => {
                        let err = RedisCacheError::from(redis_err);
                        timer.record_error(&err);
                        return Err(CacheError::OperationError(format!(
                            "Failed to scan keys: {}",
                            err
                        )));
                    }
                    Err(cache_err) => {
                        timer.record_error(&cache_err);
                        return Err(CacheError::OperationError(format!(
                            "Failed to execute scan keys: {}",
                            cache_err
                        )));
                    }
                }
            }

            timer.record_success();
            debug!("Cleared all keys with prefix");
            Ok(())
        }
    }

    pub(crate) async fn _get<K, V>(&self, key: K) -> CrateRedisCacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        let timer = OperationTimer::new("cache_get");
        metrics::counter!("cache.get.total");

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.get.error");
                return Err(e.into());
            }
        };

        // Get value from Redis
        let result: redis::RedisResult<Option<String>> = match self
            .pool
            .execute(move |conn| {
                let mut cmd = redis::cmd("GET");
                let mut cmd_clone = cmd.clone();
                Box::pin(async move { cmd_clone.arg(&key_str).query_async(conn).await })
            })
            .await
        {
            Ok(val) => Ok::<Option<String>, redis::RedisError>(val),
            Err(e) => Err(redis::RedisError::from(e)), // Convert to RedisError
        };

        match result {
            Ok(Some(value)) => {
                // Deserialize the value
                match self.serializer.deserialize::<V>(value.as_bytes()).await {
                    Ok(value) => {
                        timer.record_success();
                        metrics::counter!("cache.get.hit");
                        Ok(Some(value))
                    }
                    Err(e) => {
                        timer.record_error(&e);
                        metrics::counter!("cache.get.error");
                        Err(e.into())
                    }
                }
            }
            Ok(None) => {
                timer.record_success();
                metrics::counter!("cache.get.miss");
                Ok(None)
            }
            Err(e) => {
                let err = RedisCacheError::from(e);
                timer.record_error(&err);
                metrics::counter!("cache.get.error");
                Err(err.into())
            }
        }
    }

    pub(crate) async fn _get_many<K, V>(
        &self,
        keys: Vec<K>,
    ) -> CrateRedisCacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        let timer = OperationTimer::new("cache_get_many");
        metrics::counter!("cache.get_many.total");

        if keys.is_empty() {
            timer.record_success();
            return Ok(Vec::new());
        }

        // Convert keys to strings and validate
        let mut key_strings = Vec::with_capacity(keys.len());
        for key in keys {
            match self.key_to_string(key) {
                Ok(k) => key_strings.push(k),
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.get_many.error");
                    return Err(e.into());
                }
            }
        }

        // Get values from Redis
        let result: redis::RedisResult<Vec<Option<String>>> = match self
            .pool
            .execute(move |conn| {
                let mut cmd = redis::cmd("MGET");
                let mut cmd_clone = cmd.clone();
                Box::pin(async move { cmd_clone.arg(&key_strings).query_async(conn).await })
            })
            .await
        {
            Ok(val) => Ok::<Vec<Option<String>>, redis::RedisError>(val),
            Err(e) => Err(redis::RedisError::from(e)),
        };

        match result {
            Ok(values) => {
                let mut result = Vec::with_capacity(values.len());
                for value_opt in values {
                    if let Some(value_str) = value_opt {
                        let deserialized = self
                            .serializer
                            .deserialize::<V>(value_str.as_bytes())
                            .await?;
                        result.push(Some(deserialized));
                    } else {
                        result.push(None);
                    }
                }
                timer.record_success();
                metrics::counter!("cache.get_many.success");
                Ok(result)
            }
            Err(e) => {
                let err = RedisCacheError::from(e);
                timer.record_error(&err);
                metrics::counter!("cache.get_many.error");
                Err(err.into())
            }
        }
    }
}

impl Cache for RedisCache {}

#[async_trait]
impl CacheOperations for RedisCache {
    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        self._get_internal(key).await.map_err(|e| e.into())
    }

    #[instrument(skip(self, keys), fields(key_count = keys.len()), level = "info")]
    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        // Keep track of original keys for ordering the final result
        let key_map: HashMap<String, K> =
            keys.into_iter().map(|k| (self.prefix_key(&k), k)).collect();
        let prefixed_keys: Vec<String> = key_map.keys().cloned().collect();

        let internal_result: HashMap<String, Option<V>> =
            self._get_many_internal(prefixed_keys).await?;

        // Map the HashMap result back to a Vec in the original order
        let mut results = Vec::with_capacity(key_map.len());
        for (prefixed_key, _original_key) in &key_map {
            results.push(internal_result.get(prefixed_key).cloned().flatten());
        }

        Ok(results)
    }

    #[instrument(skip(self, key, value), fields(key = %display(key.to_string())), level = "info")]
    async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        let ttl = self.get_ttl(options);
        self._set_internal(key, value.clone(), ttl)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, entries), fields(entry_count = entries.len()), level = "info")]
    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<CacheOptions>,
    ) -> CacheResult<()>
    where
        K: CacheKey + Eq + Hash + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        let ttl = self.get_ttl(options);
        // Convert Vec<(K, V)> to HashMap<K, V>
        let items: HashMap<K, V> = entries.into_iter().collect();
        self._set_many_internal(items, ttl)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        self.delete_internal(key).await.map_err(|e| e.into())
    }

    #[instrument(skip(self, keys), fields(key_count = keys.len()), level = "info")]
    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        self.delete_many_internal(keys).await.map_err(|e| e.into())
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        self.exists_internal(key).await.map_err(|e| e.into())
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
    {
        debug!(
            key = %key.to_string(),
            amount = %amount,
            "RedisCache: increment operation is not supported due to serialization concerns."
        );
        Err(CacheError::UnsupportedOperation(
            "RedisCache does not support generic atomic increment due to serialization".to_string(),
        ))
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        self.expire_internal(key, ttl).await.map_err(|e| e.into())
    }

    #[instrument(skip(self), level = "info")]
    async fn clear(&self) -> CacheResult<()> {
        debug!("RedisCache: Clearing cache (using prefix or FLUSHDB)");
        self.clear_with_prefix().await
    }

    #[instrument(skip(self), level = "info")]
    async fn health_check(&self) -> CacheResult<()> {
        debug!("RedisCache: Performing health check (PING)");
        let result: CrateRedisCacheResult<String> = self
            .pool
            .execute(|conn| {
                Box::pin(async move { redis::cmd("PING").query_async::<String>(conn).await })
            })
            .await;
        match result {
            Ok(ref pong) if pong == "PONG" => Ok(()),
            Ok(other) => Err(CacheError::OperationError(format!(
                "Health check failed: Expected PONG, got {}",
                other
            ))),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(skip(self, key, value), fields(key = %display(key.to_string())), level = "info")]
    async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        self.list_push_right_internal(key, value.clone())
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key, values), fields(key = %display(key.to_string()), value_count = values.len()), level = "info")]
    async fn list_push_right_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        self.list_push_right_many_internal(key, values.to_vec())
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key, value), fields(key = %display(key.to_string())), level = "info")]
    async fn list_push_left<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + Clone + 'static,
    {
        self.list_push_left_internal(key, value.clone())
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key, values), fields(key = %display(key.to_string()), value_count = values.len()), level = "info")]
    async fn list_push_left_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static + Clone,
    {
        self.list_push_left_many_internal(key, values.to_vec())
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn list_pop_right<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        self.list_pop_right_internal(key)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        self.list_pop_left_internal(key).await.map_err(|e| e.into())
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        self.list_range_internal(key, start, stop)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn list_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        self.list_length_internal(key).await.map_err(|e| e.into())
    }

    #[instrument(skip(self, key, value), fields(key = %display(key.to_string())), level = "info")]
    async fn list_remove<K, V>(&self, key: K, count: isize, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static + Clone,
    {
        self.list_remove_internal(key, count, value.clone())
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn list_trim<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
    where
        K: CacheKey + 'static,
    {
        self.list_trim_internal(key, start, stop)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key, value), fields(key = %display(key.to_string())), level = "info")]
    async fn list_set<K, V>(&self, key: K, index: isize, value: &V) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static + Clone,
    {
        self.list_set_internal(key, index, value.clone())
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key, field), fields(key = %display(key.to_string()), field = %display(field.to_string())), level = "info")]
    async fn hash_get<K, F, V>(&self, key: K, field: F) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        F: CacheKey + redis::ToRedisArgs + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        self.hash_get_internal(key, field)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key, field, value), fields(key = %display(key.to_string()), field = %display(field.to_string())), level = "info")]
    async fn hash_set<K, F, V>(&self, key: K, field: F, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + redis::ToRedisArgs + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + 'static + Clone,
    {
        self.hash_set_internal(key, field, value.clone())
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key, fields), fields(key = %display(key.to_string()), field_count = fields.len()), level = "info")]
    async fn hash_get_many<K, F, V>(&self, key: K, fields: Vec<F>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        F: CacheKey
            + redis::ToRedisArgs
            + std::fmt::Debug
            + Clone
            + Eq
            + std::hash::Hash
            + redis::FromRedisValue
            + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        if fields.is_empty() {
            return Ok(Vec::new());
        }
        // Keep track of original fields for ordering the final result
        let field_map: HashMap<F, usize> = fields
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, f)| (f, i))
            .collect();
        let original_field_order: Vec<F> = fields;

        let internal_result: HashMap<F, Option<V>> = self
            .hash_get_many_internal(key, original_field_order.clone())
            .await?;

        // Map the HashMap result back to a Vec in the original order
        let mut results = vec![None; original_field_order.len()];
        for (field, value_opt) in internal_result {
            if let Some(index) = field_map.get(&field) {
                results[*index] = value_opt;
            }
        }

        Ok(results)
    }

    #[instrument(skip(self, key, entries), fields(key = %display(key.to_string()), entry_count = entries.len()), level = "info")]
    async fn hash_set_many<K, F, V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        F: CacheKey + redis::ToRedisArgs + std::fmt::Debug + Eq + std::hash::Hash + 'static,
        V: Serialize + Send + Sync + 'static + Clone,
    {
        // Convert Vec<(F, V)> to HashMap<F, V>
        let items: HashMap<F, V> = entries.into_iter().collect();
        // Map Ok(usize) -> Ok(())
        self.hash_set_many_internal(key, items)
            .await
            .map(|_| ())
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key, field), fields(key = %display(key.to_string()), field = %display(field.to_string())), level = "info")]
    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + redis::ToRedisArgs + std::fmt::Debug + 'static,
    {
        self.hash_exists_internal(key, field)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key, fields), fields(key = %display(key.to_string()), field_count = fields.len()), level = "info")]
    async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        F: CacheKey + redis::ToRedisArgs + std::fmt::Debug + 'static,
    {
        self.hash_delete_many_internal(key, fields)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        match self.hash_get_all_internal::<K, String, V>(key).await {
            Ok(map) => Ok(map.into_iter().collect()),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn hash_keys<K>(&self, key: K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + 'static,
    {
        self.hash_keys_internal::<K, String>(key)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn hash_values<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        self.hash_values_internal(key).await.map_err(|e| e.into())
    }

    #[instrument(skip(self, key, field), fields(key = %display(key.to_string()), field = %display(field.to_string())), level = "info")]
    async fn hash_increment<K, F>(&self, key: K, field: F, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
        F: CacheKey + redis::ToRedisArgs + std::fmt::Debug + 'static,
    {
        debug!(
            key = %key.to_string(),
            field = %field.to_string(),
            amount = %amount,
            "RedisCache: hash_increment operation is not supported due to serialization concerns."
        );
        Err(CacheError::UnsupportedOperation(
            "RedisCache does not support generic atomic hash increment due to serialization"
                .to_string(),
        ))
    }

    #[instrument(skip(self, key), fields(key = %display(key.to_string())), level = "info")]
    async fn hash_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        self.hash_length_internal(key).await.map_err(|e| e.into())
    }

    #[instrument(skip(self, key, values), fields(key = %display(key.to_string()), value_count = values.len()), level = "info")]
    async fn set_add<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.set_add_internal(key, values)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key, values), fields(key = %display(key.to_string()), value_count = values.len()), level = "info")]
    async fn set_remove<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.set_remove_internal(key, values)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, key, value), fields(key = %display(key.to_string())), level = "info")]
    async fn set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        self.set_contains_internal(key, value)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self), level = "info")]
    async fn set_members<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        self.set_members_internal(key).await.map_err(|e| e.into())
    }

    #[instrument(skip(self), level = "info")]
    async fn set_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
    {
        self.set_length_internal(key).await.map_err(|e| e.into())
    }

    #[instrument(skip(self, keys), fields(key_count = keys.len()), level = "info")]
    async fn set_intersection<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        self.set_intersection_internal(keys)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, destination, keys), fields(dest = %display(destination.to_string()), key_count = keys.len()), level = "info")]
    async fn set_intersection_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        self.set_intersection_store_internal(destination, keys)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, keys), fields(key_count = keys.len()), level = "info")]
    async fn set_union<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        self.set_union_internal(keys).await.map_err(|e| e.into())
    }

    #[instrument(skip(self, destination, keys), fields(dest = %display(destination.to_string()), key_count = keys.len()), level = "info")]
    async fn set_union_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        self.set_union_store_internal(destination, keys)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, keys), fields(key_count = keys.len()), level = "info")]
    async fn set_difference<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        self.set_difference_internal(keys)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, destination, keys), fields(dest = %display(destination.to_string()), key_count = keys.len()), level = "info")]
    async fn set_difference_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        self.set_difference_store_internal(destination, keys)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self), level = "info")]
    async fn set_random_members<K, V>(&self, key: K, count: usize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + 'static,
    {
        self.set_random_members_internal(key, count)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(skip(self, _key, _items), level = "info")]
    async fn zset_add<K, V>(&self, _key: K, _items: Vec<(f64, V)>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_add not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _key, _members), level = "info")]
    async fn zset_remove<K, V>(&self, _key: K, _members: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_remove not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _key, _member), level = "info")]
    async fn zset_score<K, V>(&self, _key: K, _member: &V) -> CacheResult<Option<f64>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_score not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _key, _member, _increment), level = "info")]
    async fn zset_increment_score<K, V>(
        &self,
        _key: K,
        _member: &V,
        _increment: f64,
    ) -> CacheResult<f64>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_increment_score not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _key, _start, _stop), level = "info")]
    async fn zset_range<K, V>(&self, _key: K, _start: isize, _stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_range not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _key, _start, _stop), level = "info")]
    async fn zset_range_with_scores<K, V>(
        &self,
        _key: K,
        _start: isize,
        _stop: isize,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_range_with_scores not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _key, _min, _max), level = "info")]
    async fn zset_range_by_score<K, V>(&self, _key: K, _min: f64, _max: f64) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_range_by_score not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _key, _min, _max), level = "info")]
    async fn zset_range_by_score_with_scores<K, V>(
        &self,
        _key: K,
        _min: f64,
        _max: f64,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_range_by_score_with_scores not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _key, _member), level = "info")]
    async fn zset_rank<K, V>(&self, _key: K, _member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_rank not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _key, _member), level = "info")]
    async fn zset_reverse_rank<K, V>(&self, _key: K, _member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_reverse_rank not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _key), level = "info")]
    async fn zset_length<K>(&self, _key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_length not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _key, _min, _max), level = "info")]
    async fn zset_count<K>(&self, _key: K, _min: f64, _max: f64) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_count not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _destination, _keys, _weights, _aggregate), level = "info")]
    async fn zset_intersection_store<K, D>(
        &self,
        _destination: D,
        _keys: Vec<K>,
        _weights: Option<Vec<f64>>,
        _aggregate: Option<String>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_intersection_store not implemented".to_string(),
        ))
    }

    #[instrument(skip(self, _destination, _keys, _weights, _aggregate), level = "info")]
    async fn zset_union_store<K, D>(
        &self,
        _destination: D,
        _keys: Vec<K>,
        _weights: Option<Vec<f64>>,
        _aggregate: Option<String>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        Err(CacheError::UnsupportedOperation(
            "RedisCache zset_union_store not implemented".to_string(),
        ))
    }
}
