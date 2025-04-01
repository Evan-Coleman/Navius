use async_trait::async_trait;
use redis::{AsyncCommands, FromRedisValue};
use serde::{de::DeserializeOwned, Serialize};
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
#[derive(Clone, Debug)]
pub struct RedisCache {
    /// Connection manager for Redis
    connection_manager: Arc<RedisConnectionManager>,
    /// Lua script manager
    lua_manager: Option<Arc<RedisLuaManager>>,
}

impl RedisCache {
    /// Create a new Redis cache with default configuration
    pub fn new(connection_manager: Arc<RedisConnectionManager>) -> Self {
        // Create the Redis cache instance
        let mut cache = Self {
            connection_manager,
            lua_manager: None,
        };

        // Initialize the Lua manager
        let lua_manager = Arc::new(RedisLuaManager::new(cache.connection_manager().clone()));
        cache.lua_manager = Some(lua_manager);

        cache
    }

    /// Get the connection manager
    pub fn connection_manager(&self) -> &Arc<RedisConnectionManager> {
        &self.connection_manager
    }

    /// Get the Lua manager
    pub fn lua_manager(&self) -> Option<&Arc<RedisLuaManager>> {
        self.lua_manager.as_ref()
    }

    /// Enable Lua scripting support
    pub fn with_lua_scripting(mut self) -> Self {
        if self.lua_manager.is_none() {
            let lua_manager = Arc::new(RedisLuaManager::new(self.connection_manager().clone()));
            self.lua_manager = Some(lua_manager);
        }
        self
    }

    /// Convert a key to a string
    async fn key_to_string<K: CacheKey>(&self, key: &K) -> CacheResult<String> {
        Ok(key.to_string())
    }

    /// Serialize a value to bytes
    async fn serialize<T: Serialize + Send + Sync>(&self, value: &T) -> CacheResult<Vec<u8>> {
        serde_json::to_vec(value)
            .map_err(|e| RedisCacheError::SerializationError(e.to_string()).into())
    }

    /// Deserialize bytes to a value
    async fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> CacheResult<T> {
        serde_json::from_slice(bytes)
            .map_err(|e| RedisCacheError::DeserializationError(e.to_string()).into())
    }

    /// Execute a raw Lua script with the given arguments
    #[instrument(skip(self, script, keys, args), level = "debug")]
    pub async fn execute_raw_script<T: FromRedisValue + Send + Sync>(
        &self,
        script: &str,
        keys: &[&str],
        args: &[&str],
    ) -> RedisCacheResult<T> {
        if let Some(lua_manager) = &self.lua_manager {
            match lua_manager.execute_script("raw", keys, args).await {
                Ok(result) => Ok(result),
                Err(err) => Err(RedisCacheError::ScriptError(err.to_string())),
            }
        } else {
            Err(RedisCacheError::ScriptError(
                "Lua scripting not enabled".to_string(),
            ))
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

            match lua_manager
                .execute_script::<i64>(
                    "check_and_increment",
                    &[&prefixed_key],
                    &[&max_value.to_string(), &ttl_seconds],
                )
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
        let key_str = self.key_to_string(&key).await?;
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
        let key_str = self.key_to_string(key).await?;
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
        let key_str = self.key_to_string(key).await?;
        let key_str_clone = key_str.clone();

        let result = self
            .connection_manager
            .execute_command(&key_str, "EXPIRE", |mut conn| async move {
                // Convert to usize as required by redis library
                conn.expire(&key_str_clone, ttl_secs as usize).await
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
        let key_str = self.key_to_string(key).await?;

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

    /// Push a value to the end of a list
    #[instrument(skip(self, value), level = "debug")]
    pub async fn list_push<K, V>(&self, key: &K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key).await?;
        let value_ser = self.serialize(value).await?;

        let result = self
            .connection_manager
            .execute_command(&key_str, "RPUSH", |mut conn| async move {
                let len: usize = redis::cmd("RPUSH")
                    .arg(&key_str)
                    .arg(value_ser)
                    .query_async(&mut conn)
                    .await?;
                Ok(len)
            })
            .await;

        match result {
            Ok(len) => Ok(len),
            Err(e) => Err(e.into()),
        }
    }

    /// Pop a value from the end of a list
    #[instrument(skip(self), level = "debug")]
    pub async fn list_pop<K>(&self, key: &K) -> CacheResult<Option<Vec<u8>>>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(key).await?;

        let result = self
            .connection_manager
            .execute_command(&key_str, "RPOP", |mut conn| async move {
                let res: Option<Vec<u8>> = redis::cmd("RPOP")
                    .arg(&key_str)
                    .query_async(&mut conn)
                    .await?;
                Ok(res)
            })
            .await;

        match result {
            Ok(popped) => Ok(popped),
            Err(e) => Err(e.into()),
        }
    }

    /// Get a range of values from a list
    #[instrument(skip(self), level = "debug")]
    pub async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        debug!("Getting list range for key: {}", prefixed_key);

        let results_bytes: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "LRANGE", |mut conn| async move {
                conn.lrange(&prefixed_key, start, stop).await
            })
            .await?;

        use futures::stream::{self, StreamExt};
        let results = stream::iter(results_bytes)
            .then(|bytes| async move { self.deserialize(&bytes).await })
            .collect::<Vec<CacheResult<V>>>()
            .await;

        results.into_iter().collect::<CacheResult<Vec<V>>>()
    }

    /// Get the length of a list
    #[instrument(skip(self), level = "debug")]
    pub async fn list_length<K>(&self, key: &K) -> CacheResult<usize>
    where
        K: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = self.key_to_string(key).await?;
        let key_str_clone = key_str.clone();

        let timer = metrics::TimedOperation::new(metrics::names::LIST_LENGTH);
        let result = self
            .connection_manager
            .execute_command(&key_str, "LLEN", |mut conn| async move {
                conn.llen(&key_str_clone).await
            })
            .await;

        timer.record(&result);
        result.map_err(|e| e.into())
    }

    /// Trim a list to the specified range
    #[instrument(skip(self), level = "debug")]
    async fn list_trim<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let prefixed_key_str = self.connection_manager.prefixed_key(&key_str);
        let prefixed_key_clone = prefixed_key_str.clone();

        self.connection_manager
            .execute_command(&prefixed_key_str, "LTRIM", move |mut conn| async move {
                let result: redis::RedisResult<()> = redis::cmd("LTRIM")
                    .arg(&prefixed_key_clone)
                    .arg(start)
                    .arg(stop)
                    .query_async(&mut conn)
                    .await;

                result
            })
            .await?;

        Ok(())
    }

    // === Set operations ===

    /// Add a member to a set
    #[instrument(skip(self, member), level = "debug")]
    pub async fn set_add<K, V>(&self, key: &K, member: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        V: Serialize + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key).await?;
        let member_ser = self.serialize(member).await?;

        let result = self
            .connection_manager
            .execute_command(&key_str, "SADD", |mut conn| async move {
                let added: i32 = conn.sadd(&key_str, member_ser).await?;
                Ok(added > 0)
            })
            .await;

        match result {
            Ok(added) => Ok(added),
            Err(e) => Err(e.into()),
        }
    }

    /// Remove a member from a set
    #[instrument(skip(self, member), level = "debug")]
    pub async fn set_remove<K, V>(&self, key: &K, member: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        V: Serialize + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key).await?;
        let member_ser = self.serialize(member).await?;

        let result = self
            .connection_manager
            .execute_command(&key_str, "SREM", |mut conn| async move {
                let removed: i32 = conn.srem(&key_str, member_ser).await?;
                Ok(removed > 0)
            })
            .await;

        match result {
            Ok(removed) => Ok(removed),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all members of a set
    #[instrument(skip(self), level = "debug")]
    pub async fn set_members<K>(&self, key: &K) -> CacheResult<Vec<Vec<u8>>>
    where
        K: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = self.key_to_string(key).await?;
        let key_str_clone = key_str.clone();

        let result = self
            .connection_manager
            .execute_command(&key_str, "SMEMBERS", |mut conn| async move {
                let members: Vec<Vec<u8>> = conn.smembers(&key_str_clone).await?;
                Ok(members)
            })
            .await;

        match result {
            Ok(members) => Ok(members),
            Err(e) => Err(e.into()),
        }
    }

    // === Hash operations ===

    /// Set a field in a hash
    #[instrument(skip(self, field, value), level = "debug")]
    pub async fn hash_set(&self, key: &str, field: &str, value: Vec<u8>) -> RedisCacheResult<bool> {
        let timer = metrics::TimedOperation::new(metrics::names::HASH_SET);
        let result = self
            .connection_manager
            .execute_command(key, "HSET", |mut conn| {
                let res: i32 = redis::cmd("HSET")
                    .arg(key)
                    .arg(field)
                    .arg(value)
                    .query(&mut conn)?;
                Ok(res > 0)
            })
            .await;

        timer.record(&result);
        result
    }

    /// Get a field from a hash
    #[instrument(skip(self), level = "debug")]
    pub async fn hash_get<K, F, V>(&self, key: &K, field: &F) -> CacheResult<Option<V>>
    where
        K: CacheKey + Sync + Send + std::fmt::Debug + 'static,
        F: Serialize + Sync + Send + std::fmt::Debug + 'static,
        V: DeserializeOwned + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let field_str = self.key_to_string(field)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result_bytes = self
            .connection_manager
            .execute_command(&prefixed_key, "HGET", |mut conn| async move {
                let res: Option<Vec<u8>> = redis::cmd("HGET")
                    .arg(&prefixed_key)
                    .arg(&field_str)
                    .query_async(&mut conn)
                    .await?;
                Ok(res)
            })
            .await?;

        match result_bytes {
            Some(bytes) => self.deserialize(&bytes).await.map(Some),
            None => Ok(None),
        }
    }

    /// Delete a field from a hash
    #[instrument(skip(self), level = "debug")]
    pub async fn hash_delete<K, F>(&self, key: &K, field: &F) -> CacheResult<bool>
    where
        K: CacheKey + Sync + Send + std::fmt::Debug + 'static,
        F: Serialize + Sync + Send + std::fmt::Debug + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let field_str = self.key_to_string(field)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result = self
            .connection_manager
            .execute_command(&prefixed_key, "HDEL", |mut conn| async move {
                let res: i32 = redis::cmd("HDEL")
                    .arg(&prefixed_key)
                    .arg(&field_str)
                    .query_async(&mut conn)
                    .await?;
                Ok(res > 0)
            })
            .await?;
        Ok(result)
    }

    #[instrument(skip(self, fields), level = "debug")]
    async fn hash_get_many<K, F, V>(&self, key: K, fields: Vec<F>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + 'static,
    {
        if fields.is_empty() {
            return Ok(Vec::new());
        }

        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let field_strs: Vec<String> = fields.iter().map(|f| self.key_to_string(f)?).collect();

        let mut cmd = redis::cmd("HMGET");
        cmd.arg(&prefixed_key);
        for field in &field_strs {
            cmd.arg(field);
        }

        let results_bytes: Vec<Option<Vec<u8>>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HMGET", |mut conn| async move {
                cmd.query_async(&mut conn).await
            })
            .await?;

        // Use futures::stream to handle async deserialization concurrently
        use futures::stream::{self, StreamExt};
        let results = stream::iter(results_bytes)
            .then(|bytes_opt| async move {
                match bytes_opt {
                    Some(bytes) => self.deserialize(&bytes).await.map(Some),
                    None => Ok(None),
                }
            })
            .collect::<Vec<CacheResult<Option<V>>>>()
            .await;

        // Collect results, propagating the first error if any
        results.into_iter().collect::<CacheResult<Vec<Option<V>>>>()
    }

    #[instrument(skip(self, entries), level = "debug")]
    async fn hash_set_many<K, F, V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        F: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + std::fmt::Debug + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let mut cmd = redis::cmd("HMSET");
        cmd.arg(&prefixed_key);

        for (field, value) in entries {
            let field_ser = self.serialize(&field).await?;
            let value_ser = self.serialize(&value).await?;
            cmd.arg(field_ser).arg(value_ser);
        }

        self.connection_manager
            .execute_command(&prefixed_key, "HMSET", |mut conn| async move {
                cmd.query_async(&mut conn).await
            })
            .await?;

        Ok(())
    }

    #[instrument(skip(self, field), level = "debug")]
    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let field_str = self.key_to_string(&field).await?;

        // Clone the strings for use in the closure
        let key_str_clone = key_str.clone();
        let field_str_clone = field_str.clone();

        let result = self
            .connection_manager
            .execute_command(&key_str, "HEXISTS", |mut conn| async move {
                conn.hexists(&key_str_clone, &field_str_clone).await
            })
            .await;

        result.map_err(|e| e.into())
    }

    #[instrument(skip(self, fields), level = "debug")]
    async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key).await?;

        // Convert all fields to strings asynchronously
        let mut field_strs = Vec::with_capacity(fields.len());
        for field in fields {
            let field_str = self.key_to_string(&field).await?;
            field_strs.push(field_str);
        }

        // Clone for use in closure
        let key_str_clone = key_str.clone();

        let result = self
            .connection_manager
            .execute_command(&key_str, "HDEL", |mut conn| async move {
                let count: i32 = conn.hdel(&key_str_clone, field_strs).await?;
                Ok(count as usize)
            })
            .await;

        result.map_err(|e| e.into())
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn set_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let prefixed_key_clone = prefixed_key.clone();

        let result = self
            .connection_manager
            .execute_command(&prefixed_key, "SCARD", move |mut conn| async move {
                let result: redis::RedisResult<usize> = redis::cmd("SCARD")
                    .arg(&prefixed_key_clone)
                    .query_async(&mut conn)
                    .await;

                result
            })
            .await?;

        Ok(result)
    }

    // Start Stubs for missing CacheOperations methods

    async fn set_intersection<K, V>(&self, _keys: Vec<K>) -> CacheResult<HashSet<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Eq + Hash + Send + Sync + 'static,
    {
        todo!("set_intersection not implemented for RedisCache")
    }

    async fn set_intersection_store<K, D>(
        &self,
        _destination: D,
        _keys: Vec<K>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        todo!("set_intersection_store not implemented for RedisCache")
    }

    async fn set_union<K, V>(&self, _keys: Vec<K>) -> CacheResult<HashSet<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Eq + Hash + Send + Sync + 'static,
    {
        todo!("set_union not implemented for RedisCache")
    }

    async fn set_union_store<K, D>(&self, _destination: D, _keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        todo!("set_union_store not implemented for RedisCache")
    }

    async fn set_difference<K, V>(&self, _keys: Vec<K>) -> CacheResult<HashSet<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Eq + Hash + Send + Sync + 'static,
    {
        todo!("set_difference not implemented for RedisCache")
    }

    async fn set_difference_store<K, D>(&self, _destination: D, _keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        todo!("set_difference_store not implemented for RedisCache")
    }

    async fn set_random_members<K, V>(&self, _key: K, _count: usize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        todo!("set_random_members not implemented for RedisCache")
    }

    async fn zset_add<K, V>(&self, key: K, items: Vec<(f64, V)>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + std::fmt::Debug + 'static,
    {
        if items.is_empty() {
            return Ok(0);
        }

        let key_str = self.key_to_string(&key).await?;
        let prefixed_key_str = self.connection_manager.prefixed_key(&key_str);

        let mut added = 0;
        for (score, value) in items {
            let serialized = self.serialize(&value).await?;
            let prefixed_key_clone = prefixed_key_str.clone();
            let serialized_clone = serialized.clone();

            let result: bool = self
                .connection_manager
                .execute_command(&prefixed_key_str, "ZADD", move |mut conn| async move {
                    let result: redis::RedisResult<bool> = redis::cmd("ZADD")
                        .arg(&prefixed_key_clone)
                        .arg(score)
                        .arg(serialized_clone)
                        .query_async(&mut conn)
                        .await;

                    result
                })
                .await?;

            if result {
                added += 1;
            }
        }

        Ok(added)
    }

    async fn zset_remove<K, V>(&self, _key: K, _members: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("zset_remove not implemented for RedisCache")
    }

    async fn zset_score<K, V>(&self, _key: K, _member: &V) -> CacheResult<Option<f64>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("zset_score not implemented for RedisCache")
    }

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
        todo!("zset_increment_score not implemented for RedisCache")
    }

    async fn zset_range<K, V>(&self, _key: K, _start: isize, _stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        todo!("zset_range not implemented for RedisCache")
    }

    async fn zset_range_with_scores<K, V>(
        &self,
        _key: K,
        _start: isize,
        _stop: isize,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        todo!("zset_range_with_scores not implemented for RedisCache")
    }

    async fn zset_range_by_score<K, V>(&self, _key: K, _min: f64, _max: f64) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        todo!("zset_range_by_score not implemented for RedisCache")
    }

    async fn zset_range_by_score_with_scores<K, V>(
        &self,
        _key: K,
        _min: f64,
        _max: f64,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        todo!("zset_range_by_score_with_scores not implemented for RedisCache")
    }

    async fn zset_rank<K, V>(&self, _key: K, _member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("zset_rank not implemented for RedisCache")
    }

    async fn zset_reverse_rank<K, V>(&self, _key: K, _member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("zset_reverse_rank not implemented for RedisCache")
    }

    async fn zset_length<K>(&self, _key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        todo!("zset_length not implemented for RedisCache")
    }

    async fn zset_count<K>(&self, _key: K, _min: f64, _max: f64) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        todo!("zset_count not implemented for RedisCache")
    }

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
        todo!("zset_intersection_store not implemented for RedisCache")
    }

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
        todo!("zset_union_store not implemented for RedisCache")
    }

    // End Stubs for missing CacheOperations methods

    async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let prefixed_key_str = self.connection_manager.prefixed_key(&key_str);
        let prefixed_key_clone = prefixed_key_str.clone();

        let result: HashMap<String, Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key_str, "HGETALL", move |mut conn| async move {
                let result: redis::RedisResult<HashMap<String, Vec<u8>>> = redis::cmd("HGETALL")
                    .arg(&prefixed_key_clone)
                    .query_async(&mut conn)
                    .await;

                result
            })
            .await?;

        let mut entries = Vec::with_capacity(result.len());
        for (field, value_bytes) in result {
            let value = self.deserialize(&value_bytes).await?;
            entries.push((field, value));
        }

        Ok(entries)
    }

    async fn hash_keys<K>(&self, _key: K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + 'static,
    {
        todo!("Implement hash_keys method")
    }
}

impl Cache for RedisCache {}

#[async_trait]
impl CacheOperations for RedisCache {
    /// Implement the set_contains method to check if a member exists in a set
    async fn set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let serialized = self.serialize(value).await?;
        let prefixed_key_str = self.connection_manager.prefixed_key(&key_str);
        let prefixed_key_clone = prefixed_key_str.clone();

        let result = self
            .connection_manager
            .execute_command(&prefixed_key_str, "SISMEMBER", move |mut conn| async move {
                let result: redis::RedisResult<bool> = redis::cmd("SISMEMBER")
                    .arg(&prefixed_key_clone)
                    .arg(&serialized)
                    .query_async(&mut conn)
                    .await;

                result
            })
            .await?;

        Ok(result)
    }

    /// Get a value from the cache
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let key_str_clone = key_str.clone();

        let result = self
            .connection_manager
            .execute_command(&key_str, "GET", |mut conn| async move {
                let data: Option<Vec<u8>> = conn.get(&key_str_clone).await?;
                Ok(data)
            })
            .await?;

        match result {
            Some(bytes) => {
                let value = self.deserialize(&bytes).await?;
                Ok(Some(value))
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
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.get(key).await?);
        }
        Ok(results)
    }

    /// Set a value in the cache
    async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let ttl = options.and_then(|opts| opts.ttl);
        let key_str = self.key_to_string(&key).await?;
        let value_bytes = self.serialize(value).await?;

        let key_str_clone = key_str.clone();

        self.connection_manager
            .execute_command(&key_str, "SET", |mut conn| async move {
                match ttl {
                    Some(ttl) => {
                        conn.set_ex(&key_str_clone, value_bytes, ttl.as_secs() as usize)
                            .await?;
                    }
                    None => {
                        conn.set(&key_str_clone, value_bytes).await?;
                    }
                }
                Ok(())
            })
            .await?;

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
        let ttl = options.and_then(|opts| opts.ttl);
        for (key, value) in entries {
            self.set(key, &value, ttl).await?;
        }
        Ok(())
    }

    /// Delete a value from the cache
    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        let timer = TimedOperation::new(metrics::names::DELETE);
        let key_str = self.key_to_string(&key).await?;
        let key_str_clone = key_str.clone();

        let result = self
            .connection_manager
            .execute_command(&key_str, "DEL", |mut conn| async move {
                let res: i32 = conn.del(&key_str_clone).await?;
                Ok(res > 0)
            })
            .await;

        timer.record(&result);
        result.map_err(Into::into)
    }

    /// Delete multiple values from the cache
    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let mut count = 0;
        for key in keys {
            if self.delete(&key).await? {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Check if a key exists in the cache
    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let key_str_clone = key_str.clone();

        let result = self
            .connection_manager
            .execute_command(&key_str, "EXISTS", |mut conn| async move {
                conn.exists(&key_str_clone).await
            })
            .await;

        result.map_err(|e| e.into())
    }

    /// Increment a counter in the cache
    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let key_str_clone = key_str.clone();

        let result = self
            .connection_manager
            .execute_command(&key_str, "INCRBY", |mut conn| async move {
                conn.incr(&key_str_clone, amount).await
            })
            .await;

        result.map_err(|e| e.into())
    }

    /// Expire a key in the cache
    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        let ttl_secs = ttl.as_secs() as i64;
        let key_str = self.key_to_string(&key).await?;
        let key_str_clone = key_str.clone();
        let timer = TimedOperation::new(metrics::names::EXPIRE);

        let result = self
            .connection_manager
            .execute_command(&key_str, "EXPIRE", |mut conn| async move {
                conn.expire(&key_str_clone, ttl_secs as usize).await
            })
            .await;

        timer.record(&result);
        result.map_err(Into::into)
    }

    /// Clear the entire cache
    async fn clear(&self) -> CacheResult<()> {
        self.connection_manager
            .execute_command("", "FLUSHDB", |mut conn| async move {
                let _: String = redis::cmd("FLUSHDB").query_async(&mut conn).await?;
                Ok(())
            })
            .await
            .map_err(Into::into)
    }

    /// Get the health status of the cache
    async fn health_check(&self) -> CacheResult<()> {
        self.connection_manager
            .execute_command("", "PING", |mut conn| async move {
                let response: String = redis::cmd("PING").query_async(&mut conn).await?;
                if response == "PONG" {
                    Ok(())
                } else {
                    Err(redis::RedisError::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Unexpected response: {}", response),
                    )))
                }
            })
            .await
            .map_err(Into::into)
    }

    // Add stub implementations for the remaining methods with todo!()
    async fn list_push_right<K, V>(&self, _key: K, _value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement list_push_right method")
    }

    async fn list_push_right_many<K, V>(&self, _key: K, _values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement list_push_right_many method")
    }

    async fn list_push_left<K, V>(&self, _key: K, _value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement list_push_left method")
    }

    async fn list_push_left_many<K, V>(&self, _key: K, _values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement list_push_left_many method")
    }

    async fn list_pop_right<K, V>(&self, _key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement list_pop_right method")
    }

    async fn list_pop_left<K, V>(&self, _key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement list_pop_left method")
    }

    async fn list_range<K, V>(&self, _key: K, _start: isize, _stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement list_range method")
    }

    async fn list_length<K>(&self, _key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        todo!("Implement list_length method")
    }

    async fn list_remove<K, V>(&self, _key: K, _count: isize, _value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement list_remove method")
    }

    async fn list_trim<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let prefixed_key_str = self.connection_manager.prefixed_key(&key_str);
        let prefixed_key_clone = prefixed_key_str.clone();

        self.connection_manager
            .execute_command(&prefixed_key_str, "LTRIM", move |mut conn| async move {
                let result: redis::RedisResult<()> = redis::cmd("LTRIM")
                    .arg(&prefixed_key_clone)
                    .arg(start)
                    .arg(stop)
                    .query_async(&mut conn)
                    .await;

                result
            })
            .await?;

        Ok(())
    }

    async fn list_set<K, V>(&self, _key: K, _index: isize, _value: &V) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement list_set method")
    }

    async fn hash_get<K, F, V>(&self, _key: K, _field: F) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement hash_get method")
    }

    async fn hash_set<K, F, V>(&self, _key: K, _field: F, _value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement hash_set method")
    }

    async fn hash_get_many<K, F, V>(&self, _key: K, _fields: Vec<F>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement hash_get_many method")
    }

    async fn hash_set_many<K, F, V>(&self, _key: K, _entries: Vec<(F, V)>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement hash_set_many method")
    }

    async fn hash_exists<K, F>(&self, _key: K, _field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        todo!("Implement hash_exists method")
    }

    async fn hash_delete<K, F>(&self, _key: K, _fields: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        todo!("Implement hash_delete method")
    }

    async fn hash_keys<K>(&self, _key: K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + 'static,
    {
        todo!("Implement hash_keys method")
    }

    async fn hash_values<K, V>(&self, _key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement hash_values method")
    }

    async fn hash_increment<K, F>(&self, _key: K, _field: F, _amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        todo!("Implement hash_increment method")
    }

    async fn hash_length<K>(&self, _key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        todo!("Implement hash_length method")
    }

    async fn set_add<K, V>(&self, _key: K, _members: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement set_add method")
    }

    async fn set_remove<K, V>(&self, _key: K, _members: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement set_remove method")
    }

    async fn set_members<K, V>(&self, _key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement set_members method")
    }

    async fn set_length<K>(&self, _key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        todo!("Implement set_length method")
    }

    async fn set_intersection<K, V>(&self, _keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement set_intersection method")
    }

    async fn set_intersection_store<K, D>(
        &self,
        _destination: D,
        _keys: Vec<K>,
    ) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        todo!("Implement set_intersection_store method")
    }

    async fn set_union<K, V>(&self, _keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement set_union method")
    }

    async fn set_union_store<K, D>(&self, _destination: D, _keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        todo!("Implement set_union_store method")
    }

    async fn set_difference<K, V>(&self, _keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement set_difference method")
    }

    async fn set_difference_store<K, D>(&self, _destination: D, _keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        todo!("Implement set_difference_store method")
    }

    async fn set_random_members<K, V>(&self, _key: K, _count: usize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement set_random_members method")
    }

    async fn zset_add<K, V>(&self, _key: K, _items: Vec<(f64, V)>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement zset_add method")
    }

    async fn zset_remove<K, V>(&self, _key: K, _members: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement zset_remove method")
    }

    async fn zset_score<K, V>(&self, _key: K, _member: &V) -> CacheResult<Option<f64>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement zset_score method")
    }

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
        todo!("Implement zset_increment_score method")
    }

    async fn zset_range<K, V>(&self, _key: K, _start: isize, _stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement zset_range method")
    }

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
        todo!("Implement zset_range_with_scores method")
    }

    async fn zset_range_by_score<K, V>(&self, _key: K, _min: f64, _max: f64) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        todo!("Implement zset_range_by_score method")
    }

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
        todo!("Implement zset_range_by_score_with_scores method")
    }

    async fn zset_rank<K, V>(&self, _key: K, _member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement zset_rank method")
    }

    async fn zset_reverse_rank<K, V>(&self, _key: K, _member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("Implement zset_reverse_rank method")
    }

    async fn zset_length<K>(&self, _key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        todo!("Implement zset_length method")
    }

    async fn zset_count<K>(&self, _key: K, _min: f64, _max: f64) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        todo!("Implement zset_count method")
    }

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
        todo!("Implement zset_intersection_store method")
    }

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
        todo!("Implement zset_union_store method")
    }
}
