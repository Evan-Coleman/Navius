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
#[derive(Clone)]
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

    // Update helper methods to handle async properly
    async fn key_to_string<K: CacheKey>(&self, key: &K) -> CacheResult<String> {
        key.to_string()
            .map_err(|e| CacheError::SerializationError(e.to_string()))
    }

    async fn serialize_sync<T: Serialize>(&self, value: &T) -> CacheResult<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| CacheError::SerializationError(e.to_string()))
    }

    // Helper method to serialize a value
    async fn serialize<T: Serialize + Send + Sync>(&self, value: &T) -> CacheResult<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| CacheError::SerializationError(e.to_string()))
    }

    // Helper method to deserialize a value
    async fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> CacheResult<T> {
        serde_json::from_slice(bytes).map_err(|e| CacheError::SerializationError(e.to_string()))
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
    pub async fn exists(&self, key: &str) -> RedisCacheResult<bool> {
        let timer = metrics::TimedOperation::new(metrics::names::EXISTS);
        let result = self
            .connection_manager
            .execute_command(key, "EXISTS", |mut conn| conn.exists(key))
            .await;

        timer.record(&result);
        result
    }

    /// Delete a key from Redis
    #[instrument(skip(self), level = "debug")]
    pub async fn delete(&self, key: &str) -> RedisCacheResult<bool> {
        let timer = metrics::TimedOperation::new(metrics::names::DELETE);
        let result = self
            .connection_manager
            .execute_command(key, "DEL", |mut conn| {
                let res: i32 = redis::cmd("DEL").arg(key).query(&mut conn)?;
                Ok(res > 0)
            })
            .await;

        timer.record(&result);
        result
    }

    /// Set the expiration time for a key
    #[instrument(skip(self), level = "debug")]
    pub async fn expire(&self, key: &str, seconds: usize) -> RedisCacheResult<bool> {
        let timer = metrics::TimedOperation::new(metrics::names::EXPIRE);
        let result = self
            .connection_manager
            .execute_command(key, "EXPIRE", |mut conn| {
                let res: i32 = redis::cmd("EXPIRE")
                    .arg(key)
                    .arg(seconds)
                    .query(&mut conn)?;
                Ok(res > 0)
            })
            .await;

        timer.record(&result);
        result
    }

    /// Get the time-to-live for a key in seconds
    #[instrument(skip(self), level = "debug")]
    pub async fn ttl(&self, key: &str) -> RedisCacheResult<i64> {
        let timer = metrics::TimedOperation::new(metrics::names::TTL);
        let result = self
            .connection_manager
            .execute_command(key, "TTL", |mut conn| {
                let res: i64 = redis::cmd("TTL").arg(key).query(&mut conn)?;
                Ok(res)
            })
            .await;

        timer.record(&result);
        result
    }

    // === List operations ===

    /// Push a value to the end of a list
    #[instrument(skip(self, value), level = "debug")]
    pub async fn list_push(&self, key: &str, value: Vec<u8>) -> RedisCacheResult<usize> {
        let timer = metrics::TimedOperation::new(metrics::names::LIST_PUSH);
        let result = self
            .connection_manager
            .execute_command(key, "RPUSH", |mut conn| {
                let len: usize = redis::cmd("RPUSH").arg(key).arg(value).query(&mut conn)?;
                Ok(len)
            })
            .await;

        timer.record(&result);
        result
    }

    /// Pop a value from the end of a list
    #[instrument(skip(self), level = "debug")]
    pub async fn list_pop(&self, key: &str) -> RedisCacheResult<Option<Vec<u8>>> {
        let timer = metrics::TimedOperation::new(metrics::names::LIST_POP);
        let result = self
            .connection_manager
            .execute_command(key, "RPOP", |mut conn| {
                let res: Option<Vec<u8>> = redis::cmd("RPOP").arg(key).query(&mut conn)?;
                Ok(res)
            })
            .await;

        timer.record(&result);
        result
    }

    /// Get a range of values from a list
    #[instrument(skip(self), level = "debug")]
    pub async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = self.key_to_string(&key)?;
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
    pub async fn list_len(&self, key: &str) -> RedisCacheResult<usize> {
        let timer = metrics::TimedOperation::new(metrics::names::LIST_LENGTH);
        let result = self
            .connection_manager
            .execute_command(key, "LLEN", |mut conn| {
                let len: usize = redis::cmd("LLEN").arg(key).query(&mut conn)?;
                Ok(len)
            })
            .await;

        timer.record(&result);
        result
    }

    // === Set operations ===

    /// Add a member to a set
    #[instrument(skip(self, member), level = "debug")]
    pub async fn set_add(&self, key: &str, member: Vec<u8>) -> RedisCacheResult<bool> {
        let timer = metrics::TimedOperation::new(metrics::names::SET_ADD);
        let result = self
            .connection_manager
            .execute_command(key, "SADD", |mut conn| {
                let added: i32 = redis::cmd("SADD").arg(key).arg(member).query(&mut conn)?;
                Ok(added > 0)
            })
            .await;

        timer.record(&result);
        result
    }

    /// Remove a member from a set
    #[instrument(skip(self, member), level = "debug")]
    pub async fn set_remove(&self, key: &str, member: Vec<u8>) -> RedisCacheResult<bool> {
        let timer = metrics::TimedOperation::new(metrics::names::SET_REMOVE);
        let result = self
            .connection_manager
            .execute_command(key, "SREM", |mut conn| {
                let removed: i32 = redis::cmd("SREM").arg(key).arg(member).query(&mut conn)?;
                Ok(removed > 0)
            })
            .await;

        timer.record(&result);
        result
    }

    /// Get all members of a set
    #[instrument(skip(self), level = "debug")]
    pub async fn set_members(&self, key: &str) -> RedisCacheResult<Vec<Vec<u8>>> {
        let timer = metrics::TimedOperation::new(metrics::names::SET_MEMBERS);
        let result = self
            .connection_manager
            .execute_command(key, "SMEMBERS", |mut conn| {
                let members: Vec<Vec<u8>> = redis::cmd("SMEMBERS").arg(key).query(&mut conn)?;
                Ok(members)
            })
            .await;

        timer.record(&result);
        result
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
        K: CacheKey + 'static + std::fmt::Debug,
        F: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = self.key_to_string(&key)?;
        let field_str = self.key_to_string(&field)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        self.connection_manager
            .execute_command(&prefixed_key, "HEXISTS", |mut conn| async move {
                conn.hexists(&prefixed_key, &field_str).await
            })
            .await
    }

    #[instrument(skip(self, fields), level = "debug")]
    async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        F: CacheKey + 'static + std::fmt::Debug,
    {
        if fields.is_empty() {
            return Ok(0);
        }

        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let field_strs: Vec<String> = fields
            .into_iter()
            .map(|f| self.key_to_string(&f)?)
            .collect();

        self.connection_manager
            .execute_command(&prefixed_key, "HDEL", |mut conn| async move {
                conn.hdel(&prefixed_key, field_strs).await
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_get_all<K, V>(&self, key: &K) -> CacheResult<HashMap<String, V>>
    where
        K: CacheKey + Sync + Send + 'static,
        V: DeserializeOwned + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result_map: HashMap<String, Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HGETALL", |mut conn| async move {
                redis::cmd("HGETALL")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        let mut deserialized_map = HashMap::new();
        for (field, bytes) in result_map {
            let value = self.deserialize(&bytes).await?;
            deserialized_map.insert(field, value);
        }
        Ok(deserialized_map)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_keys<K>(&self, key: &K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result_keys: Vec<String> = self
            .connection_manager
            .execute_command(&prefixed_key, "HKEYS", |mut conn| async move {
                redis::cmd("HKEYS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
                    .await
            })
            .await?;
        Ok(result_keys)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_values<K, V>(&self, key: &K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + Sync + Send + 'static,
        V: DeserializeOwned + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result_values: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HVALS", |mut conn| async move {
                redis::cmd("HVALS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        use futures::stream::{self, StreamExt};
        let values = stream::iter(result_values)
            .then(|bytes| async move { self.deserialize(&bytes).await })
            .collect::<Vec<CacheResult<V>>>()
            .await;

        values.into_iter().collect::<CacheResult<Vec<V>>>()
    }

    #[instrument(skip(self, field), level = "debug")]
    async fn hash_increment<K, F>(&self, key: &K, field: &F, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + Sync + Send + 'static,
        F: Serialize + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let field_str = self.key_to_string(field)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let new_value: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "HINCRBY", |mut conn| async move {
                redis::cmd("HINCRBY")
                    .arg(&prefixed_key)
                    .arg(&field_str)
                    .arg(amount)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        Ok(new_value)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_length<K>(&self, key: &K) -> CacheResult<usize>
    where
        K: CacheKey + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let length: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "HLEN", |mut conn| async move {
                redis::cmd("HLEN")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        Ok(length)
    }

    // Set Operations

    #[instrument(skip(self, values), level = "debug")]
    async fn set_add<K, V>(&self, key: &K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if values.is_empty() {
            return Ok(0);
        }
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized_values = values
            .iter()
            .map(|v| self.serialize_sync(v))
            .collect::<CacheResult<Vec<Vec<u8>>>>()?;

        self.connection_manager
            .execute_command(&prefixed_key, "SADD", |mut conn| async move {
                conn.sadd(&prefixed_key, serialized_values).await
            })
            .await
    }

    #[instrument(skip(self, values), level = "debug")]
    async fn set_remove<K, V>(&self, key: &K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if values.is_empty() {
            return Ok(0);
        }
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized_values = values
            .iter()
            .map(|v| self.serialize_sync(v))
            .collect::<CacheResult<Vec<Vec<u8>>>>()?;

        self.connection_manager
            .execute_command(&prefixed_key, "SREM", |mut conn| async move {
                conn.srem(&prefixed_key, serialized_values).await
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + std::fmt::Debug + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;

        let is_member: bool = self
            .connection_manager
            .execute_command(&prefixed_key, "SISMEMBER", |mut conn| async move {
                redis::cmd("SISMEMBER")
                    .arg(&prefixed_key)
                    .arg(serialized)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        Ok(is_member)
    }

    #[instrument(skip(self), level = "debug")]
    async fn set_members<K, V>(&self, key: &K) -> CacheResult<HashSet<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Eq + Hash + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let members_bytes: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "SMEMBERS", |mut conn| async move {
                redis::cmd("SMEMBERS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        let mut members = HashSet::new();
        for bytes in members_bytes {
            let member = self.deserialize(&bytes).await?;
            members.insert(member);
        }
        Ok(members)
    }

    #[instrument(skip(self), level = "debug")]
    async fn set_length<K>(&self, key: &K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        self.connection_manager
            .execute_command(&prefixed_key, "SCARD", |mut conn| async move {
                conn.scard(&prefixed_key).await
            })
            .await
    }
}

#[async_trait]
impl CacheOperations for RedisCache {
    #[instrument(skip(self), level = "debug")]
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + std::fmt::Debug + Send + Sync + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result: Option<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "GET", |mut conn| async move {
                conn.get(&prefixed_key).await
            })
            .await
            .map_err(CacheError::from)?;

        match result {
            Some(bytes) => self.deserialize(&bytes).await.map(Some),
            None => Ok(None),
        }
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + std::fmt::Debug + Send + Sync + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let prefixed_keys: Vec<String> =
            futures::future::join_all(keys.iter().map(|k| async move {
                let key_str = self.key_to_string(k).await?;
                Ok(self.connection_manager.prefixed_key(&key_str))
            }))
            .await
            .into_iter()
            .collect::<CacheResult<Vec<_>>>()?;

        let routing_key = &prefixed_keys[0];
        let results: Vec<Option<Vec<u8>>> = self
            .connection_manager
            .execute_command(routing_key, "MGET", |mut conn| async move {
                conn.get(prefixed_keys).await
            })
            .await
            .map_err(CacheError::from)?;

        futures::future::join_all(results.into_iter().map(|bytes_opt| async move {
            match bytes_opt {
                Some(bytes) => self.deserialize(&bytes).await.map(Some),
                None => Ok(None),
            }
        }))
        .await
        .into_iter()
        .collect()
    }

    #[instrument(skip(self, value, options), level = "debug")]
    async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + Send + Sync + 'static,
        V: Serialize + Send + Sync + std::fmt::Debug + 'static,
    {
        let key_str = self.key_to_string(&key).await?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;

        self.connection_manager
            .execute_command(&prefixed_key, "SET", |mut conn| async move {
                let mut cmd = redis::cmd("SET");
                cmd.arg(&prefixed_key).arg(serialized);
                if let Some(opts) = options {
                    if let Some(ttl) = opts.ttl {
                        cmd.arg("PX").arg(ttl.as_millis() as u64);
                    }
                }
                cmd.query_async(&mut conn).await
            })
            .await
            .map_err(CacheError::from)?;
        Ok(())
    }

    #[instrument(skip(self, entries, options), level = "debug")]
    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<CacheOptions>,
    ) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + std::fmt::Debug + 'static,
    {
        if entries.is_empty() {
            return Ok(());
        }

        let use_pipeline = options.map_or(false, |o| o.ttl.is_some());

        if !use_pipeline {
            let prefixed_entries: Vec<(String, Vec<u8>)> = entries
                .into_iter()
                .map(|(k, v)| {
                    let key_str = self.key_to_string(&k)?;
                    let prefixed_key = self.connection_manager.prefixed_key(&key_str);
                    let serialized = self.serialize_sync(&v)?;
                    Ok((prefixed_key, serialized))
                })
                .collect::<CacheResult<Vec<(String, Vec<u8>)>>>()?;

            let routing_key = &prefixed_entries[0].0;
            self.connection_manager
                .execute_command(routing_key, "MSET", |mut conn| async move {
                    conn.set_multiple(&prefixed_entries).await
                })
                .await?;
        } else {
            let ttl_ms = options.and_then(|o| o.ttl).map(|d| d.as_millis() as usize);
            let mut pipe = redis::pipe();
            let mut routing_key = String::new();

            for (i, (key, value)) in entries.into_iter().enumerate() {
                let key_str = self.key_to_string(&key)?;
                let prefixed_key = self.connection_manager.prefixed_key(&key_str);
                if i == 0 {
                    routing_key = prefixed_key.clone();
                }
                let serialized = self.serialize_sync(&value)?;
                pipe.add_command(redis::cmd("SET").arg(&prefixed_key).arg(serialized));
                if let Some(ms) = ttl_ms {
                    pipe.add_command(redis::cmd("PEXPIRE").arg(&prefixed_key).arg(ms));
                }
            }

            self.connection_manager
                .execute_command(&routing_key, "PIPELINE_SET_MANY", |mut conn| async move {
                    pipe.query_async(&mut conn).await
                })
                .await?;
        }
        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let deleted_count: i32 = self
            .connection_manager
            .execute_command(&prefixed_key, "DEL", |mut conn| async move {
                conn.del(&prefixed_key).await
            })
            .await?;
        Ok(deleted_count > 0)
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        if keys.is_empty() {
            return Ok(0);
        }
        let prefixed_keys: Vec<String> = keys
            .iter()
            .map(|k| self.connection_manager.prefixed_key(&k.to_string()))
            .collect();

        let routing_key = &prefixed_keys[0];

        self.connection_manager
            .execute_command(routing_key, "DEL_MANY", |mut conn| async move {
                conn.del(prefixed_keys).await
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        self.connection_manager
            .execute_command(&prefixed_key, "EXISTS", |mut conn| async move {
                conn.exists(&prefixed_key).await
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        self.connection_manager
            .execute_command(&prefixed_key, "INCRBY", |mut conn| async move {
                conn.incr(&prefixed_key, amount).await
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let ttl_ms = ttl.as_millis() as usize;

        self.connection_manager
            .execute_command(&prefixed_key, "PEXPIRE", |mut conn| async move {
                conn.pexpire(&prefixed_key, ttl_ms).await
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn clear(&self) -> CacheResult<()> {
        warn!("clear() called on RedisCache. This might be a dangerous operation.");
        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn health_check(&self) -> CacheResult<()> {
        let dummy_key = "__health_check__";
        self.connection_manager
            .execute_command(dummy_key, "PING", |mut conn| async move {
                redis::cmd("PING").query_async(&mut conn).await
            })
            .await
            .map(|_: String| ())
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;
        self.connection_manager
            .execute_command(&prefixed_key, "RPUSH", |mut conn| async move {
                conn.rpush(&prefixed_key, serialized).await
            })
            .await
    }

    #[instrument(skip(self, values), level = "debug")]
    async fn list_push_right_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if values.is_empty() {
            return Ok(0);
        }
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized_values = values
            .iter()
            .map(|v| self.serialize_sync(v))
            .collect::<CacheResult<Vec<Vec<u8>>>>()?;
        self.connection_manager
            .execute_command(&prefixed_key, "RPUSH_MANY", |mut conn| async move {
                conn.rpush(&prefixed_key, serialized_values).await
            })
            .await
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn list_push_left<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;
        self.connection_manager
            .execute_command(&prefixed_key, "LPUSH", |mut conn| async move {
                conn.lpush(&prefixed_key, serialized).await
            })
            .await
    }

    #[instrument(skip(self, values), level = "debug")]
    async fn list_push_left_many<K, V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if values.is_empty() {
            return Ok(0);
        }
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized_values = values
            .iter()
            .map(|v| self.serialize_sync(v))
            .collect::<CacheResult<Vec<Vec<u8>>>>()?;
        self.connection_manager
            .execute_command(&prefixed_key, "LPUSH_MANY", |mut conn| async move {
                conn.lpush(&prefixed_key, serialized_values).await
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_pop_right<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let result: Option<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "RPOP", |mut conn| async move {
                conn.rpop(&prefixed_key, 1).await
            })
            .await?;
        match result {
            Some(bytes) => self.deserialize(&bytes).await.map(Some),
            None => Ok(None),
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let result: Option<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "LPOP", |mut conn| async move {
                conn.lpop(&prefixed_key, 1).await
            })
            .await?;
        match result {
            Some(bytes) => self.deserialize(&bytes).await.map(Some),
            None => Ok(None),
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let results_bytes: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "LRANGE", |mut conn| async move {
                conn.lrange(&prefixed_key, start, stop).await
            })
            .await?;
        results_bytes
            .iter()
            .map(|bytes| self.deserialize(bytes))
            .collect()
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        self.connection_manager
            .execute_command(&prefixed_key, "LLEN", |mut conn| async move {
                conn.llen(&prefixed_key).await
            })
            .await
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn list_remove<K, V>(&self, key: K, count: isize, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;
        self.connection_manager
            .execute_command(&prefixed_key, "LREM", |mut conn| async move {
                conn.lrem(&prefixed_key, count, serialized).await
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_trim<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        self.connection_manager
            .execute_command(&prefixed_key, "LTRIM", |mut conn| async move {
                conn.ltrim(&prefixed_key, start, stop).await
            })
            .await
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn list_set<K, V>(&self, key: K, index: isize, value: &V) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;
        self.connection_manager
            .execute_command(&prefixed_key, "LSET", |mut conn| async move {
                conn.lset(&prefixed_key, index, serialized).await
            })
            .await
    }

    #[instrument(skip(self, field), level = "debug")]
    async fn hash_get<K, F, V>(&self, key: &K, field: &F) -> CacheResult<Option<V>>
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

    #[instrument(skip(self, field, value), level = "debug")]
    async fn hash_set<K, F, V>(&self, key: K, field: F, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let field_str = self.key_to_string(&field)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;

        let result: i32 = self
            .connection_manager
            .execute_command(&prefixed_key, "HSET", |mut conn| async move {
                conn.hset(&prefixed_key, &field_str, serialized).await
            })
            .await?;
        Ok(result == 1)
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

        let results_bytes: Vec<Option<Vec<u8>>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HMGET", |mut conn| async move {
                conn.hget(&prefixed_key, field_strs).await
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
        if entries.is_empty() {
            return Ok(());
        }
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let serialized_entries: Vec<(String, Vec<u8>)> = entries
            .into_iter()
            .map(|(f, v)| Ok((f.to_string(), self.serialize_sync(&v)?)))
            .collect::<CacheResult<Vec<(String, Vec<u8>)>>>()?;

        self.connection_manager
            .execute_command(&prefixed_key, "HMSET", |mut conn| async move {
                conn.hset_multiple(&prefixed_key, &serialized_entries).await
            })
            .await
    }

    #[instrument(skip(self, field), level = "debug")]
    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        let key_str = self.key_to_string(&key)?;
        let field_str = self.key_to_string(&field)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        self.connection_manager
            .execute_command(&prefixed_key, "HEXISTS", |mut conn| async move {
                conn.hexists(&prefixed_key, &field_str).await
            })
            .await
    }

    #[instrument(skip(self, fields), level = "debug")]
    async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        F: CacheKey + 'static + std::fmt::Debug,
    {
        if fields.is_empty() {
            return Ok(0);
        }
        let key_str = self.key_to_string(&key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let field_strs: Vec<String> = fields
            .into_iter()
            .map(|f| self.key_to_string(&f)?)
            .collect();

        self.connection_manager
            .execute_command(&prefixed_key, "HDEL", |mut conn| async move {
                conn.hdel(&prefixed_key, field_strs).await
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_get_all<K, V>(&self, key: &K) -> CacheResult<HashMap<String, V>>
    where
        K: CacheKey + Sync + Send + 'static,
        V: DeserializeOwned + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result_map: HashMap<String, Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HGETALL", |mut conn| async move {
                redis::cmd("HGETALL")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        let mut deserialized_map = HashMap::new();
        for (field, bytes) in result_map {
            let value = self.deserialize(&bytes).await?;
            deserialized_map.insert(field, value);
        }
        Ok(deserialized_map)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_keys<K>(&self, key: &K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result_keys: Vec<String> = self
            .connection_manager
            .execute_command(&prefixed_key, "HKEYS", |mut conn| async move {
                redis::cmd("HKEYS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
                    .await
            })
            .await?;
        Ok(result_keys)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_values<K, V>(&self, key: &K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + Sync + Send + 'static,
        V: DeserializeOwned + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result_values: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HVALS", |mut conn| async move {
                redis::cmd("HVALS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        use futures::stream::{self, StreamExt};
        let values = stream::iter(result_values)
            .then(|bytes| async move { self.deserialize(&bytes).await })
            .collect::<Vec<CacheResult<V>>>()
            .await;

        values.into_iter().collect::<CacheResult<Vec<V>>>()
    }

    #[instrument(skip(self, field), level = "debug")]
    async fn hash_increment<K, F>(&self, key: &K, field: &F, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + Sync + Send + 'static,
        F: Serialize + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let field_str = self.key_to_string(field)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let new_value: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "HINCRBY", |mut conn| async move {
                redis::cmd("HINCRBY")
                    .arg(&prefixed_key)
                    .arg(&field_str)
                    .arg(amount)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        Ok(new_value)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_length<K>(&self, key: &K) -> CacheResult<usize>
    where
        K: CacheKey + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let length: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "HLEN", |mut conn| async move {
                redis::cmd("HLEN")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        Ok(length)
    }

    // Set Operations

    #[instrument(skip(self, values), level = "debug")]
    async fn set_add<K, V>(&self, key: &K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if values.is_empty() {
            return Ok(0);
        }
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized_values = values
            .iter()
            .map(|v| self.serialize_sync(v))
            .collect::<CacheResult<Vec<Vec<u8>>>>()?;

        self.connection_manager
            .execute_command(&prefixed_key, "SADD", |mut conn| async move {
                conn.sadd(&prefixed_key, serialized_values).await
            })
            .await
    }

    #[instrument(skip(self, values), level = "debug")]
    async fn set_remove<K, V>(&self, key: &K, values: &[V]) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if values.is_empty() {
            return Ok(0);
        }
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized_values = values
            .iter()
            .map(|v| self.serialize_sync(v))
            .collect::<CacheResult<Vec<Vec<u8>>>>()?;

        self.connection_manager
            .execute_command(&prefixed_key, "SREM", |mut conn| async move {
                conn.srem(&prefixed_key, serialized_values).await
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn set_members<K, V>(&self, key: &K) -> CacheResult<HashSet<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + Eq + Hash + Sync + Send + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let members_bytes: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "SMEMBERS", |mut conn| async move {
                redis::cmd("SMEMBERS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
                    .await
            })
            .await?;

        let mut members = HashSet::new();
        for bytes in members_bytes {
            let member = self.deserialize(&bytes).await?;
            members.insert(member);
        }
        Ok(members)
    }

    #[instrument(skip(self), level = "debug")]
    async fn set_length<K>(&self, key: &K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = self.key_to_string(key)?;
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        self.connection_manager
            .execute_command(&prefixed_key, "SCARD", |mut conn| async move {
                conn.scard(&prefixed_key).await
            })
            .await
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

    async fn zset_add<K, V>(&self, _key: K, _members: Vec<(f64, V)>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        todo!("zset_add not implemented for RedisCache")
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
        _amount: f64,
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
}

impl Cache for RedisCache {}
