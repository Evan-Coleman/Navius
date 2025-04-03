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
use redis::{AsyncCommands, FromRedisValue, RedisError, RedisResult, ToRedisArgs, Value};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument, trace, warn};

// Hash operations will be moved here

impl RedisCache {
    #[instrument(skip(self, field, value), level = "trace")]
    pub(crate) async fn hash_set_internal<K, F, V>(
        &self,
        key: K,
        field: F,
        value: V,
    ) -> CacheResult<bool>
    // Returns true if field is new, false if updated
    where
        K: CacheKey + 'static,
        F: ToRedisArgs + Send + Sync + 'static + Debug,
        V: Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_hash_set");
        metrics::counter!("cache.hash_set.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_set.error").increment(1);
                return Err(e.into());
            }
        };

        let serialized_value = match self.serializer.serialize(&value).await {
            Ok(v) => v,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_set.error").increment(1);
                return Err(e.into());
            }
        };

        // HSET returns integer: 1 if field is new, 0 if updated
        let result: Result<isize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                let field_clone = field; // Requires F: Clone or pass ownership
                let serialized_value_clone = serialized_value.clone();
                async move {
                    // Removed Box::pin
                    redis::cmd("HSET")
                        .arg(&key_str_clone)
                        .arg(field_clone)
                        .arg(&serialized_value_clone)
                        .query_async::<isize>(conn) // HSET returns integer
                        .await
                }
            })
            .await;

        match result {
            Ok(val) => {
                timer.record_success();
                metrics::counter!("cache.hash_set.success").increment(1);
                let is_new_field = val == 1;
                if is_new_field {
                    metrics::counter!("cache.hash_set.created").increment(1);
                } else {
                    metrics::counter!("cache.hash_set.updated").increment(1);
                }
                Ok(is_new_field)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.hash_set.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self, items), level = "trace")]
    pub(crate) async fn hash_set_many_internal<K, F, V>(
        &self,
        key: K,
        items: HashMap<F, V>,
    ) -> CacheResult<usize>
    // Returns number of fields added
    where
        K: CacheKey + 'static,
        F: ToRedisArgs + Send + Sync + 'static + Debug,
        V: Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_hash_set_many");
        metrics::counter!("cache.hash_set_many.total").increment(items.len() as u64);

        if items.is_empty() {
            timer.record_success();
            return Ok(0);
        }

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_set_many.error").increment(1);
                return Err(e.into());
            }
        };

        // Serialize values and prepare args for HMSET/HSET
        let mut args: Vec<Box<dyn ToRedisArgs + Send + Sync>> = Vec::with_capacity(items.len() * 2);
        let mut item_count = 0;
        for (field, value) in items {
            let serialized_value = match self.serializer.serialize(&value).await {
                Ok(v) => v,
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.hash_set_many.error").increment(1);
                    // Fail fast on serialization error
                    return Err(e.into());
                }
            };
            args.push(Box::new(field));
            args.push(Box::new(serialized_value));
            item_count += 1;
        }

        // HSET (Redis >= 4.0) returns number of fields added (not updated)
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                // Clone the Vec<Box<dyn ToRedisArgs + Send + Sync>>
                let args_clone = args.into_iter().map(|arg| arg).collect::<Vec<_>>();
                async move {
                    // Removed Box::pin
                    redis::cmd("HSET")
                        .arg(&key_str_clone)
                        .arg(&args_clone) // Pass Vec<Box<dyn ToRedisArgs>>
                        .query_async::<usize>(conn) // HSET returns number of fields added
                        .await
                }
            })
            .await;

        match result {
            Ok(fields_added) => {
                timer.record_success();
                metrics::counter!("cache.hash_set_many.success").increment(item_count);
                metrics::counter!("cache.hash_set_many.added").increment(fields_added as u64);
                Ok(fields_added)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.hash_set_many.error").increment(item_count);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self, field), level = "trace")]
    pub(crate) async fn hash_get_internal<K, F, V>(
        &self,
        key: K,
        field: F,
    ) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        F: ToRedisArgs + Send + Sync + 'static + Debug,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_hash_get");
        metrics::counter!("cache.hash_get.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_get.error").increment(1);
                return Err(e.into());
            }
        };

        let result: Result<Option<Vec<u8>>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                let field_clone = field; // Requires F: Clone or pass ownership
                async move {
                    // Removed Box::pin
                    redis::cmd("HGET")
                        .arg(&key_str_clone)
                        .arg(field_clone)
                        .query_async::<Option<Vec<u8>>>(conn)
                        .await
                }
            })
            .await;

        match result {
            Ok(Some(bytes)) => {
                match self.serializer.deserialize::<V>(&bytes).await {
                    Ok(value) => {
                        timer.record_success();
                        metrics::counter!("cache.hash_get.success").increment(1);
                        metrics::counter!("cache.hash_get.hit").increment(1);
                        Ok(Some(value))
                    }
                    Err(e) => {
                        timer.record_error(&e);
                        metrics::counter!("cache.hash_get.error").increment(1);
                        metrics::counter!("cache.hash_get.miss_deserialization_error").increment(1);
                        warn!(
                            "Deserialization failed for hash key '{}', field '{:?}': {}",
                            key_str, field, e
                        );
                        Ok(None) // Treat deserialization error as miss
                    }
                }
            }
            Ok(None) => {
                timer.record_success();
                metrics::counter!("cache.hash_get.success").increment(1);
                metrics::counter!("cache.hash_get.miss").increment(1);
                Ok(None)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.hash_get.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self, fields), level = "trace")]
    pub(crate) async fn hash_get_many_internal<K, F, V>(
        &self,
        key: K,
        fields: Vec<F>,
    ) -> CacheResult<HashMap<F, Option<V>>>
    where
        K: CacheKey + 'static,
        F: ToRedisArgs
            + FromRedisValue
            + Send
            + Sync
            + Eq
            + std::hash::Hash
            + Clone
            + Debug
            + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_hash_get_many");
        metrics::counter!("cache.hash_get_many.total").increment(fields.len() as u64);

        if fields.is_empty() {
            timer.record_success();
            return Ok(HashMap::new());
        }

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_get_many.error").increment(1);
                return Err(e.into());
            }
        };

        // HMGET returns Vec<Option<Vec<u8>>>, matching the order of requested fields
        let result: Result<Vec<Option<Vec<u8>>>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                let fields_clone = fields.clone(); // Clone the vec of fields
                async move {
                    // Removed Box::pin
                    redis::cmd("HMGET")
                        .arg(&key_str_clone)
                        .arg(&fields_clone) // Pass the Vec<F>
                        .query_async::<Vec<Option<Vec<u8>>>>(conn)
                        .await
                }
            })
            .await;

        match result {
            Ok(values) => {
                timer.record_success();
                metrics::counter!("cache.hash_get_many.success").increment(fields.len() as u64);
                let mut results_map = HashMap::with_capacity(fields.len());
                let mut hits = 0;
                let mut misses = 0;
                let mut errors = 0;

                // fields vector is moved into the closure, so we need it again or a clone
                // Let's re-use the original fields Vec by taking ownership back
                // (assuming it wasn't needed elsewhere, which is typical here)
                // We need to ensure the order matches: zip fields with values
                let original_fields = fields; // Take ownership back

                for (field, value_opt) in original_fields.into_iter().zip(values.into_iter()) {
                    match value_opt {
                        Some(bytes) => {
                            match self.serializer.deserialize::<V>(&bytes).await {
                                Ok(value) => {
                                    results_map.insert(field, Some(value));
                                    hits += 1;
                                }
                                Err(e) => {
                                    warn!(
                                        "Deserialization failed for hash key '{}', field '{:?}': {}",
                                        key_str,
                                        field, // Use the actual field from the loop
                                        e
                                    );
                                    results_map.insert(field, None);
                                    errors += 1;
                                }
                            }
                        }
                        None => {
                            results_map.insert(field, None);
                            misses += 1;
                        }
                    }
                }
                metrics::counter!("cache.hash_get_many.hit").increment(hits);
                metrics::counter!("cache.hash_get_many.miss").increment(misses);
                metrics::counter!("cache.hash_get_many.miss_deserialization_error")
                    .increment(errors);
                Ok(results_map)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.hash_get_many.error").increment(fields.len() as u64);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn hash_get_all_internal<K, F, V>(&self, key: K) -> CacheResult<HashMap<F, V>>
    where
        K: CacheKey + 'static,
        F: FromRedisValue + Send + Sync + Eq + std::hash::Hash + Debug + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_hash_get_all");
        metrics::counter!("cache.hash_get_all.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_get_all.error").increment(1);
                return Err(e.into());
            }
        };

        // HGETALL returns Vec<u8>, needs careful parsing (interleaved keys/values)
        let result: Result<HashMap<F, Vec<u8>>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                async move {
                    // Removed Box::pin
                    redis::cmd("HGETALL")
                        .arg(&key_str_clone)
                        .query_async::<HashMap<F, Vec<u8>>>(conn)
                        .await
                }
            })
            .await;

        match result {
            Ok(raw_map) => {
                timer.record_success();
                metrics::counter!("cache.hash_get_all.success").increment(1);
                let gauge = metrics::gauge!("cache.hash_get_all.field_count");
                gauge.set(raw_map.len() as f64);

                let mut deserialized_map = HashMap::with_capacity(raw_map.len());
                let mut errors = 0;

                for (field, bytes) in raw_map {
                    match self.serializer.deserialize::<V>(&bytes).await {
                        Ok(value) => {
                            deserialized_map.insert(field, value);
                        }
                        Err(e) => {
                            warn!(
                                "Deserialization failed for hash key '{}', field '{:?}': {}",
                                key_str, field, e
                            );
                            errors += 1;
                            // Skip this field or error out? Skipping for now.
                        }
                    }
                }
                metrics::counter!("cache.hash_get_all.deserialization_error").increment(errors);
                Ok(deserialized_map)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.hash_get_all.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self, field), level = "trace")]
    pub(crate) async fn hash_delete_internal<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: ToRedisArgs + Send + Sync + 'static + Debug,
    {
        let timer = OperationTimer::new("cache_hash_delete");
        metrics::counter!("cache.hash_delete.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_delete.error").increment(1);
                return Err(e.into());
            }
        };

        // HDEL returns integer: number of fields deleted (0 or 1 here)
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                let field_clone = field; // Requires F: Clone or pass ownership
                async move {
                    // Removed Box::pin
                    redis::cmd("HDEL")
                        .arg(&key_str_clone)
                        .arg(field_clone)
                        .query_async::<usize>(conn)
                        .await
                }
            })
            .await;

        match result {
            Ok(count) => {
                let deleted = count > 0;
                timer.record_success();
                metrics::counter!("cache.hash_delete.success").increment(1);
                if deleted {
                    metrics::counter!("cache.hash_delete.deleted").increment(1);
                }
                Ok(deleted)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.hash_delete.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self, fields), level = "trace")]
    pub(crate) async fn hash_delete_many_internal<K, F>(
        &self,
        key: K,
        fields: Vec<F>,
    ) -> CacheResult<usize>
    // Returns number of fields deleted
    where
        K: CacheKey + 'static,
        F: ToRedisArgs + Send + Sync + 'static + Debug,
    {
        let timer = OperationTimer::new("cache_hash_delete_many");
        metrics::counter!("cache.hash_delete_many.total").increment(fields.len() as u64);

        if fields.is_empty() {
            timer.record_success();
            return Ok(0);
        }

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_delete_many.error").increment(1);
                return Err(e.into());
            }
        };

        let fields_count = fields.len() as u64;

        // HDEL returns integer: number of fields deleted
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                let fields_clone = fields; // Pass ownership of Vec<F>
                async move {
                    // Removed Box::pin
                    redis::cmd("HDEL")
                        .arg(&key_str_clone)
                        .arg(&fields_clone) // Pass Vec<F>
                        .query_async::<usize>(conn)
                        .await
                }
            })
            .await;

        match result {
            Ok(deleted_count) => {
                timer.record_success();
                metrics::counter!("cache.hash_delete_many.success").increment(fields_count);
                metrics::counter!("cache.hash_delete_many.deleted").increment(deleted_count as u64);
                Ok(deleted_count)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.hash_delete_many.error").increment(fields_count);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self, field), level = "trace")]
    pub(crate) async fn hash_exists_internal<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: ToRedisArgs + Send + Sync + 'static + Debug,
    {
        let timer = OperationTimer::new("cache_hash_exists");
        metrics::counter!("cache.hash_exists.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_exists.error").increment(1);
                return Err(e.into());
            }
        };

        // HEXISTS returns boolean
        let result: Result<bool, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                let field_clone = field; // Requires F: Clone or pass ownership
                async move {
                    // Removed Box::pin
                    redis::cmd("HEXISTS")
                        .arg(&key_str_clone)
                        .arg(field_clone)
                        .query_async::<bool>(conn)
                        .await
                }
            })
            .await;

        match result {
            Ok(exists) => {
                timer.record_success();
                metrics::counter!("cache.hash_exists.success").increment(1);
                if exists {
                    metrics::counter!("cache.hash_exists.true").increment(1);
                } else {
                    metrics::counter!("cache.hash_exists.false").increment(1);
                }
                Ok(exists)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.hash_exists.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn hash_length_internal<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let timer = OperationTimer::new("cache_hash_length");
        metrics::counter!("cache.hash_length.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_length.error").increment(1);
                return Err(e.into());
            }
        };

        // HLEN returns integer
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                async move {
                    // Removed Box::pin
                    redis::cmd("HLEN")
                        .arg(&key_str_clone)
                        .query_async::<usize>(conn)
                        .await
                }
            })
            .await;

        match result {
            Ok(length) => {
                timer.record_success();
                metrics::counter!("cache.hash_length.success").increment(1);
                let gauge = metrics::gauge!("cache.hash_length.value");
                gauge.set(length as f64);
                Ok(length)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.hash_length.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn hash_keys_internal<K, F>(&self, key: K) -> CacheResult<Vec<F>>
    where
        K: CacheKey + 'static,
        F: FromRedisValue + Send + Sync + 'static + Debug,
    {
        let timer = OperationTimer::new("cache_hash_keys");
        metrics::counter!("cache.hash_keys.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_keys.error").increment(1);
                return Err(e.into());
            }
        };

        // HKEYS returns Vec<F>
        let result: Result<Vec<F>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                async move {
                    // Removed Box::pin
                    redis::cmd("HKEYS")
                        .arg(&key_str_clone)
                        .query_async::<Vec<F>>(conn)
                        .await
                }
            })
            .await;

        match result {
            Ok(fields) => {
                timer.record_success();
                metrics::counter!("cache.hash_keys.success").increment(1);
                let gauge = metrics::gauge!("cache.hash_keys.count");
                gauge.set(fields.len() as f64);
                Ok(fields)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.hash_keys.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn hash_values_internal<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_hash_values");
        metrics::counter!("cache.hash_values.total").increment(1);

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.hash_values.error").increment(1);
                return Err(e.into());
            }
        };

        // HVALS returns Vec<Vec<u8>>
        let result: Result<Vec<Vec<u8>>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                async move {
                    // Removed Box::pin
                    redis::cmd("HVALS")
                        .arg(&key_str_clone)
                        .query_async::<Vec<Vec<u8>>>(conn)
                        .await
                }
            })
            .await;

        match result {
            Ok(byte_values) => {
                timer.record_success();
                metrics::counter!("cache.hash_values.success").increment(1);
                let gauge = metrics::gauge!("cache.hash_values.count");
                gauge.set(byte_values.len() as f64);

                let mut deserialized_values = Vec::with_capacity(byte_values.len());
                let mut errors = 0;

                for bytes in byte_values {
                    match self.serializer.deserialize::<V>(&bytes).await {
                        Ok(value) => deserialized_values.push(value),
                        Err(e) => {
                            warn!(
                                "Deserialization failed for value in hash key '{}': {}",
                                key_str, e
                            );
                            errors += 1;
                            // Skip or error? Skipping for now.
                        }
                    }
                }
                metrics::counter!("cache.hash_values.deserialization_error").increment(errors);
                Ok(deserialized_values)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.hash_values.error").increment(1);
                Err(cache_err.into())
            }
        }
    }
}
