use async_trait::async_trait;
use redis::{AsyncCommands, FromRedisValue};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::time::Duration;
use std::{collections::HashMap, collections::HashSet, hash::Hash, sync::Arc};
use tracing::{debug, instrument, warn};

use navius_cache::{
    error::{CacheError, CacheResult},
    operations::{Cache, CacheKey, CacheOperations, CacheOptions},
};

use crate::{
    connection::RedisConnectionManager,
    error::{RedisCacheError, RedisCacheResult},
    lua::{RedisLuaManager, RedisLuaScripting},
    metrics,
};

use redis::Aggregate;

#[derive(Debug, Clone, Copy)]
pub enum AggregateOptions {
    Sum,
    Min,
    Max,
}

// Replace the CacheSerializer trait with a concrete Box<dyn Fn> approach
/// Cache serializer function type
pub type SerializeFn<T> = Box<dyn Fn(&T) -> CacheResult<Vec<u8>> + Send + Sync>;
pub type DeserializeFn<T> = Box<dyn Fn(&[u8]) -> CacheResult<T> + Send + Sync>;

/// JSON serializer functions
pub struct JsonSerializer;

impl JsonSerializer {
    pub fn serialize<T: Serialize + Send + Sync>(
    ) -> impl Fn(&T) -> CacheResult<Vec<u8>> + Send + Sync {
        |value| serde_json::to_vec(value).map_err(|e| CacheError::SerializationError(e.to_string()))
    }

    pub fn deserialize<T: DeserializeOwned>() -> impl Fn(&[u8]) -> CacheResult<T> + Send + Sync {
        |bytes| {
            serde_json::from_slice(bytes).map_err(|e| CacheError::SerializationError(e.to_string()))
        }
    }
}

/// Redis cache implementation
pub struct RedisCache {
    /// Connection manager for Redis
    connection_manager: Arc<RedisConnectionManager<String>>,
    /// Lua script manager
    lua_manager: Option<Arc<RedisLuaManager>>,
}

impl RedisCache {
    /// Create a new Redis cache with default configuration
    pub async fn new(connection_manager: Arc<RedisConnectionManager<String>>) -> Self {
        // Create the Redis cache instance
        let mut cache = Self {
            connection_manager,
            lua_manager: None,
        };

        // Initialize the Lua manager
        let lua_manager = Arc::new(RedisLuaManager::new(cache.connection_manager().clone()).await);
        cache.lua_manager = Some(lua_manager);

        cache
    }

    /// Get the connection manager
    pub fn connection_manager(&self) -> &Arc<RedisConnectionManager<String>> {
        &self.connection_manager
    }

    /// Get the Lua manager
    pub fn lua_manager(&self) -> Option<&Arc<RedisLuaManager>> {
        self.lua_manager.as_ref()
    }

    /// Enable Lua scripting support
    pub async fn with_lua_scripting(mut self) -> Self {
        if self.lua_manager.is_none() {
            let lua_manager = Arc::new(RedisLuaManager::new(self.connection_manager().clone()).await);
            self.lua_manager = Some(lua_manager);
        }
        self
    }

    /// Convert a key to a string
    fn key_to_string<K: CacheKey>(&self, key: &K) -> String {
        key.to_string()
    }

    /// Serialize a value to bytes
    async fn serialize<T: Serialize + Send + Sync>(&self, value: &T) -> CacheResult<Vec<u8>> {
        serde_json::to_vec(value)
            .map_err(|e| RedisCacheError::Serialization(e.to_string()))
            .map_err(CacheError::from)
    }

    /// Deserialize bytes to a value
    async fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> CacheResult<T> {
        serde_json::from_slice(bytes)
            .map_err(|e| RedisCacheError::Serialization(e.to_string()))
            .map_err(CacheError::from)
    }

    /// Execute a raw Lua script with the given arguments
    #[instrument(skip(self, script, keys, args), level = "debug")]
    pub async fn execute_raw_script<'a, T>(
        &'a self,
        script: &'a str,
        keys: &'a [&'a str],
        args: &'a [&'a str],
    ) -> CacheResult<T>
    where
        T: FromRedisValue + Send + Sync + 'static,
    {
        if let Some(lua_manager) = self.lua_manager() {
            let keys: Vec<String> = keys.iter().map(|&k| k.to_string()).collect();
            let args: Vec<String> = args.iter().map(|&a| a.to_string()).collect();
            match lua_manager.execute_script::<T>("raw", &keys, &args).await {
                Ok(result) => Ok(result),
                Err(e) => Err(e.into()),
            }
        } else {
            Err(CacheError::LuaManagerNotInitialized)
        }
    }

    /// Register a script for later execution
    #[instrument(skip(self, name, script), level = "debug")]
    pub async fn register_script(&self, name: &str, script: &str) -> RedisCacheResult<()> {
        if let Some(lua_manager) = &self.lua_manager {
            match lua_manager.register_script(name, script).await {
                Ok(_) => Ok(()),
                Err(err) => Err(RedisCacheError::ScriptError(err.to_string())),
            }
        } else {
            Err(RedisCacheError::ScriptError(
                "Lua scripting not enabled".to_string(),
            ))
        }
    }

    /// Check and increment a counter atomically with a maximum value.
    /// Returns true if the counter was incremented successfully, false if it reached the maximum.
    #[instrument(skip(self), level = "debug")]
    pub async fn check_and_increment_counter(
        &self,
        key: &str,
        max_value: i64,
        ttl: Option<Duration>,
    ) -> RedisCacheResult<bool> {
        if let Some(lua_manager) = &self.lua_manager {
            let script = r#"
            local current = tonumber(redis.call('GET', KEYS[1])) or 0
            if current < tonumber(ARGV[1]) then
                redis.call('INCR', KEYS[1])
                if ARGV[2] ~= '' then
                    redis.call('EXPIRE', KEYS[1], ARGV[2])
                end
                return 1
            else
                return 0
            end
            "#;

            // Register the script if not already registered
            if !lua_manager.script_exists("check_and_increment") {
                match lua_manager
                    .register_script("check_and_increment", script)
                    .await
                {
                    Ok(_) => {}
                    Err(err) => return Err(RedisCacheError::ScriptError(err.to_string())),
                }
            }

            let prefixed_key = self.connection_manager.prefixed_key(key);
            let ttl_seconds = ttl.map(|t| t.as_secs().to_string()).unwrap_or_default();
            let keys = vec![prefixed_key];
            let args = vec![max_value.to_string(), ttl_seconds];

            match lua_manager
                .execute_script::<i64>("check_and_increment", &keys, &args)
                .await
            {
                Ok(result) => Ok(result == 1),
                Err(err) => Err(RedisCacheError::ScriptError(err.to_string())),
            }
        } else {
            Err(RedisCacheError::ScriptError(
                "Lua scripting not enabled".to_string(),
            ))
        }
    }

    /// Set a value only if the key doesn't exist (atomic SETNX with TTL)
    #[instrument(skip(self, value), level = "debug")]
    pub async fn set_if_not_exists<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> RedisCacheResult<bool> {
        if let Some(lua_manager) = &self.lua_manager {
            let serialized = self
                .serialize(value)
                .await
                .map_err(|e| RedisCacheError::Serialization(e.to_string()))?;

            match lua_manager
                .atomic_set_nx(
                    key,
                    &value,
                    ttl.unwrap_or(Duration::from_secs(3600)), // Default 1 hour TTL
                )
                .await
            {
                Ok(result) => Ok(result),
                Err(err) => Err(RedisCacheError::ScriptError(err.to_string())),
            }
        } else {
            Err(RedisCacheError::ScriptError(
                "Lua scripting not enabled".to_string(),
            ))
        }
    }

    /// Get a raw Redis value by key
    #[instrument(skip(self), level = "debug")]
    pub async fn get_raw(&self, key: &str) -> RedisCacheResult<Option<Vec<u8>>> {
        let timer = metrics::TimedOperation::new(metrics::names::GET);
        let result = self
            .connection_manager
            .execute_command(key, "GET", |mut conn| conn.get(key))
            .await;

        timer.record(&result);
        result
    }

    /// Set a raw Redis value by key
    #[instrument(skip(self, value), level = "debug")]
    pub async fn set_raw(&self, key: &str, value: Vec<u8>) -> RedisCacheResult<()> {
        let timer = metrics::TimedOperation::new(metrics::names::SET);
        let result = self
            .connection_manager
            .execute_command(key, "SET", |mut conn| conn.set(key, value))
            .await;

        timer.record(&result);
        result
    }

    /// Check if a key exists in Redis
    #[instrument(skip(self), level = "debug")]
    pub async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key);
        let key_str_clone = key_str.clone();

        let result = self
            .connection_manager
            .execute_command(&key_str, "EXISTS", |mut conn| async move {
                conn.exists(&key_str_clone).await
            })
            .await;

        result.map_err(|e| e.into())
    }

    /// Delete a key from Redis
    #[instrument(skip(self), level = "debug")]
    pub async fn delete<K>(&self, key: &K) -> CacheResult<bool>
    where
        K: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = self.key_to_string(key);
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let prefixed_key_clone = prefixed_key.clone();

        let result = self
            .connection_manager
            .execute_command(&prefixed_key, "DEL", move |mut conn| async move {
                let res: redis::RedisResult<i32> = redis::cmd("DEL")
                    .arg(&prefixed_key_clone)
                    .query_async(&mut conn)
                    .await;

                res.map(|count| count > 0)
            })
            .await?;

        Ok(result)
    }

    /// Set an expiration time for a key
    #[instrument(skip(self), level = "debug")]
    pub async fn expire_key<K: CacheKey + std::fmt::Debug + 'static>(
        &self,
        key: &K,
        ttl_secs: i64,
    ) -> CacheResult<bool> {
        let timer = TimedOperation::new(metrics::names::EXPIRE);
        let key_str = self.key_to_string(key);
        let key_str_clone = key_str.clone();

        let result = self
            .connection_manager
            .execute_command(&key_str, "EXPIRE", |mut conn| async move {
                conn.expire(&key_str_clone, ttl_secs as i64).await
            })
            .await;

        timer.record(&result);
        result.map_err(Into::into)
    }

    /// Get the time-to-live for a key in seconds
    #[instrument(skip(self), level = "debug")]
    pub async fn ttl<K>(&self, key: &K) -> CacheResult<Option<std::time::Duration>>
    where
        K: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = self.key_to_string(key);

        let result = self
            .connection_manager
            .execute_command(&key_str, "TTL", |mut conn| async move {
                let res: i64 = redis::cmd("TTL")
                    .arg(&key_str)
                    .query_async(&mut conn)
                    .await?;
                Ok(res)
            })
            .await;

        match result {
            Ok(ttl) => {
                if ttl < 0 {
                    Ok(None)
                } else {
                    Ok(Some(Duration::from_secs(ttl as u64)))
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    // === List operations ===

    /// Push a value to the right end of a list
    async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .rpush(&key_str, serialized)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Push multiple values to the right end of a list
    async fn list_push_right_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let serialized_values: Result<Vec<String>, _> = values
            .iter()
            .map(|v| serde_json::to_string(v).map_err(|e| CacheError::SerializationError(e.to_string())))
            .collect();
        let serialized_values = serialized_values?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .rpush(&key_str, &serialized_values)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Push a value to the left end of a list
    async fn list_push_left<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .lpush(&key_str, serialized)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Push multiple values to the left end of a list
    async fn list_push_left_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let serialized_values: Result<Vec<String>, _> = values
            .iter()
            .map(|v| serde_json::to_string(v).map_err(|e| CacheError::SerializationError(e.to_string())))
            .collect();
        let serialized_values = serialized_values?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .lpush(&key_str, &serialized_values)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Pop a value from the right end of a list
    async fn list_pop_right<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: Option<String> = conn
            .rpop(&key_str, Some(1))
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        match result {
            Some(value) => {
                let deserialized = serde_json::from_str(&value)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Pop a value from the left end of a list
    async fn list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: Option<String> = conn
            .lpop(&key_str, Some(1))
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        match result {
            Some(value) => {
                let deserialized = serde_json::from_str(&value)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Get a range of values from a list
    async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let results: Vec<String> = conn
            .lrange(&key_str, start, stop)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut values = Vec::with_capacity(results.len());
        for value in results {
            let deserialized = serde_json::from_str(&value)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            values.push(deserialized);
        }

        Ok(values)
    }

    /// Get the length of a list
    async fn list_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .llen(&key_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Remove elements equal to value from the list
    async fn list_remove<K, V>(&self, key: K, value: &V, count: i64) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .lrem(&key_str, count, serialized)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Trim a list to the specified range
    async fn list_trim<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        conn.ltrim(&key_str, start, stop)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(())
    }

    /// Set a value at a specific index in a list
    async fn list_set<K, V>(&self, key: K, index: isize, value: &V) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.connection_manager.get_connection().await?;

        conn.lset(&key_str, index, serialized)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(())
    }

    // === Set operations ===

    /// Add a member to a set
    #[instrument(skip(self, member), level = "debug")]
    async fn set_remove<K, V>(&self, key: K, member: &V) -> CacheResult<bool>
    where
        K: CacheKey + Debug + 'static,
        V: Serialize + Send + Sync + Debug + 'static,
    {
        let key_str = self.key_to_string(&key);
        let member_ser = self.serialize(member).await?;
        let mut conn = self.connection_manager.get().await?;
        let removed: i32 = conn.srem(&key_str, member_ser).await?;
        Ok(removed > 0)
    }

    /// Get all members of a set
    #[instrument(skip(self), level = "debug")]
    pub async fn set_members<K, V>(&self, key: &K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + Debug + 'static,
        V: DeserializeOwned + Debug + 'static,
    {
        let key_str = self.key_to_string(key);
        let key_str_clone = key_str.clone();

        let result: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&key_str, "SMEMBERS", |mut conn| async move {
                redis::cmd("SMEMBERS")
                    .arg(&key_str_clone)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        let mut values = Vec::with_capacity(result.len());
        for bytes in result {
            values.push(self.deserialize(&bytes).await?);
        }

        Ok(values)
    }

    /// Get the length of a set
    #[instrument(skip(self), level = "debug")]
    pub async fn set_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + Debug + 'static,
    {
        let key_str = self.key_to_string(&key);
        let key_str_clone = key_str.clone();

        let result = self
            .connection_manager
            .execute_command(&key_str, "SCARD", |mut conn| async move {
                redis::cmd("SCARD")
                    .arg(&key_str_clone)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        Ok(result)
    }

    /// Add values to a set
    async fn set_add<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let serialized_values: Result<Vec<String>, _> = values
            .iter()
            .map(|v| serde_json::to_string(v).map_err(|e| CacheError::SerializationError(e.to_string())))
            .collect();
        let serialized_values = serialized_values?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .sadd(&key_str, &serialized_values)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Remove values from a set
    async fn set_remove<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let serialized_values: Result<Vec<String>, _> = values
            .iter()
            .map(|v| serde_json::to_string(v).map_err(|e| CacheError::SerializationError(e.to_string())))
            .collect();
        let serialized_values = serialized_values?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .srem(&key_str, &serialized_values)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Check if a value is a member of a set
    async fn set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: bool = conn
            .sismember(&key_str, serialized)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result)
    }

    /// Get all members of a set
    async fn set_members<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let members: Vec<String> = conn
            .smembers(&key_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut values = Vec::with_capacity(members.len());
        for member in members {
            let deserialized = serde_json::from_str(&member)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            values.push(deserialized);
        }

        Ok(values)
    }

    /// Get the number of members in a set
    async fn set_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .scard(&key_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Get the intersection of multiple sets
    async fn set_intersection<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
        let mut conn = self.connection_manager.get_connection().await?;

        let members: Vec<String> = conn
            .sinter(&key_strs)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut values = Vec::with_capacity(members.len());
        for member in members {
            let deserialized = serde_json::from_str(&member)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            values.push(deserialized);
        }

        Ok(values)
    }

    /// Store the intersection of multiple sets in a destination set
    async fn set_intersection_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        let dest_str = destination.to_string();
        let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .sinterstore(&dest_str, &key_strs)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Get the union of multiple sets
    async fn set_union<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
        let mut conn = self.connection_manager.get_connection().await?;

        let members: Vec<String> = conn
            .sunion(&key_strs)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut values = Vec::with_capacity(members.len());
        for member in members {
            let deserialized = serde_json::from_str(&member)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            values.push(deserialized);
        }

        Ok(values)
    }

    /// Store the union of multiple sets in a destination set
    async fn set_union_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        let dest_str = destination.to_string();
        let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .sunionstore(&dest_str, &key_strs)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Get the difference between multiple sets
    async fn set_difference<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
        let mut conn = self.connection_manager.get_connection().await?;

        let members: Vec<String> = conn
            .sdiff(&key_strs)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut values = Vec::with_capacity(members.len());
        for member in members {
            let deserialized = serde_json::from_str(&member)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            values.push(deserialized);
        }

        Ok(values)
    }

    /// Store the difference between multiple sets in a destination set
    async fn set_difference_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        let dest_str = destination.to_string();
        let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .sdiffstore(&dest_str, &key_strs)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Get random members from a set
    async fn set_random_members<K, V>(&self, key: K, count: usize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let members: Vec<String> = if count == 1 {
            let result: Option<String> = conn
                .srandmember(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;
            result.into_iter().collect()
        } else {
            conn.srandmember_multiple(&key_str, count as isize)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?
        };

        let mut values = Vec::with_capacity(members.len());
        for member in members {
            let deserialized = serde_json::from_str(&member)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            values.push(deserialized);
        }

        Ok(values)
    }

    // === Hash operations ===

    /// Get a value from a hash
    async fn hash_get<K, F, V>(&self, key: K, field: F) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: Option<String> = conn
            .hget(&key_str, &field_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        match result {
            Some(value) => {
                let deserialized = serde_json::from_str(&value)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Set a value in a hash
    async fn hash_set<K, F, V>(&self, key: K, field: F, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: bool = conn
            .hset(&key_str, &field_str, serialized)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result)
    }

    /// Get multiple values from a hash
    async fn hash_get_many<K, F, V>(&self, key: K, fields: Vec<F>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let field_strs: Vec<String> = fields.into_iter().map(|f| f.to_string()).collect();
        let mut conn = self.connection_manager.get_connection().await?;

        let results: Vec<Option<String>> = conn
            .hmget(&key_str, &field_strs)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut values = Vec::with_capacity(results.len());
        for result in results {
            match result {
                Some(value) => {
                    let deserialized = serde_json::from_str(&value)
                        .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                    values.push(Some(deserialized));
                }
                None => values.push(None),
            }
        }

        Ok(values)
    }

    /// Set multiple values in a hash
    async fn hash_set_many<K, F, V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let mut field_values = Vec::with_capacity(entries.len() * 2);

        for (field, value) in entries {
            let field_str = field.to_string();
            let serialized = serde_json::to_string(&value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;
            field_values.push((field_str, serialized));
        }

        let mut conn = self.connection_manager.get_connection().await?;

        conn.hset_multiple(&key_str, &field_values)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(())
    }

    /// Check if a field exists in a hash
    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: bool = conn
            .hexists(&key_str, &field_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result)
    }

    /// Delete fields from a hash
    async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let field_strs: Vec<String> = fields.into_iter().map(|f| f.to_string()).collect();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .hdel(&key_str, &field_strs)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Get all entries from a hash
    async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let entries: HashMap<String, String> = conn
            .hgetall(&key_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut values = Vec::with_capacity(entries.len());
        for (field, value) in entries {
            let deserialized = serde_json::from_str(&value)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            values.push((field, deserialized));
        }

        Ok(values)
    }

    /// Get all fields from a hash
    async fn hash_keys<K>(&self, key: K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let keys: Vec<String> = conn
            .hkeys(&key_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(keys)
    }

    /// Get all values from a hash
    async fn hash_values<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let values: Vec<String> = conn
            .hvals(&key_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut deserialized_values = Vec::with_capacity(values.len());
        for value in values {
            let deserialized = serde_json::from_str(&value)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            deserialized_values.push(deserialized);
        }

        Ok(deserialized_values)
    }

    /// Increment a field in a hash
    async fn hash_increment<K, F>(&self, key: K, field: F, increment: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .hincr(&key_str, &field_str, increment)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result)
    }

    /// Get the number of fields in a hash
    async fn hash_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .hlen(&key_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Add members with scores to a sorted set
    async fn zset_add<K, V>(&self, key: K, items: Vec<(f64, V)>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let mut score_members = Vec::with_capacity(items.len() * 2);
        for (score, member) in items {
            let member_str = serde_json::to_string(&member)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;
            score_members.push((score, member_str));
        }

        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .zadd_multiple(&key_str, &score_members)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Remove members from a sorted set
    async fn zset_remove<K, V>(&self, key: K, members: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let member_strs: Result<Vec<String>, _> = members
            .iter()
            .map(|m| serde_json::to_string(m).map_err(|e| CacheError::SerializationError(e.to_string())))
            .collect();
        let member_strs = member_strs?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .zrem(&key_str, &member_strs)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Get the score of a member in a sorted set
    async fn zset_score<K, V>(&self, key: K, member: &V) -> CacheResult<Option<f64>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let member_str = serde_json::to_string(member)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: Option<f64> = conn
            .zscore(&key_str, member_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result)
    }

    /// Increment the score of a member in a sorted set
    async fn zset_increment_score<K, V>(&self, key: K, member: &V, increment: f64) -> CacheResult<f64>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let member_str = serde_json::to_string(member)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: f64 = conn
            .zincr(&key_str, member_str, increment)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result)
    }

    /// Get a range of members from a sorted set by rank
    async fn zset_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let members: Vec<String> = conn
            .zrange(&key_str, start, stop)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut values = Vec::with_capacity(members.len());
        for member in members {
            let deserialized = serde_json::from_str(&member)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            values.push(deserialized);
        }

        Ok(values)
    }

    /// Get a range of members with scores from a sorted set by rank
    async fn zset_range_with_scores<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let members: Vec<(String, f64)> = conn
            .zrange_withscores(&key_str, start, stop)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut values = Vec::with_capacity(members.len());
        for (member, score) in members {
            let deserialized = serde_json::from_str(&member)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            values.push((deserialized, score));
        }

        Ok(values)
    }

    /// Get a range of members from a sorted set by score
    async fn zset_range_by_score<K, V>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let members: Vec<String> = conn
            .zrangebyscore(&key_str, min, max)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut values = Vec::with_capacity(members.len());
        for member in members {
            let deserialized = serde_json::from_str(&member)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            values.push(deserialized);
        }

        Ok(values)
    }

    /// Get a range of members with scores from a sorted set by score
    async fn zset_range_by_score_with_scores<K, V>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let members: Vec<(String, f64)> = conn
            .zrangebyscore_withscores(&key_str, min, max)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut values = Vec::with_capacity(members.len());
        for (member, score) in members {
            let deserialized = serde_json::from_str(&member)
                .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
            values.push((deserialized, score));
        }

        Ok(values)
    }

    /// Get the rank of a member in a sorted set
    async fn zset_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let member_str = serde_json::to_string(member)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: Option<isize> = conn
            .zrank(&key_str, member_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result.map(|r| r as usize))
    }

    /// Get the reverse rank of a member in a sorted set
    async fn zset_reverse_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let member_str = serde_json::to_string(member)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.connection_manager.get_connection().await?;

        let result: Option<isize> = conn
            .zrevrank(&key_str, member_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result.map(|r| r as usize))
    }

    /// Get the number of members in a sorted set
    async fn zset_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .zcard(&key_str)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Count the number of members in a sorted set with scores within the given range
    async fn zset_count<K>(&self, key: K, min: f64, max: f64) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .zcount(&key_str, min, max)
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Store the intersection of multiple sorted sets in a destination sorted set
    async fn zset_intersection_store<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
        weights: Option<Vec<f64>>,
        aggregate: Option<String>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        if keys.is_empty() {
            return Ok(0);
        }

        let dest_str = destination.to_string();
        let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .zinterstore_weighted(&dest_str, &key_strs, weights.as_deref(), aggregate.as_deref())
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    /// Store the union of multiple sorted sets in a destination sorted set
    async fn zset_union_store<K, D>(
        &self,
        destination: D,
        keys: Vec<K>,
        weights: Option<Vec<f64>>,
        aggregate: Option<String>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        if keys.is_empty() {
            return Ok(0);
        }

        let dest_str = destination.to_string();
        let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
        let mut conn = self.connection_manager.get_connection().await?;

        let result: i64 = conn
            .zunionstore_weighted(&dest_str, &key_strs, weights.as_deref(), aggregate.as_deref())
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(result as usize)
    }

    #[async_trait]
    impl CacheOperations for RedisCache {
        /// Get a value from the cache
        async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: Option<String> = conn
                .get(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            match result {
                Some(value) => {
                    let deserialized = serde_json::from_str(&value)
                        .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                    Ok(Some(deserialized))
                }
                None => Ok(None),
            }
        }

        /// Get multiple values from the cache
        async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
            let mut conn = self.connection_manager.get_connection().await?;

            let results: Vec<Option<String>> = conn
                .mget(&key_strs)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(results.len());
            for result in results {
                match result {
                    Some(value) => {
                        let deserialized = serde_json::from_str(&value)
                            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                        values.push(Some(deserialized));
                    }
                    None => values.push(None),
                }
            }

            Ok(values)
        }

        /// Set a value in the cache
        async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let serialized = serde_json::to_string(value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            let mut conn = self.connection_manager.get_connection().await?;

            if let Some(opts) = options {
                if let Some(ttl) = opts.ttl {
                    conn.set_ex(&key_str, serialized, ttl.as_secs() as usize)
                        .await
                        .map_err(|e| CacheError::OperationError(e.to_string()))?;
                } else {
                    conn.set(&key_str, serialized)
                        .await
                        .map_err(|e| CacheError::OperationError(e.to_string()))?;
                }
            } else {
                conn.set(&key_str, serialized)
                    .await
                    .map_err(|e| CacheError::OperationError(e.to_string()))?;
            }

            Ok(())
        }

        /// Set multiple values in the cache
        async fn set_many<K, V>(
            &self,
            entries: Vec<(K, V)>,
            options: Option<CacheOptions>,
        ) -> CacheResult<()>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let mut conn = self.connection_manager.get_connection().await?;
            let mut pipeline = redis::pipe();

            for (key, value) in entries {
                let key_str = key.to_string();
                let serialized = serde_json::to_string(&value)
                    .map_err(|e| CacheError::SerializationError(e.to_string()))?;

                if let Some(opts) = &options {
                    if let Some(ttl) = opts.ttl {
                        pipeline.set_ex(&key_str, serialized, ttl.as_secs() as usize);
                    } else {
                        pipeline.set(&key_str, serialized);
                    }
                } else {
                    pipeline.set(&key_str, serialized);
                }
            }

            pipeline
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(())
        }

        /// Delete a value from the cache
        async fn delete<K>(&self, key: K) -> CacheResult<bool>
        where
            K: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .del(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result > 0)
        }

        /// Delete multiple values from the cache
        async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
        {
            let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .del(&key_strs)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        /// Check if a key exists in the cache
        async fn exists<K>(&self, key: K) -> CacheResult<bool>
        where
            K: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .exists(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result > 0)
        }

        /// Increment a counter in the cache
        async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
        where
            K: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .incr(&key_str, amount)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result)
        }

        /// Set expiry for a key
        async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
        where
            K: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: bool = conn
                .expire(&key_str, ttl.as_secs() as usize)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result)
        }

        /// Clear the entire cache
        async fn clear(&self) -> CacheResult<()> {
            let mut conn = self.connection_manager.get_connection().await?;

            conn.flushdb()
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(())
        }

        /// Check the health of the cache
        async fn health_check(&self) -> CacheResult<()> {
            let mut conn = self.connection_manager.get_connection().await?;

            let _: String = conn
                .ping()
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(())
        }

        #[instrument(skip(self, key, field, value))]
        async fn hash_set<K, F, V>(&self, key: K, field: F, value: &V) -> CacheResult<bool>
        where
            K: CacheKey + 'static,
            F: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let field_str = field.to_string();
            let serialized = serde_json::to_string(value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            let mut conn = self
                .connection_manager
                .get_connection()
                .await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

            let result: bool = conn
                .hset(&key_str, &field_str, serialized)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result)
        }

        #[instrument(skip(self, key, field))]
        async fn hash_get<K, F, V>(&self, key: K, field: F) -> CacheResult<Option<V>>
        where
            K: CacheKey + 'static,
            F: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let field_str = field.to_string();

            let mut conn = self
                .connection_manager
                .get_connection()
                .await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

            let result: Option<String> = conn
                .hget(&key_str, &field_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            match result {
                Some(value) => {
                    let deserialized = serde_json::from_str(&value)
                        .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                    Ok(Some(deserialized))
                }
                None => Ok(None),
            }
        }

        #[instrument(skip(self, key, fields))]
        async fn hash_get_many<K, F, V>(&self, key: K, fields: Vec<F>) -> CacheResult<Vec<Option<V>>>
        where
            K: CacheKey + 'static,
            F: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let field_strs: Vec<String> = fields.into_iter().map(|f| f.to_string()).collect();

            let mut conn = self
                .connection_manager
                .get_connection()
                .await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

            let results: Vec<Option<String>> = conn
                .hmget(&key_str, &field_strs)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(results.len());
            for result in results {
                match result {
                    Some(value) => {
                        let deserialized = serde_json::from_str(&value)
                            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                        values.push(Some(deserialized));
                    }
                    None => values.push(None),
                }
            }

            Ok(values)
        }

        #[instrument(skip(self, key, entries))]
        async fn hash_set_many<K, F, V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<()>
        where
            K: CacheKey + 'static,
            F: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let mut field_values = Vec::with_capacity(entries.len() * 2);

            for (field, value) in entries {
                let field_str = field.to_string();
                let serialized = serde_json::to_string(&value)
                    .map_err(|e| CacheError::SerializationError(e.to_string()))?;
                field_values.push((field_str, serialized));
            }

            let mut conn = self
                .connection_manager
                .get_connection()
                .await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

            conn.hset_multiple(&key_str, &field_values)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(())
        }

        #[instrument(skip(self, key, field))]
        async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
        where
            K: CacheKey + 'static,
            F: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let field_str = field.to_string();

            let mut conn = self
                .connection_manager
                .get_connection()
                .await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

            let result: bool = conn
                .hexists(&key_str, &field_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result)
        }

        #[instrument(skip(self, key, fields))]
        async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            F: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let field_strs: Vec<String> = fields.into_iter().map(|f| f.to_string()).collect();

            let mut conn = self
                .connection_manager
                .get_connection()
                .await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

            let result: i64 = conn
                .hdel(&key_str, &field_strs)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        #[instrument(skip(self, key))]
        async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();

            let mut conn = self
                .connection_manager
                .get_connection()
                .await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

            let entries: HashMap<String, String> = conn
                .hgetall(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(entries.len());
            for (field, value) in entries {
                let deserialized = serde_json::from_str(&value)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                values.push((field, deserialized));
            }

            Ok(values)
        }

        #[instrument(skip(self, key))]
        async fn hash_keys<K>(&self, key: K) -> CacheResult<Vec<String>>
        where
            K: CacheKey + 'static,
        {
            let key_str = key.to_string();

            let mut conn = self
                .connection_manager
                .get_connection()
                .await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

            let keys: Vec<String> = conn
                .hkeys(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(keys)
        }

        #[instrument(skip(self, key))]
        async fn hash_values<K, V>(&self, key: K) -> CacheResult<Vec<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();

            let mut conn = self
                .connection_manager
                .get_connection()
                .await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

            let values: Vec<String> = conn
                .hvals(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut deserialized_values = Vec::with_capacity(values.len());
            for value in values {
                let deserialized = serde_json::from_str(&value)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                deserialized_values.push(deserialized);
            }

            Ok(deserialized_values)
        }

        #[instrument(skip(self, key, field))]
        async fn hash_increment<K, F>(&self, key: K, field: F, increment: i64) -> CacheResult<i64>
        where
            K: CacheKey + 'static,
            F: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let field_str = field.to_string();

            let mut conn = self
                .connection_manager
                .get_connection()
                .await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

            let result: i64 = conn
                .hincr(&key_str, &field_str, increment)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result)
        }

        #[instrument(skip(self, key))]
        async fn hash_length<K>(&self, key: K) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
        {
            let key_str = key.to_string();

            let mut conn = self
                .connection_manager
                .get_connection()
                .await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

            let length: i64 = conn
                .hlen(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(length as usize)
        }

        // List operations
        async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let serialized = serde_json::to_string(value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .rpush(&key_str, serialized)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn list_push_right_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let serialized_values: Result<Vec<String>, _> = values
                .iter()
                .map(|v| serde_json::to_string(v).map_err(|e| CacheError::SerializationError(e.to_string())))
                .collect();
            let serialized_values = serialized_values?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .rpush(&key_str, &serialized_values)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn list_push_left<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let serialized = serde_json::to_string(value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .lpush(&key_str, serialized)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn list_push_left_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let serialized_values: Result<Vec<String>, _> = values
                .iter()
                .map(|v| serde_json::to_string(v).map_err(|e| CacheError::SerializationError(e.to_string())))
                .collect();
            let serialized_values = serialized_values?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .lpush(&key_str, &serialized_values)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn list_pop_right<K, V>(&self, key: K) -> CacheResult<Option<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: Option<String> = conn
                .rpop(&key_str, Some(1))
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            match result {
                Some(value) => {
                    let deserialized = serde_json::from_str(&value)
                        .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                    Ok(Some(deserialized))
                }
                None => Ok(None),
            }
        }

        async fn list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: Option<String> = conn
                .lpop(&key_str, Some(1))
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            match result {
                Some(value) => {
                    let deserialized = serde_json::from_str(&value)
                        .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                    Ok(Some(deserialized))
                }
                None => Ok(None),
            }
        }

        async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let results: Vec<String> = conn
                .lrange(&key_str, start, stop)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(results.len());
            for value in results {
                let deserialized = serde_json::from_str(&value)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                values.push(deserialized);
            }

            Ok(values)
        }

        async fn list_length<K>(&self, key: K) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .llen(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn list_remove<K, V>(&self, key: K, value: &V, count: i64) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let serialized = serde_json::to_string(value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .lrem(&key_str, count, serialized)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn list_trim<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
        where
            K: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            conn.ltrim(&key_str, start, stop)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(())
        }

        async fn list_set<K, V>(&self, key: K, index: isize, value: &V) -> CacheResult<()>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let serialized = serde_json::to_string(value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            let mut conn = self.connection_manager.get_connection().await?;

            conn.lset(&key_str, index, serialized)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(())
        }

        // Set operations
        async fn set_add<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let serialized_values: Result<Vec<String>, _> = values
                .iter()
                .map(|v| serde_json::to_string(v).map_err(|e| CacheError::SerializationError(e.to_string())))
                .collect();
            let serialized_values = serialized_values?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .sadd(&key_str, &serialized_values)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn set_remove<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let serialized_values: Result<Vec<String>, _> = values
                .iter()
                .map(|v| serde_json::to_string(v).map_err(|e| CacheError::SerializationError(e.to_string())))
                .collect();
            let serialized_values = serialized_values?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .srem(&key_str, &serialized_values)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let serialized = serde_json::to_string(value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: bool = conn
                .sismember(&key_str, serialized)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result)
        }

        async fn set_members<K, V>(&self, key: K) -> CacheResult<Vec<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let members: Vec<String> = conn
                .smembers(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(members.len());
            for member in members {
                let deserialized = serde_json::from_str(&member)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                values.push(deserialized);
            }

            Ok(values)
        }

        async fn set_length<K>(&self, key: K) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .scard(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn set_intersection<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
            let mut conn = self.connection_manager.get_connection().await?;

            let members: Vec<String> = conn
                .sinter(&key_strs)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(members.len());
            for member in members {
                let deserialized = serde_json::from_str(&member)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                values.push(deserialized);
            }

            Ok(values)
        }

        async fn set_intersection_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            D: CacheKey + 'static,
        {
            let dest_str = destination.to_string();
            let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .sinterstore(&dest_str, &key_strs)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn set_union<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
            let mut conn = self.connection_manager.get_connection().await?;

            let members: Vec<String> = conn
                .sunion(&key_strs)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(members.len());
            for member in members {
                let deserialized = serde_json::from_str(&member)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                values.push(deserialized);
            }

            Ok(values)
        }

        async fn set_union_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            D: CacheKey + 'static,
        {
            let dest_str = destination.to_string();
            let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .sunionstore(&dest_str, &key_strs)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn set_difference<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
            let mut conn = self.connection_manager.get_connection().await?;

            let members: Vec<String> = conn
                .sdiff(&key_strs)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(members.len());
            for member in members {
                let deserialized = serde_json::from_str(&member)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                values.push(deserialized);
            }

            Ok(values)
        }

        async fn set_difference_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            D: CacheKey + 'static,
        {
            let dest_str = destination.to_string();
            let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .sdiffstore(&dest_str, &key_strs)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn set_random_members<K, V>(&self, key: K, count: usize) -> CacheResult<Vec<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let members: Vec<String> = if count == 1 {
                let result: Option<String> = conn
                    .srandmember(&key_str)
                    .await
                    .map_err(|e| CacheError::OperationError(e.to_string()))?;
                result.into_iter().collect()
            } else {
                conn.srandmember_multiple(&key_str, count as isize)
                    .await
                    .map_err(|e| CacheError::OperationError(e.to_string()))?
            };

            let mut values = Vec::with_capacity(members.len());
            for member in members {
                let deserialized = serde_json::from_str(&member)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                values.push(deserialized);
            }

            Ok(values)
        }

        // Sorted set operations
        async fn zset_add<K, V>(&self, key: K, items: Vec<(f64, V)>) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let mut score_members = Vec::with_capacity(items.len() * 2);
            for (score, member) in items {
                let member_str = serde_json::to_string(&member)
                    .map_err(|e| CacheError::SerializationError(e.to_string()))?;
                score_members.push((score, member_str));
            }

            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .zadd_multiple(&key_str, &score_members)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn zset_remove<K, V>(&self, key: K, members: Vec<V>) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let member_strs: Result<Vec<String>, _> = members
                .iter()
                .map(|m| serde_json::to_string(m).map_err(|e| CacheError::SerializationError(e.to_string())))
                .collect();
            let member_strs = member_strs?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .zrem(&key_str, &member_strs)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn zset_score<K, V>(&self, key: K, member: &V) -> CacheResult<Option<f64>>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let member_str = serde_json::to_string(member)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: Option<f64> = conn
                .zscore(&key_str, member_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result)
        }

        async fn zset_increment_score<K, V>(&self, key: K, member: &V, increment: f64) -> CacheResult<f64>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let member_str = serde_json::to_string(member)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: f64 = conn
                .zincr(&key_str, member_str, increment)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result)
        }

        async fn zset_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let members: Vec<String> = conn
                .zrange(&key_str, start, stop)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(members.len());
            for member in members {
                let deserialized = serde_json::from_str(&member)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                values.push(deserialized);
            }

            Ok(values)
        }

        async fn zset_range_with_scores<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<(V, f64)>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let members: Vec<(String, f64)> = conn
                .zrange_withscores(&key_str, start, stop)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(members.len());
            for (member, score) in members {
                let deserialized = serde_json::from_str(&member)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                values.push((deserialized, score));
            }

            Ok(values)
        }

        async fn zset_range_by_score<K, V>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<V>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let members: Vec<String> = conn
                .zrangebyscore(&key_str, min, max)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(members.len());
            for member in members {
                let deserialized = serde_json::from_str(&member)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                values.push(deserialized);
            }

            Ok(values)
        }

        async fn zset_range_by_score_with_scores<K, V>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<(V, f64)>>
        where
            K: CacheKey + 'static,
            V: DeserializeOwned + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let members: Vec<(String, f64)> = conn
                .zrangebyscore_withscores(&key_str, min, max)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            let mut values = Vec::with_capacity(members.len());
            for (member, score) in members {
                let deserialized = serde_json::from_str(&member)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                values.push((deserialized, score));
            }

            Ok(values)
        }

        async fn zset_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let member_str = serde_json::to_string(member)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: Option<isize> = conn
                .zrank(&key_str, member_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result.map(|r| r as usize))
        }

        async fn zset_reverse_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
        where
            K: CacheKey + 'static,
            V: Serialize + Send + Sync + 'static,
        {
            let key_str = key.to_string();
            let member_str = serde_json::to_string(member)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            let mut conn = self.connection_manager.get_connection().await?;

            let result: Option<isize> = conn
                .zrevrank(&key_str, member_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result.map(|r| r as usize))
        }

        async fn zset_length<K>(&self, key: K) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .zcard(&key_str)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn zset_count<K>(&self, key: K, min: f64, max: f64) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
        {
            let key_str = key.to_string();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .zcount(&key_str, min, max)
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn zset_intersection_store<K, D>(
            &self,
            destination: D,
            keys: Vec<K>,
            weights: Option<Vec<f64>>,
            aggregate: Option<String>,
        ) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            D: CacheKey + 'static,
        {
            if keys.is_empty() {
                return Ok(0);
            }

            let dest_str = destination.to_string();
            let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .zinterstore_weighted(&dest_str, &key_strs, weights.as_deref(), aggregate.as_deref())
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }

        async fn zset_union_store<K, D>(
            &self,
            destination: D,
            keys: Vec<K>,
            weights: Option<Vec<f64>>,
            aggregate: Option<String>,
        ) -> CacheResult<usize>
        where
            K: CacheKey + 'static,
            D: CacheKey + 'static,
        {
            if keys.is_empty() {
                return Ok(0);
            }

            let dest_str = destination.to_string();
            let key_strs: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();
            let mut conn = self.connection_manager.get_connection().await?;

            let result: i64 = conn
                .zunionstore_weighted(&dest_str, &key_strs, weights.as_deref(), aggregate.as_deref())
                .await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;

            Ok(result as usize)
        }
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> Result<(), CacheError>
    where
        K: CacheKey + Send + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let value_str = serde_json::to_string(value)
            .map_err(|e| RedisCacheError::Serialization(e.to_string()))?;

        match self.connection_manager.execute_command(&key_str, "SET", |mut conn| async move {
            if let Some(opts) = options {
                if let Some(ttl) = opts.ttl {
                    conn.set_ex(&key_str, &value_str, ttl.as_secs() as usize).await
                } else {
                    conn.set(&key_str, &value_str).await
                }
            } else {
                conn.set(&key_str, &value_str).await
            }
        }).await {
            Ok(_) => Ok(()),
            Err(err) => {
                Err(RedisCacheError::Operation(format!(
                    "Failed to set key {}: {}",
                    key_str, err
                )).into())
            }
        }
    }
}
