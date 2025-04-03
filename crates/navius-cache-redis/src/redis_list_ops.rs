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
use redis::{AsyncCommands, RedisError, RedisResult, ToRedisArgs};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument, trace, warn};

// List operations will be moved here

impl RedisCache {
    pub async fn list_push_left_internal<K, V>(&self, key: K, value: V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_list_push_left");
        metrics::counter!("cache.list_push_left.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_push_left.error").increment(1);
                // Return Err instead of Ok(0)
                return Err(e.into());
            }
        };

        // Serialize the value
        let value_str = match self.serializer.serialize(&value).await {
            Ok(v) => v,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_push_left.error").increment(1);
                // Return Err instead of Ok(0)
                return Err(e.into());
            }
        };

        // Push the value to the left of the list in Redis
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                let value_str_clone = value_str.clone();
                Box::pin(async move {
                    redis::cmd("LPUSH")
                        .arg(&key_str_clone)
                        .arg(&value_str_clone)
                        .query_async::<usize>(conn)
                        .await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(len) => {
                timer.record_success();
                metrics::counter!("cache.list_push_left.success").increment(1);
                Ok(len)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.list_push_left.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub async fn list_push_right_internal<K, V>(&self, key: K, value: V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_list_push_right");
        metrics::counter!("cache.list_push_right.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_push_right.error").increment(1);
                // Return Err instead of Ok(0)
                return Err(e.into());
            }
        };

        // Serialize the value
        let value_str = match self.serializer.serialize(&value).await {
            Ok(v) => v,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_push_right.error").increment(1);
                // Return Err instead of Ok(0)
                return Err(e.into());
            }
        };

        // Push the value to the right of the list in Redis
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                let value_str_clone = value_str.clone();
                Box::pin(async move {
                    redis::cmd("RPUSH")
                        .arg(&key_str_clone)
                        .arg(&value_str_clone)
                        .query_async::<usize>(conn)
                        .await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(len) => {
                timer.record_success();
                metrics::counter!("cache.list_push_right.success").increment(1);
                Ok(len)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.list_push_right.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub async fn list_pop_left_internal<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        let timer = OperationTimer::new("cache_list_pop_left");
        metrics::counter!("cache.list_pop_left.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_pop_left.error").increment(1);
                // Return Err instead of Ok(None)
                return Err(e.into());
            }
        };

        // Pop the value from the left of the list in Redis
        let result: Result<Option<String>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                Box::pin(async move {
                    redis::cmd("LPOP")
                        .arg(&key_str_clone)
                        .query_async::<Option<String>>(conn)
                        .await
                })
            })
            .await;

        // Handle result and deserialize
        match result {
            Ok(Some(val)) => match self.serializer.deserialize::<V>(val.as_bytes()).await {
                Ok(value) => {
                    timer.record_success();
                    metrics::counter!("cache.list_pop_left.success").increment(1);
                    metrics::counter!("cache.list_pop_left.hit").increment(1);
                    Ok(Some(value))
                }
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.list_pop_left.error").increment(1);
                    Err(e.into())
                }
            },
            Ok(None) => {
                timer.record_success();
                metrics::counter!("cache.list_pop_left.miss").increment(1);
                Ok(None)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.list_pop_left.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub async fn list_pop_right_internal<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        let timer = OperationTimer::new("cache_list_pop_right");
        metrics::counter!("cache.list_pop_right.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_pop_right.error").increment(1);
                // Return Err instead of Ok(None)
                return Err(e.into());
            }
        };

        // Pop the value from the right of the list in Redis
        let result: Result<Option<String>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                Box::pin(async move {
                    redis::cmd("RPOP")
                        .arg(&key_str_clone)
                        .query_async::<Option<String>>(conn)
                        .await
                })
            })
            .await;

        // Handle result and deserialize
        match result {
            Ok(Some(val)) => match self.serializer.deserialize::<V>(val.as_bytes()).await {
                Ok(value) => {
                    timer.record_success();
                    metrics::counter!("cache.list_pop_right.success").increment(1);
                    metrics::counter!("cache.list_pop_right.hit").increment(1);
                    Ok(Some(value))
                }
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.list_pop_right.error").increment(1);
                    Err(e.into())
                }
            },
            Ok(None) => {
                timer.record_success();
                metrics::counter!("cache.list_pop_right.miss").increment(1);
                Ok(None)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.list_pop_right.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub async fn list_length_internal<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let timer = OperationTimer::new("cache_list_length");
        metrics::counter!("cache.list_length.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_length.error").increment(1);
                // Return Err instead of Ok(0)
                return Err(e.into());
            }
        };

        // Get the length of the list in Redis
        let result: Result<isize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                Box::pin(async move {
                    redis::cmd("LLEN")
                        .arg(&key_str_clone)
                        .query_async::<isize>(conn)
                        .await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(val) => {
                timer.record_success();
                metrics::counter!("cache.list_length.success").increment(1);
                Ok(val as usize)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.list_length.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub async fn list_push_right_many_internal<K, V>(
        &self,
        key: K,
        values: Vec<V>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_list_push_right_many");
        metrics::counter!("cache.list_push_right_many.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_push_right_many.error").increment(1);
                return Err(e.into());
            }
        };

        // Serialize all values
        let mut serialized_values = Vec::with_capacity(values.len());
        for value in values {
            // Use proper await syntax
            match self.serializer.serialize(&value).await {
                Ok(v) => serialized_values.push(v),
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.list_push_right_many.error").increment(1);
                    return Err(e.into());
                }
            }
        }

        // Push values to the right of the list in Redis
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                let serialized_values_clone = serialized_values.clone();
                Box::pin(async move {
                    let mut cmd = redis::cmd("RPUSH");
                    cmd.arg(&key_str_clone);
                    cmd.arg(&serialized_values_clone);
                    cmd.query_async::<usize>(conn).await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(len) => {
                timer.record_success();
                metrics::counter!("cache.list_push_right_many.success").increment(1);
                Ok(len)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.list_push_right_many.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub async fn list_push_left_many_internal<K, V>(
        &self,
        key: K,
        values: Vec<V>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_list_push_left_many");
        metrics::counter!("cache.list_push_left_many.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_push_left_many.error").increment(1);
                return Err(e.into());
            }
        };

        // Serialize all values
        let mut serialized_values = Vec::with_capacity(values.len());
        for value in values {
            // Use proper await syntax
            match self.serializer.serialize(&value).await {
                Ok(v) => serialized_values.push(v),
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.list_push_left_many.error").increment(1);
                    return Err(e.into());
                }
            }
        }

        // Push values to the left of the list in Redis
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                let serialized_values_clone = serialized_values.clone();
                Box::pin(async move {
                    let mut cmd = redis::cmd("LPUSH");
                    cmd.arg(&key_str_clone);
                    cmd.arg(&serialized_values_clone);
                    cmd.query_async::<usize>(conn).await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(len) => {
                timer.record_success();
                metrics::counter!("cache.list_push_left_many.success").increment(1);
                Ok(len)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.list_push_left_many.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub async fn list_range_internal<K, V>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_list_range");
        metrics::counter!("cache.list_range.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_range.error").increment(1);
                return Err(e.into());
            }
        };

        // Get the range of values from the list in Redis
        let result: Result<Vec<String>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                Box::pin(async move {
                    let mut cmd = redis::cmd("LRANGE");
                    cmd.arg(&key_str_clone);
                    cmd.arg(start);
                    cmd.arg(stop);
                    cmd.query_async::<Vec<String>>(conn).await
                })
            })
            .await;

        // Handle result and deserialize
        match result {
            Ok(values) => {
                if values.is_empty() {
                    timer.record_success();
                    metrics::counter!("cache.list_range.miss").increment(1);
                    return Ok(Vec::new());
                }

                let mut result_vec = Vec::with_capacity(values.len());
                for value_str in values {
                    match self.serializer.deserialize::<V>(value_str.as_bytes()).await {
                        Ok(deserialized) => result_vec.push(deserialized),
                        Err(e) => {
                            timer.record_error(&e);
                            metrics::counter!("cache.list_range.error").increment(1);
                            return Err(e.into());
                        }
                    }
                }

                timer.record_success();
                metrics::counter!("cache.list_range.success").increment(1);
                metrics::counter!("cache.list_range.hit").increment(1);
                Ok(result_vec)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.list_range.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub async fn list_trim_internal<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
    where
        K: CacheKey + 'static,
    {
        let timer = OperationTimer::new("cache_list_trim");
        metrics::counter!("cache.list_trim.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_trim.error").increment(1);
                return Err(e.into());
            }
        };

        // Trim the list in Redis
        let result: Result<String, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                Box::pin(async move {
                    redis::cmd("LTRIM")
                        .arg(&key_str_clone)
                        .arg(start)
                        .arg(stop)
                        .query_async::<String>(conn)
                        .await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(_) => {
                timer.record_success();
                metrics::counter!("cache.list_trim.success").increment(1);
                Ok(())
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.list_trim.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub async fn list_set_internal<K, V>(&self, key: K, index: isize, value: V) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_list_set");
        metrics::counter!("cache.list_set.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_set.error").increment(1);
                return Err(e.into());
            }
        };

        // Serialize the value
        let value_str = match self.serializer.serialize(&value).await {
            Ok(v) => v,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_set.error").increment(1);
                // Return Err instead of Ok(())
                return Err(e.into());
            }
        };

        // Set the value at the specified index in the list in Redis
        let result: Result<String, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                let key_str_clone = key_str.clone();
                let value_str_clone = value_str.clone();
                Box::pin(async move {
                    redis::cmd("LSET")
                        .arg(&key_str_clone)
                        .arg(index)
                        .arg(&value_str_clone)
                        .query_async::<String>(conn)
                        .await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(_) => {
                timer.record_success();
                metrics::counter!("cache.list_set.success").increment(1);
                Ok(())
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.list_set.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    // ADDED: Internal implementation for removing elements from a list.
    #[instrument(skip(self, key, value), fields(key = %key.to_string()), level = "debug")]
    pub async fn list_remove_internal<K, V>(
        &self,
        key: K,
        count: i32,
        value: V,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        debug!(key = %key.to_string(), count = count, "Executing LREM");
        let timer = OperationTimer::new("list_remove_internal");

        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_remove.error").increment(1);
                return Err(e.into());
            }
        };

        let serialized_value = match self.serializer.serialize(&value).await {
            Ok(val) => val,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.list_remove.error").increment(1);
                return Err(e.into());
            }
        };

        let key_str_clone = key_str.clone();
        let serialized_value_clone = serialized_value.clone();

        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    redis::cmd("LREM")
                        .arg(&key_str_clone)
                        .arg(count)
                        .arg(&serialized_value_clone)
                        .query_async::<usize>(conn)
                        .await
                })
            })
            .await;

        match result {
            Ok(removed_count) => {
                timer.record_success();
                metrics::counter!("cache.list_remove.success").increment(removed_count as u64);
                Ok(removed_count)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.list_remove.error").increment(1);
                Err(cache_err.into())
            }
        }
    }
}
