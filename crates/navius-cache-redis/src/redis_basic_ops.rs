#![allow(unused_imports)]
use super::RedisCache;
use crate::config::RedisCacheConfig;
use crate::connection::RedisConnectionPool;
use crate::error::{RedisCacheError, RedisCacheResult};
use crate::key::{KeyValidationOptions, validate_key};
use crate::metrics::OperationTimer;
use crate::serialization::{SerializationFormat, SerializerImpl};
use async_trait::async_trait;
use metrics;
use navius_cache::cache::{Cache, CacheKey, CacheOperations, CacheOptions, CacheResult};
use navius_cache::error::CacheError;
use redis::{AsyncCommands, RedisError, RedisResult, ToRedisArgs, Value};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, instrument, trace, warn};

// Basic key/value and management operations will be moved here

impl RedisCache {
    pub async fn health_check_internal(&self) -> CacheResult<()> {
        let timer = OperationTimer::new("cache_health_check");
        let result = self.pool.health_check().await;
        match result {
            Ok(_) => {
                timer.record_success();
                Ok(())
            }
            Err(err) => {
                timer.record_error(&err);
                Err(CacheError::from(err))
            }
        }
    }

    pub async fn get_internal<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let val: Result<Option<V>, _> = self._get::<K, V>(key).await;
        val.map_err(|e| e.into())
    }

    pub async fn get_many_internal<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let vals: Result<Vec<Option<V>>, _> = self._get_many::<K, V>(keys).await;
        vals.map_err(|e| e.into())
    }

    #[instrument(skip(self, value), level = "trace")]
    pub(crate) async fn _set_internal<K, V>(
        &self,
        key: K,
        value: V,
        ttl: Option<Duration>,
    ) -> CacheResult<()>
    where
        K: CacheKey + Debug + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_set");
        metrics::counter!("cache.set.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set.error").increment(1);
                return Err(e.into());
            }
        };

        let serialized_value = match self.serializer.serialize(&value).await {
            Ok(v) => v,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set.error").increment(1);
                return Err(e.into());
            }
        };

        // Correct Result type
        let result: Result<(), RedisCacheError> = self
            .pool
            .execute(move |conn| {
                // Clone necessary data
                let key_str_clone = key_str.clone();
                let serialized_value_clone = serialized_value.clone();
                let ttl_clone = ttl;
                Box::pin(async move {
                    if let Some(duration) = ttl_clone {
                        if let Ok(secs) = duration.as_secs().try_into() as Result<i64, _> {
                            if secs > 0 {
                                // Return RedisResult directly
                                return redis::cmd("SET")
                                    .arg(&key_str_clone)
                                    .arg(&serialized_value_clone)
                                    .arg("EX")
                                    .arg(secs)
                                    .query_async::<()>(conn)
                                    .await;
                            } else {
                                warn!("Attempted to set key '{}' with zero or negative TTL {:?}. Setting without TTL.", key_str_clone, duration);
                            }
                        } else {
                            error!("TTL duration {:?} exceeds Redis expiration limit for key '{}'. Setting without TTL.", duration, key_str_clone);
                        }
                    }
                    // Set without TTL, return RedisResult directly
                    redis::cmd("SET")
                        .arg(&key_str_clone)
                        .arg(&serialized_value_clone)
                        .query_async::<()>(conn)
                        .await
                })
            })
            .await;

        // Simplified match
        match result {
            Ok(_) => {
                timer.record_success();
                metrics::counter!("cache.set.success").increment(1);
                Ok(())
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self, items), level = "trace")]
    pub(crate) async fn _set_many_internal<K, V>(
        &self,
        items: HashMap<K, V>,
        ttl: Option<Duration>,
    ) -> CacheResult<()>
    where
        K: CacheKey + Debug + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_set_many");
        metrics::counter!("cache.set_many.total").increment(items.len() as u64);

        if items.is_empty() {
            timer.record_success();
            return Ok(());
        }

        // Pre-allocate for efficiency
        let mut redis_cmd_args: Vec<(String, Vec<u8>)> = Vec::with_capacity(items.len());
        let mut key_count = 0;

        for (key, value) in items {
            let key_str = match self.key_to_string(key) {
                Ok(k) => k,
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_many.error").increment(1);
                    return Err(e.into()); // Fail fast on key error
                }
            };
            let serialized_value = match self.serializer.serialize(&value).await {
                Ok(v) => v,
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_many.error").increment(1);
                    return Err(e.into()); // Fail fast on serialization error
                }
            };
            redis_cmd_args.push((key_str, serialized_value));
            key_count += 1;
        }

        // Correct Result type
        let result: Result<(), RedisCacheError> = self
            .pool
            .execute(move |conn| {
                // Clone needed data
                let redis_cmd_args_clone = redis_cmd_args.clone();
                let ttl_clone = ttl;
                Box::pin(async move {
                    // MSET uses a sequence of key value key value...
                    let mset_args: Vec<_> = redis_cmd_args_clone
                        .iter()
                        .flat_map(|(k, v)| [k.as_str().to_redis_args(), v.as_slice().to_redis_args()])
                        .collect();

                    // Execute MSET
                    let mset_result = redis::cmd("MSET").arg(&mset_args).query_async::<()>(conn).await;

                    // If MSET succeeded and TTL is set, attempt to set TTLs
                    if mset_result.is_ok() {
                        if let Some(duration) = ttl_clone {
                            if let Ok(secs) = duration.as_secs().try_into() as Result<i64, _> {
                                if secs > 0 {
                                    // Create a pipeline for EXPIRE commands
                                    let mut pipe = redis::pipe();
                                    for (key, _) in &redis_cmd_args_clone {
                                        pipe.cmd("EXPIRE").arg(key).arg(secs).ignore();
                                    }
                                    // Execute pipeline and return its result
                                    return pipe.query_async::<()>(conn).await;
                                } else {
                                    warn!("Attempted to MSET keys with zero or negative TTL {:?}. Keys set without TTL.", duration);
                                }
                            } else {
                                error!("TTL duration {:?} exceeds Redis expiration limit for MSET. Keys set without TTL.", duration);
                            }
                        }
                    }
                    // Return the MSET result if no TTL was applied or if MSET failed
                    mset_result
                })
            })
            .await;

        // Simplified match
        match result {
            Ok(_) => {
                timer.record_success();
                metrics::counter!("cache.set_many.success").increment(key_count);
                Ok(())
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_many.error").increment(key_count);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn _get_internal<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_get");
        metrics::counter!("cache.get.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.get.error").increment(1);
                return Err(e.into());
            }
        };

        // Store key_str for error logging
        let key_str_for_error = key_str.clone();

        // Correct Result type
        let result: Result<Option<Vec<u8>>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                // Clone necessary data
                let key_str_clone = key_str.clone();
                Box::pin(async move {
                    redis::cmd("GET")
                        .arg(&key_str_clone)
                        .query_async::<Option<Vec<u8>>>(conn)
                        .await
                })
            })
            .await;

        // Simplified match - logic inside Ok branches remains similar
        match result {
            Ok(Some(bytes)) => {
                match self.serializer.deserialize::<V>(&bytes).await {
                    Ok(value) => {
                        timer.record_success();
                        metrics::counter!("cache.get.success").increment(1);
                        metrics::counter!("cache.get.hit").increment(1);
                        Ok(Some(value))
                    }
                    Err(e) => {
                        timer.record_error(&e);
                        metrics::counter!("cache.get.error").increment(1);
                        metrics::counter!("cache.get.miss_deserialization_error").increment(1);
                        warn!(
                            "Deserialization failed for key '{}': {}",
                            key_str_for_error, e
                        );
                        Ok(None) // Treat deserialization error as miss
                    }
                }
            }
            Ok(None) => {
                timer.record_success();
                metrics::counter!("cache.get.success").increment(1);
                metrics::counter!("cache.get.miss").increment(1);
                Ok(None)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.get.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self, keys), level = "trace")]
    pub(crate) async fn _get_many_internal<K, V>(
        &self,
        keys: Vec<K>,
    ) -> CacheResult<HashMap<String, Option<V>>>
    where
        K: CacheKey + Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_get_many");
        metrics::counter!("cache.get_many.total").increment(keys.len() as u64);

        if keys.is_empty() {
            timer.record_success();
            return Ok(HashMap::new());
        }

        let mut key_strings = Vec::with_capacity(keys.len());
        let mut key_map = HashMap::with_capacity(keys.len()); // Original K.to_string() -> Prefixed String
        let mut original_key_reprs = Vec::with_capacity(keys.len()); // Store original K.to_string()

        for key in keys {
            let original_key_repr = key.to_string(); // Capture original representation
            match self.key_to_string(key) {
                Ok(key_str) => {
                    key_strings.push(key_str.clone());
                    key_map.insert(key_str, original_key_repr.clone()); // Map prefixed -> original
                    original_key_reprs.push(original_key_repr);
                }
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.get_many.error").increment(1);
                    return Err(e.into()); // Fail fast
                }
            }
        }

        let key_strings_len = key_strings.len();

        // Clone key_strings for Redis command
        let key_strings_for_redis = key_strings.clone();

        // Correct Result type
        let result: Result<Vec<Option<Vec<u8>>>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                // Clone necessary data
                let key_strings_clone = key_strings_for_redis.clone();
                Box::pin(async move {
                    redis::cmd("MGET")
                        .arg(&key_strings_clone)
                        .query_async::<Vec<Option<Vec<u8>>>>(conn)
                        .await
                })
            })
            .await;

        // Simplified match - logic inside Ok branch remains similar
        match result {
            Ok(values) => {
                timer.record_success();
                metrics::counter!("cache.get_many.success").increment(key_strings_len as u64);
                let mut results_map = HashMap::with_capacity(key_strings_len);
                let mut hits = 0;
                let mut misses = 0;
                let mut errors = 0;

                // Now key_strings is available for this iteration
                for (prefixed_key_str, value_opt) in key_strings.into_iter().zip(values.into_iter())
                {
                    // Use the map to find the original key representation
                    let original_key_repr = key_map
                        .get(&prefixed_key_str)
                        .cloned()
                        .unwrap_or_else(|| prefixed_key_str.clone()); // Fallback just in case

                    match value_opt {
                        Some(bytes) => match self.serializer.deserialize::<V>(&bytes).await {
                            Ok(value) => {
                                results_map.insert(original_key_repr, Some(value));
                                hits += 1;
                            }
                            Err(e) => {
                                warn!(
                                    "Deserialization failed for key '{}' (original: '{}'): {}",
                                    prefixed_key_str, original_key_repr, e
                                );
                                results_map.insert(original_key_repr, None);
                                errors += 1;
                            }
                        },
                        None => {
                            results_map.insert(original_key_repr, None);
                            misses += 1;
                        }
                    }
                }
                metrics::counter!("cache.get_many.hit").increment(hits);
                metrics::counter!("cache.get_many.miss").increment(misses);
                metrics::counter!("cache.get_many.miss_deserialization_error").increment(errors);
                Ok(results_map)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                // Use original key count for error metric
                metrics::counter!("cache.get_many.error")
                    .increment(original_key_reprs.len() as u64);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn delete_internal<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + Debug + 'static,
    {
        let timer = OperationTimer::new("cache_delete");
        metrics::counter!("cache.delete.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.delete.error").increment(1);
                return Err(e.into()); // Return Err
            }
        };

        // Correct Result type (DEL returns integer count)
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                // Clone necessary data
                let key_str_clone = key_str.clone();
                Box::pin(async move {
                    redis::cmd("DEL")
                        .arg(&key_str_clone)
                        .query_async::<usize>(conn)
                        .await
                })
            })
            .await;

        // Simplified match
        match result {
            Ok(count) => {
                let deleted = count > 0;
                timer.record_success();
                metrics::counter!("cache.delete.success").increment(1);
                if deleted {
                    metrics::counter!("cache.delete.deleted").increment(1);
                }
                Ok(deleted)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.delete.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self, keys), level = "trace")]
    pub(crate) async fn delete_many_internal<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + Debug + 'static,
    {
        let timer = OperationTimer::new("cache_delete_many");
        metrics::counter!("cache.delete_many.total").increment(keys.len() as u64);

        if keys.is_empty() {
            timer.record_success();
            return Ok(0);
        }

        let mut key_strings = Vec::with_capacity(keys.len());
        let key_count = keys.len() as u64;

        for key in keys {
            match self.key_to_string(key) {
                Ok(k) => key_strings.push(k),
                Err(e) => {
                    timer.record_error(&e);
                    // Increment error count based on total potential keys
                    metrics::counter!("cache.delete_many.error").increment(key_count);
                    return Err(e.into()); // Fail fast
                }
            }
        }

        let key_strings_len = key_strings.len();

        // Correct Result type
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                // Clone necessary data
                let key_strings_clone = key_strings.clone();
                Box::pin(async move {
                    redis::cmd("DEL")
                        .arg(&key_strings_clone)
                        .query_async::<usize>(conn)
                        .await
                })
            })
            .await;

        // Simplified match
        match result {
            Ok(count) => {
                timer.record_success();
                // Success metric based on total keys attempted
                metrics::counter!("cache.delete_many.success").increment(key_strings_len as u64);
                metrics::counter!("cache.delete_many.deleted").increment(count as u64);
                Ok(count)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                // Error metric based on total keys attempted
                metrics::counter!("cache.delete_many.error").increment(key_strings_len as u64);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn exists_internal<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + Debug + 'static,
    {
        let timer = OperationTimer::new("cache_exists");
        metrics::counter!("cache.exists.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.exists.error").increment(1);
                return Err(e.into()); // Return Err
            }
        };

        // Correct Result type
        let result: Result<bool, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                // Clone necessary data
                let key_str_clone = key_str.clone();
                Box::pin(async move {
                    redis::cmd("EXISTS")
                        .arg(&key_str_clone)
                        .query_async::<bool>(conn)
                        .await
                })
            })
            .await;

        // Simplified match
        match result {
            Ok(exists) => {
                timer.record_success();
                metrics::counter!("cache.exists.success").increment(1);
                if exists {
                    metrics::counter!("cache.exists.true").increment(1);
                } else {
                    metrics::counter!("cache.exists.false").increment(1);
                }
                Ok(exists)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.exists.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn expire_internal<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + Debug + 'static,
    {
        let timer = OperationTimer::new("cache_expire");
        metrics::counter!("cache.expire.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.expire.error").increment(1);
                return Err(e.into()); // Return Err
            }
        };

        let seconds = match ttl.as_secs().try_into() as Result<i64, _> {
            Ok(secs) if secs > 0 => secs,
            _ => {
                let err =
                    RedisCacheError::InvalidKey("TTL must be a positive duration".to_string());
                timer.record_error(&err);
                metrics::counter!("cache.expire.error").increment(1);
                return Err(err.into());
            }
        };

        // Correct Result type
        let result: Result<bool, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                // Clone necessary data
                let key_str_clone = key_str.clone();
                Box::pin(async move {
                    redis::cmd("EXPIRE")
                        .arg(&key_str_clone)
                        .arg(seconds)
                        .query_async::<bool>(conn)
                        .await
                })
            })
            .await;

        // Simplified match
        match result {
            Ok(set) => {
                timer.record_success();
                metrics::counter!("cache.expire.success").increment(1);
                if set {
                    metrics::counter!("cache.expire.set").increment(1);
                }
                Ok(set)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.expire.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub async fn clear_internal(&self) -> CacheResult<()> {
        let timer = OperationTimer::new("cache_clear");
        metrics::counter!("cache.clear.total");
        metrics::counter!("cache.clear.error");
        metrics::counter!("cache.clear.success");

        // Clear all keys in the Redis cache
        let result = self.clear_with_prefix().await;

        match result {
            Ok(_) => {
                timer.record_success();
                metrics::counter!("cache.clear.success");
                Ok(())
            }
            Err(e) => {
                let cache_err = RedisCacheError::InvalidKey(e.to_string());
                timer.record_error(&cache_err);
                metrics::counter!("cache.clear.error");
                Err(CacheError::BackendError(e.to_string()))
            }
        }
    }
}
