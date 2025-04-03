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
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument, trace, warn};

// Set operations will be moved here

impl RedisCache {
    pub(crate) async fn set_add_internal<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_set_add");
        metrics::counter!("cache.set_add.total").increment(1);

        // Convert key to string
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set_add.error").increment(1);
                // Return Err instead of Ok(0)
                return Err(e.into());
            }
        };

        // If there are no values, return immediately
        if values.is_empty() {
            timer.record_success(); // Technically success, 0 added
            return Ok(0);
        }

        // Serialize all values
        let mut serialized_values = Vec::with_capacity(values.len());
        for value in values {
            match self.serializer.serialize(&value).await {
                Ok(v) => serialized_values.push(v),
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_add.error").increment(1);
                    return Err(e.into());
                }
            }
        }

        // Add values to the set in Redis
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    let mut cmd = redis::cmd("SADD");
                    cmd.arg(&key_str);
                    cmd.arg(&serialized_values);
                    cmd.query_async::<usize>(conn).await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(val) => {
                timer.record_success();
                metrics::counter!("cache.set_add.success").increment(val as u64);
                Ok(val)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_add.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub(crate) async fn set_remove_internal<K, V>(
        &self,
        key: K,
        values: Vec<V>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_set_remove");
        metrics::counter!("cache.set_remove.total").increment(1);

        // Convert key to string
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set_remove.error").increment(1);
                // Return Err instead of Ok(0)
                return Err(e.into());
            }
        };

        // If there are no values, return immediately
        if values.is_empty() {
            timer.record_success(); // Technically success, 0 removed
            return Ok(0);
        }

        // Serialize all values
        let mut serialized_values = Vec::with_capacity(values.len());
        for value in values {
            match self.serializer.serialize(&value).await {
                Ok(v) => serialized_values.push(v),
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_remove.error").increment(1);
                    return Err(e.into());
                }
            }
        }

        // Remove values from the set in Redis
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    let mut cmd = redis::cmd("SREM");
                    cmd.arg(&key_str);
                    cmd.arg(&serialized_values);
                    cmd.query_async::<usize>(conn).await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(val) => {
                timer.record_success();
                metrics::counter!("cache.set_remove.success").increment(val as u64);
                Ok(val)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_remove.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub(crate) async fn set_contains_internal<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        V: serde::Serialize + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_set_contains");
        metrics::counter!("cache.set_contains.total").increment(1);

        // Convert key to string
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set_contains.error").increment(1);
                return Err(e.into()); // Error on invalid key
            }
        };

        // Serialize the value
        let value_str = match self.serializer.serialize(value).await {
            Ok(v) => v,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set_contains.error").increment(1);
                return Err(e.into()); // Error on serialization failure
            }
        };

        // Check if the value is in the set
        let result: Result<bool, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    redis::cmd("SISMEMBER")
                        .arg(&key_str)
                        .arg(&value_str)
                        .query_async::<bool>(conn)
                        .await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(val) => {
                timer.record_success();
                metrics::counter!("cache.set_contains.success").increment(1);
                if val {
                    metrics::counter!("cache.set_contains.hit").increment(1);
                } else {
                    metrics::counter!("cache.set_contains.miss").increment(1);
                }
                Ok(val)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_contains.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub(crate) async fn set_members_internal<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let timer = OperationTimer::new("cache_set_members");
        metrics::counter!("cache.set_members.total").increment(1);

        // Initialize gauge
        let gauge = metrics::gauge!("cache.set_members.count");
        gauge.set(0.0);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set_members.error").increment(1);
                // Return Err instead of Ok(Vec::new())
                return Err(e.into());
            }
        };

        // Get all members from the set in Redis
        let result: Result<Vec<String>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    redis::cmd("SMEMBERS")
                        .arg(&key_str)
                        .query_async::<Vec<String>>(conn)
                        .await
                })
            })
            .await;

        // Handle result and deserialize
        match result {
            Ok(members) => {
                timer.record_success();
                metrics::counter!("cache.set_members.success").increment(1);
                gauge.set(members.len() as f64);

                let mut deserialized_members = Vec::with_capacity(members.len());
                for member_str in members {
                    match self
                        .serializer
                        .deserialize::<V>(member_str.as_bytes())
                        .await
                    {
                        Ok(member) => deserialized_members.push(member),
                        Err(e) => {
                            timer.record_error(&e);
                            metrics::counter!("cache.set_members.error").increment(1);
                            // Decide if we should return partial results or error out
                            return Err(e.into());
                        }
                    }
                }
                Ok(deserialized_members)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_members.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub(crate) async fn set_length_internal<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let timer = OperationTimer::new("cache_set_length");
        metrics::counter!("cache.set_length.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set_length.error").increment(1);
                // Return Err instead of Ok(0)
                return Err(e.into());
            }
        };

        // Get the length of the set in Redis
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    redis::cmd("SCARD")
                        .arg(&key_str)
                        .query_async::<usize>(conn)
                        .await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(val) => {
                timer.record_success();
                metrics::counter!("cache.set_length.success").increment(1);
                Ok(val)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_length.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub(crate) async fn set_intersection_internal<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        let timer = OperationTimer::new("cache_set_intersection");
        metrics::counter!("cache.set_intersection.total").increment(1);

        // Convert keys to strings and validate
        if keys.is_empty() {
            timer.record_success();
            return Ok(Vec::new());
        }

        let mut key_strings = Vec::with_capacity(keys.len());
        for key in keys {
            match self.key_to_string(key) {
                Ok(key_str) => key_strings.push(key_str),
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_intersection.error").increment(1);
                    return Err(e.into()); // Error if any key is invalid
                }
            }
        }

        // Get the intersection of the sets in Redis
        let result: Result<Vec<String>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    redis::cmd("SINTER")
                        .arg(&key_strings)
                        .query_async::<Vec<String>>(conn)
                        .await
                })
            })
            .await;

        // Handle result and deserialize
        match result {
            Ok(intersection) => {
                timer.record_success();
                metrics::counter!("cache.set_intersection.success").increment(1);
                let mut deserialized_intersection = Vec::with_capacity(intersection.len());
                for value_str in intersection {
                    match self.serializer.deserialize::<V>(value_str.as_bytes()).await {
                        Ok(value) => deserialized_intersection.push(value),
                        Err(e) => {
                            timer.record_error(&e);
                            metrics::counter!("cache.set_intersection.error").increment(1);
                            return Err(e.into());
                        }
                    }
                }
                Ok(deserialized_intersection)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_intersection.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub(crate) async fn set_intersection_store_internal<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        let timer = OperationTimer::new("cache_set_intersection_store");
        metrics::counter!("cache.set_intersection_store.total").increment(1);

        // Convert destination to string
        let destination_str = match self.key_to_string(destination) {
            Ok(d) => d,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set_intersection_store.error").increment(1);
                // Return Err instead of Ok(0)
                return Err(e.into());
            }
        };

        // Convert keys to strings and validate
        if keys.is_empty() {
            // SINTERSTORE with no source keys will delete the destination key
            // TODO: Revisit if this behavior is desired.
            // For now, let's delete the destination key as per Redis behavior.
            match self.delete(destination_str.clone()).await {
                Ok(_) => { /* Key deleted or didn't exist */ }
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_intersection_store.error").increment(1);
                    return Err(e);
                }
            };
            timer.record_success();
            return Ok(0);
        }

        let mut key_strings = Vec::with_capacity(keys.len());
        for key in keys {
            match self.key_to_string(key) {
                Ok(key_str) => key_strings.push(key_str),
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_intersection_store.error").increment(1);
                    // Return Err instead of Ok(0)
                    return Err(e.into());
                }
            }
        }

        // Store the intersection of the sets in Redis
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    redis::cmd("SINTERSTORE")
                        .arg(&destination_str)
                        .arg(&key_strings)
                        .query_async::<usize>(conn)
                        .await
                })
            })
            .await;

        match result {
            Ok(val) => {
                timer.record_success();
                metrics::counter!("cache.set_intersection_store.success").increment(val as u64);
                Ok(val)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_intersection_store.error").increment(1);
                Err(cache_err)
            }
        }
    }

    pub(crate) async fn set_union_internal<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        let timer = OperationTimer::new("cache_set_union");
        metrics::counter!("cache.set_union.total").increment(1);

        // Convert keys to strings and validate
        if keys.is_empty() {
            timer.record_success();
            return Ok(Vec::new());
        }

        let mut key_strings = Vec::with_capacity(keys.len());
        for key in keys {
            match self.key_to_string(key) {
                Ok(key_str) => key_strings.push(key_str),
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_union.error").increment(1);
                    return Err(e.into()); // Error if any key is invalid
                }
            }
        }

        // Get the union of the sets in Redis
        let result: Result<Vec<String>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    redis::cmd("SUNION")
                        .arg(&key_strings)
                        .query_async::<Vec<String>>(conn)
                        .await
                })
            })
            .await;

        // Handle result and deserialize
        match result {
            Ok(union) => {
                timer.record_success();
                metrics::counter!("cache.set_union.success").increment(1);
                let mut deserialized_union = Vec::with_capacity(union.len());
                for value_str in union {
                    match self.serializer.deserialize::<V>(value_str.as_bytes()).await {
                        Ok(value) => deserialized_union.push(value),
                        Err(e) => {
                            timer.record_error(&e);
                            metrics::counter!("cache.set_union.error").increment(1);
                            return Err(e.into());
                        }
                    }
                }
                Ok(deserialized_union)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_union.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub(crate) async fn set_union_store_internal<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        let timer = OperationTimer::new("cache_set_union_store");
        metrics::counter!("cache.set_union_store.total").increment(1);

        // Convert destination to string
        let destination_str = match self.key_to_string(destination) {
            Ok(d) => d,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set_union_store.error").increment(1);
                // Return Err instead of Ok(0)
                return Err(e.into());
            }
        };

        // Convert keys to strings and validate
        if keys.is_empty() {
            // SUNIONSTORE with no source keys results in an empty set at destination.
            // Mimic this by ensuring it's empty if it exists.
            match self.delete(destination_str.clone()).await {
                Ok(_) => { /* Key deleted or didn't exist */ }
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_union_store.error").increment(1);
                    return Err(e);
                }
            };
            timer.record_success();
            return Ok(0);
        }

        let mut key_strings = Vec::with_capacity(keys.len());
        for key in keys {
            match self.key_to_string(key) {
                Ok(key_str) => key_strings.push(key_str),
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_union_store.error").increment(1);
                    // Return Err instead of Ok(0)
                    return Err(e.into());
                }
            }
        }

        // Store the union of the sets in Redis
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    redis::cmd("SUNIONSTORE")
                        .arg(&destination_str)
                        .arg(&key_strings)
                        .query_async::<usize>(conn)
                        .await
                })
            })
            .await;

        match result {
            Ok(val) => {
                timer.record_success();
                metrics::counter!("cache.set_union_store.success").increment(val as u64);
                Ok(val)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_union_store.error").increment(1);
                Err(cache_err)
            }
        }
    }

    pub(crate) async fn set_difference_internal<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        let timer = OperationTimer::new("cache_set_difference");
        metrics::counter!("cache.set_difference.total").increment(1);

        // Convert keys to strings and validate
        if keys.is_empty() {
            // SDIFF requires at least one key in some Redis versions?
            // Let's return empty vector for consistency with SINTER/SUNION
            timer.record_success();
            return Ok(Vec::new());
        }
        // The redis crate handles the case where keys has only one element fine.

        let mut key_strings = Vec::with_capacity(keys.len());
        for key in keys {
            match self.key_to_string(key) {
                Ok(key_str) => key_strings.push(key_str),
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_difference.error").increment(1);
                    return Err(e.into()); // Error if any key is invalid
                }
            }
        }

        // Get the difference between the sets in Redis
        let result: Result<Vec<String>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    redis::cmd("SDIFF")
                        .arg(&key_strings)
                        .query_async::<Vec<String>>(conn)
                        .await
                })
            })
            .await;

        // Handle result and deserialize
        match result {
            Ok(difference) => {
                timer.record_success();
                metrics::counter!("cache.set_difference.success").increment(1);
                let mut deserialized_difference = Vec::with_capacity(difference.len());
                for value_str in difference {
                    match self.serializer.deserialize::<V>(value_str.as_bytes()).await {
                        Ok(value) => deserialized_difference.push(value),
                        Err(e) => {
                            timer.record_error(&e);
                            metrics::counter!("cache.set_difference.error").increment(1);
                            return Err(e.into());
                        }
                    }
                }
                Ok(deserialized_difference)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_difference.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub(crate) async fn set_difference_store_internal<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        let timer = OperationTimer::new("cache_set_difference_store");
        metrics::counter!("cache.set_difference_store.total").increment(1);

        // Convert destination to string
        let destination_str = match self.key_to_string(destination) {
            Ok(d) => d,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set_difference_store.error").increment(1);
                // Return Err instead of Ok(0)
                return Err(e.into());
            }
        };

        // Convert keys to strings and validate
        if keys.is_empty() {
            // SDIFFSTORE requires at least one source key
            let err = RedisCacheError::InvalidArgument(
                "SDIFFSTORE requires at least one key".to_string(),
            );
            timer.record_error(&err);
            metrics::counter!("cache.set_difference_store.error").increment(1);
            return Err(err.into());
        }

        let mut key_strings = Vec::with_capacity(keys.len());
        for key in keys {
            match self.key_to_string(key) {
                Ok(key_str) => key_strings.push(key_str),
                Err(e) => {
                    timer.record_error(&e);
                    metrics::counter!("cache.set_difference_store.error").increment(1);
                    // Return Err instead of Ok(0)
                    return Err(e.into());
                }
            }
        }

        // Store the difference between the sets in Redis
        let result: Result<usize, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    redis::cmd("SDIFFSTORE")
                        .arg(&destination_str)
                        .arg(&key_strings)
                        .query_async::<usize>(conn)
                        .await
                })
            })
            .await;

        // Simplified error handling
        match result {
            Ok(val) => {
                timer.record_success();
                metrics::counter!("cache.set_difference_store.success").increment(val as u64);
                Ok(val)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_difference_store.error").increment(1);
                Err(cache_err.into())
            }
        }
    }

    pub(crate) async fn set_random_members_internal<K, V>(
        &self,
        key: K,
        count: usize,
    ) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        let timer = OperationTimer::new("cache_set_random_members");
        metrics::counter!("cache.set_random_members.total").increment(1);

        // Convert key to string and validate
        let key_str = match self.key_to_string(key) {
            Ok(k) => k,
            Err(e) => {
                timer.record_error(&e);
                metrics::counter!("cache.set_random_members.error").increment(1);
                return Err(e.into()); // Error on invalid key
            }
        };

        // Basic validation: Ensure count is not negative (although usize prevents this)
        // Redis SRANDMEMBER handles count > set size gracefully.
        if count > i64::MAX as usize {
            // Check against potential Redis limitations
            let err = RedisCacheError::InvalidKey(
                // Use InvalidKey instead of InvalidArgument
                format!("SRANDMEMBER count {} exceeds reasonable limits", count),
            );
            timer.record_error(&err);
            metrics::counter!("cache.set_random_members.error").increment(1);
            return Err(err.into());
        }

        // Get random members from the set in Redis
        let result: Result<Vec<String>, RedisCacheError> = self
            .pool
            .execute(move |conn| {
                Box::pin(async move {
                    redis::cmd("SRANDMEMBER")
                        .arg(&key_str)
                        .arg(count)
                        .query_async::<Vec<String>>(conn)
                        .await
                })
            })
            .await;

        // Handle result and deserialize
        match result {
            Ok(members) => {
                timer.record_success();
                metrics::counter!("cache.set_random_members.success").increment(1);
                let mut deserialized_members = Vec::with_capacity(members.len());
                for member_str in members {
                    match self
                        .serializer
                        .deserialize::<V>(member_str.as_bytes())
                        .await
                    {
                        Ok(member) => deserialized_members.push(member),
                        Err(e) => {
                            timer.record_error(&e);
                            metrics::counter!("cache.set_random_members.error").increment(1);
                            return Err(e.into());
                        }
                    }
                }
                Ok(deserialized_members)
            }
            Err(cache_err) => {
                timer.record_error(&cache_err);
                metrics::counter!("cache.set_random_members.error").increment(1);
                Err(cache_err.into())
            }
        }
    }
}
