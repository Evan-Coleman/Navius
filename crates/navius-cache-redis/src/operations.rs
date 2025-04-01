use async_trait::async_trait;
use redis::{AsyncCommands, FromRedisValue};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, instrument};

use navius_cache::{
    error::{CacheError, CacheResult},
    operations::{Cache, CacheKey, CacheOperations, CacheOptions},
};

use crate::{
    connection::RedisConnectionManager,
    error::{RedisCacheError, RedisCacheResult},
    lua::RedisLuaManager,
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
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        debug!("Getting list range for key: {}", prefixed_key);

        let data: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "LRANGE", |mut conn| {
                redis::cmd("LRANGE")
                    .arg(&prefixed_key)
                    .arg(start)
                    .arg(stop)
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut values = Vec::with_capacity(data.len());
        for item in data {
            match self.deserialize(&item).await {
                Ok(value) => values.push(value),
                Err(e) => {
                    error!(
                        "Failed to deserialize value for key {}: {:?}",
                        prefixed_key, e
                    );
                    return Err(e);
                }
            }
        }

        Ok(values)
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
    pub async fn hash_get(&self, key: &str, field: &str) -> RedisCacheResult<Option<Vec<u8>>> {
        let timer = metrics::TimedOperation::new(metrics::names::HASH_GET);
        let result = self
            .connection_manager
            .execute_command(key, "HGET", |mut conn| {
                let res: Option<Vec<u8>> =
                    redis::cmd("HGET").arg(key).arg(field).query(&mut conn)?;
                Ok(res)
            })
            .await;

        timer.record(&result);
        result
    }

    /// Delete a field from a hash
    #[instrument(skip(self), level = "debug")]
    pub async fn hash_delete(&self, key: &str, field: &str) -> RedisCacheResult<bool> {
        let timer = metrics::TimedOperation::new(metrics::names::HASH_DELETE);
        let result = self
            .connection_manager
            .execute_command(key, "HDEL", |mut conn| {
                let res: i32 = redis::cmd("HDEL").arg(key).arg(field).query(&mut conn)?;
                Ok(res > 0)
            })
            .await;

        timer.record(&result);
        result
    }

    #[instrument(skip(self, fields), level = "debug")]
    async fn hash_get_many<K, F, V>(&self, key: K, fields: Vec<F>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        if fields.is_empty() {
            return Ok(Vec::new());
        }

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Convert fields to strings
        let field_strings: Vec<String> = fields.into_iter().map(|f| f.to_string()).collect();

        // Use HMGET to get multiple fields at once
        let mut cmd = redis::cmd("HMGET");
        cmd.arg(&prefixed_key);
        for field in &field_strings {
            cmd.arg(field);
        }

        let results: Vec<Option<Vec<u8>>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HMGET", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        // Deserialize each result
        let mut values = Vec::with_capacity(results.len());
        for raw in results {
            match raw {
                Some(data) => match self.deserialize(&data).await {
                    Ok(value) => values.push(Some(value)),
                    Err(_) => values.push(None),
                },
                None => values.push(None),
            }
        }

        Ok(values)
    }

    #[instrument(skip(self, entries), level = "debug")]
    async fn hash_set_many<K, F, V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<()>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        F: Serialize + 'static + std::fmt::Debug,
        V: Serialize + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let mut cmd = redis::cmd("HMSET");
        cmd.arg(&prefixed_key);

        for (field, value) in entries {
            let field_ser = self.serialize(&field).await?;
            let value_ser = self.serialize(&value).await?;
            cmd.arg(field_ser).arg(value_ser);
        }

        self.connection_manager
            .execute_command(&prefixed_key, "HMSET", |mut conn| cmd.query(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        F: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result: bool = self
            .connection_manager
            .execute_command(&prefixed_key, "HEXISTS", |mut conn| {
                Ok(redis::cmd("HEXISTS")
                    .arg(&prefixed_key)
                    .arg(&field_str)
                    .query_async(&mut conn)
                    .await?)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result)
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

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Convert fields to strings
        let field_strings: Vec<String> = fields.into_iter().map(|f| f.to_string()).collect();

        // Use HDEL to delete multiple fields at once
        let mut cmd = redis::cmd("HDEL");
        cmd.arg(&prefixed_key);
        for field in &field_strings {
            cmd.arg(field);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "HDEL", |mut conn| cmd.query(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Get all fields and values as flattened array of [field1, val1, field2, val2, ...]
        let result: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HGETALL", |mut conn| {
                redis::cmd("HGETALL")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        // Process results into pairs
        let mut pairs = Vec::new();
        let mut iter = result.into_iter();

        while let (Some(field_bytes), Some(value_bytes)) = (iter.next(), iter.next()) {
            let field = String::from_utf8(field_bytes)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            match self.deserialize(&value_bytes).await {
                Ok(value) => pairs.push((field, value)),
                Err(e) => return Err(e),
            }
        }

        Ok(pairs)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_keys<K>(&self, key: K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let keys: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HKEYS", |mut conn| {
                redis::cmd("HKEYS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        // Convert bytes to strings
        let string_keys = keys
            .into_iter()
            .map(|k| {
                String::from_utf8(k).map_err(|e| CacheError::SerializationError(e.to_string()))
            })
            .collect::<Result<Vec<String>, CacheError>>()?;

        Ok(string_keys)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_values<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let values_bytes: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HVALS", |mut conn| {
                redis::cmd("HVALS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        // Deserialize each value
        let mut values = Vec::with_capacity(values_bytes.len());
        for value_bytes in values_bytes {
            match self.deserialize(&value_bytes).await {
                Ok(value) => values.push(value),
                Err(e) => return Err(e),
            }
        }

        Ok(values)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_increment<K, F>(&self, key: K, field: F, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "HINCRBY", |mut conn| {
                redis::cmd("HINCRBY")
                    .arg(&prefixed_key)
                    .arg(&field_str)
                    .arg(amount)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "HLEN", |mut conn| {
                redis::cmd("HLEN").arg(&prefixed_key).query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    // Set Operations

    #[instrument(skip(self, values), level = "debug")]
    async fn set_remove<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        V: Serialize + Send + Sync + 'static + std::fmt::Debug,
    {
        if values.is_empty() {
            return Ok(0);
        }

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let mut cmd = redis::cmd("SREM");
        cmd.arg(&prefixed_key);

        for value in values {
            let serialized = self.serialize(&value).await?;
            cmd.arg(serialized);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "SREM", |mut conn| cmd.query(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;

        let result: bool = self
            .connection_manager
            .execute_command(&prefixed_key, "SISMEMBER", |mut conn| {
                redis::cmd("SISMEMBER")
                    .arg(&prefixed_key)
                    .arg(serialized)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    async fn set_members<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let members_bytes: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "SMEMBERS", |mut conn| {
                redis::cmd("SMEMBERS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        // Deserialize each member
        let mut members = Vec::with_capacity(members_bytes.len());
        for member_bytes in members_bytes {
            match self.deserialize(&member_bytes).await {
                Ok(member) => members.push(member),
                Err(e) => return Err(e),
            }
        }

        Ok(members)
    }

    #[instrument(skip(self), level = "debug")]
    async fn set_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "SCARD", |mut conn| {
                redis::cmd("SCARD")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }
}

pub trait RedisHash<K, F>
where
    K: CacheKey + 'static + std::fmt::Debug,
    F: CacheKey + 'static + std::fmt::Debug + Send + Sync,
{
    // ... existing code ...
}

impl<K, F> RedisHash<K, F> for RedisCache
where
    K: CacheKey + 'static + std::fmt::Debug,
    F: CacheKey + 'static + std::fmt::Debug + Send + Sync,
{
    #[instrument(skip(self, field, value), level = "debug")]
    async fn hset<V>(&self, key: K, field: F, value: V) -> CacheResult<bool>
    where
        V: Serialize + 'static + Send + Sync,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "HSET", |mut conn| {
            let mut cmd = redis::cmd("HSET");
            cmd.arg(&prefixed_key).arg(&field_str).arg(value);
            cmd.query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self, entries), level = "debug")]
    async fn hset_multiple<V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<bool>
    where
        V: Serialize + 'static + Send + Sync,
    {
        if entries.is_empty() {
            return Ok(false);
        }

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "HMSET", |mut conn| {
            let mut cmd = redis::cmd("HMSET");
            cmd.arg(&prefixed_key);

            for (field, value) in entries {
                let field_str = field.to_string();
                cmd.arg(&field_str).arg(value);
            }

            cmd.query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn hgetall<V>(&self, key: K) -> CacheResult<std::collections::HashMap<String, V>>
    where
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "HGETALL", |mut conn| {
            redis::cmd("HGETALL").arg(&prefixed_key).query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn hkeys(&self, key: K) -> CacheResult<Vec<String>> {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "HKEYS", |mut conn| {
            redis::cmd("HKEYS").arg(&prefixed_key).query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn hvals<V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "HVALS", |mut conn| {
            redis::cmd("HVALS").arg(&prefixed_key).query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn hincrby<N>(&self, key: K, field: F, amount: N) -> CacheResult<N>
    where
        N: FromRedisValue + ToRedisArgs + 'static,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "HINCRBY", |mut conn| {
            redis::cmd("HINCRBY")
                .arg(&prefixed_key)
                .arg(&field_str)
                .arg(amount)
                .query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_length(&self, key: K) -> CacheResult<usize> {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "HLEN", |mut conn| {
            redis::cmd("HLEN").arg(&prefixed_key).query(&mut conn)
        })
        .await
    }
}

pub trait RedisSet<K>
where
    K: CacheKey + 'static + std::fmt::Debug,
{
    // ... existing code ...
}

impl<K> RedisSet<K> for RedisCache
where
    K: CacheKey + 'static + std::fmt::Debug,
{
    #[instrument(skip(self, value), level = "debug")]
    async fn sismember<V>(&self, key: K, value: V) -> CacheResult<bool>
    where
        V: Serialize + 'static + Send + Sync,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);
        let serialized = self.serialize(&value).await?;

        self.execute_command(&prefixed_key, "SISMEMBER", |mut conn| {
            redis::cmd("SISMEMBER")
                .arg(&prefixed_key)
                .arg(serialized)
                .query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn smembers<V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "SMEMBERS", |mut conn| {
            redis::cmd("SMEMBERS").arg(&prefixed_key).query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn scard(&self, key: K) -> CacheResult<usize> {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "SCARD", |mut conn| {
            redis::cmd("SCARD").arg(&prefixed_key).query(&mut conn)
        })
        .await
    }
}

pub trait RedisStringOperations<K>
where
    K: CacheKey + 'static + std::fmt::Debug,
{
    // ... existing function signatures ...
}

impl<K> RedisStringOperations<K> for RedisCache
where
    K: CacheKey + 'static + std::fmt::Debug,
{
    #[instrument(skip(self), level = "debug")]
    async fn get<V>(&self, key: K) -> CacheResult<Option<V>>
    where
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "GET", |mut conn| {
            redis::cmd("GET").arg(&prefixed_key).query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self, value, options), level = "debug")]
    async fn set<V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<bool>
    where
        V: Serialize + 'static + Send + Sync,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        let serialized = self.serialize(value).await?;

        self.execute_command(&prefixed_key, "SET", |mut conn| {
            let mut cmd = redis::cmd("SET");
            cmd.arg(&prefixed_key).arg(serialized);

            if let Some(opts) = options {
                if let Some(ttl) = opts.ttl {
                    cmd.arg("PX").arg(ttl.as_millis() as u64);
                }
            }

            cmd.query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn exists(&self, key: K) -> CacheResult<bool> {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "EXISTS", |mut conn| {
            let result: i64 = redis::cmd("EXISTS").arg(&prefixed_key).query(&mut conn)?;
            Ok(result > 0)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn delete(&self, key: K) -> CacheResult<bool> {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "DEL", |mut conn| {
            let result: i64 = redis::cmd("DEL").arg(&prefixed_key).query(&mut conn)?;
            Ok(result > 0)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn expire(&self, key: K, ttl: Duration) -> CacheResult<bool> {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "EXPIRE", |mut conn| {
            let result: i64 = redis::cmd("PEXPIRE")
                .arg(&prefixed_key)
                .arg(ttl.as_millis() as u64)
                .query(&mut conn)?;
            Ok(result > 0)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn ttl(&self, key: K) -> CacheResult<Duration> {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "PTTL", |mut conn| {
            let result: i64 = redis::cmd("PTTL").arg(&prefixed_key).query(&mut conn)?;

            if result < 0 {
                // -1 means no expiry, -2 means key doesn't exist
                return Ok(Duration::from_secs(0));
            }

            Ok(Duration::from_millis(result as u64))
        })
        .await
    }
}

pub trait RedisListOperations<K>
where
    K: CacheKey + 'static + std::fmt::Debug,
{
    // ... existing function signatures ...
}

impl<K> RedisListOperations<K> for RedisCache
where
    K: CacheKey + 'static + std::fmt::Debug,
{
    #[instrument(skip(self, value), level = "debug")]
    async fn lpush<V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        V: Serialize + 'static + Send + Sync,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);
        let serialized = self.serialize(value).await?;

        self.execute_command(&prefixed_key, "LPUSH", |mut conn| {
            redis::cmd("LPUSH")
                .arg(&prefixed_key)
                .arg(serialized)
                .query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self, values), level = "debug")]
    async fn lpush_multiple<V>(&self, key: K, values: &[V]) -> CacheResult<usize>
    where
        V: Serialize + 'static + Send + Sync,
    {
        if values.is_empty() {
            return Ok(0);
        }

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "LPUSH", |mut conn| {
            let mut cmd = redis::cmd("LPUSH");
            cmd.arg(&prefixed_key);

            for value in values {
                let serialized = match serde_json::to_string(value) {
                    Ok(s) => s,
                    Err(e) => return Err(redis::RedisError::from(e)),
                };
                cmd.arg(serialized);
            }

            cmd.query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn lrange<V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        let values: Vec<String> = self
            .execute_command(&prefixed_key, "LRANGE", |mut conn| {
                redis::cmd("LRANGE")
                    .arg(&prefixed_key)
                    .arg(start)
                    .arg(stop)
                    .query(&mut conn)
            })
            .await?;

        let mut result = Vec::with_capacity(values.len());
        for value_str in values {
            match serde_json::from_str(&value_str) {
                Ok(value) => result.push(value),
                Err(e) => return Err(CacheError::SerializationError(e.to_string())),
            }
        }

        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    async fn llen(&self, key: K) -> CacheResult<usize> {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        self.execute_command(&prefixed_key, "LLEN", |mut conn| {
            redis::cmd("LLEN").arg(&prefixed_key).query(&mut conn)
        })
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn rpop<V>(&self, key: K) -> CacheResult<Option<V>>
    where
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefix_key(&key_str);

        let value: Option<String> = self
            .execute_command(&prefixed_key, "RPOP", |mut conn| {
                redis::cmd("RPOP").arg(&prefixed_key).query(&mut conn)
            })
            .await?;

        if let Some(value_str) = value {
            match serde_json::from_str(&value_str) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(CacheError::SerializationError(e.to_string())),
            }
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl CacheOperations for RedisCache {
    #[instrument(skip(self), level = "debug")]
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result: Option<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "GET", |mut conn| {
                redis::cmd("GET").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            Some(data) => match self.deserialize(&data).await {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let mut results = Vec::with_capacity(keys.len());

        // Convert keys to strings and prefix them
        let key_strings: Vec<String> = keys
            .into_iter()
            .map(|k| self.connection_manager.prefixed_key(&k.to_string()))
            .collect();

        // Get multiple values in one pipeline call for efficiency
        let timer = metrics::TimedOperation::new(metrics::names::GET);

        // Use pipeline to get all keys in one request
        let pipe_cmd = key_strings
            .iter()
            .fold(redis::pipe(), |pipe, key| pipe.cmd("GET").arg(key));

        let raw_results: Vec<Option<Vec<u8>>> = self
            .connection_manager
            .execute_pipeline_command("MGET", |mut conn| pipe_cmd.query_async(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        timer.record(&CacheResult::Ok(()));

        // Deserialize each result
        for raw in raw_results {
            match raw {
                Some(data) => match self.deserialize(&data).await {
                    Ok(value) => results.push(Some(value)),
                    Err(_) => results.push(None),
                },
                None => results.push(None),
            }
        }

        Ok(results)
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn set<K, V>(&self, key: K, value: V, options: Option<CacheOptions>) -> CacheResult<()>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(&value).await?;

        match options.and_then(|opt| opt.ttl) {
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

    #[instrument(skip(self, entries, options), level = "debug")]
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

        let ttl = options.and_then(|opt| opt.ttl);

        // Build Redis pipeline
        let mut pipe = redis::pipe();

        for (key, value) in entries {
            let key_str = key.to_string();
            let prefixed_key = self.connection_manager.prefixed_key(&key_str);
            let serialized = self.serialize(&value).await?;

            match ttl {
                Some(ttl) => {
                    pipe.cmd("SETEX")
                        .arg(&prefixed_key)
                        .arg(ttl.as_secs())
                        .arg(&serialized);
                }
                None => {
                    pipe.cmd("SET").arg(&prefixed_key).arg(&serialized);
                }
            }
        }

        // Execute pipeline
        self.connection_manager
            .execute_pipeline_command("MSET", move |mut conn| pipe.query_async(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "DEL", |mut conn| {
                redis::cmd("DEL").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count > 0)
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        if keys.is_empty() {
            return Ok(0);
        }

        // Convert keys to strings and prefix them
        let key_strings: Vec<String> = keys
            .into_iter()
            .map(|k| self.connection_manager.prefixed_key(&k.to_string()))
            .collect();

        // Delete all keys in one command
        let count: i64 = self
            .connection_manager
            .execute_command("", "DEL", |mut conn| {
                let mut cmd = redis::cmd("DEL");
                for key in &key_strings {
                    cmd.arg(key);
                }
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count as usize)
    }

    #[instrument(skip(self), level = "debug")]
    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result: bool = self
            .connection_manager
            .execute_command(&prefixed_key, "EXISTS", |mut conn| {
                redis::cmd("EXISTS").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        debug!("Incrementing key: {} by {}", prefixed_key, amount);

        self.connection_manager
            .execute_command(&prefixed_key, "INCRBY", |mut conn| {
                redis::cmd("INCRBY")
                    .arg(&prefixed_key)
                    .arg(amount)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))
    }

    #[instrument(skip(self), level = "debug")]
    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result: bool = self
            .connection_manager
            .execute_command(&prefixed_key, "EXPIRE", |mut conn| {
                redis::cmd("EXPIRE")
                    .arg(&prefixed_key)
                    .arg(ttl.as_secs())
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    async fn clear(&self) -> CacheResult<()> {
        debug!(
            "Clearing all keys with prefix: {}",
            self.connection_manager.key_prefix()
        );

        // Find all keys with this prefix
        let pattern = format!("{}*", self.connection_manager.key_prefix());

        let keys: Vec<String> = self
            .connection_manager
            .execute_command("", "KEYS", |mut conn| {
                redis::cmd("KEYS").arg(&pattern).query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        if !keys.is_empty() {
            // Delete all found keys
            self.connection_manager
                .execute_command("", "DEL", |mut conn| {
                    let mut cmd = redis::cmd("DEL");
                    for key in &keys {
                        cmd.arg(key);
                    }
                    cmd.query_async::<_, ()>(&mut conn)
                })
                .await
                .map_err(|e| CacheError::from(e))?;
        }

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn health_check(&self) -> CacheResult<()> {
        debug!("Performing health check");

        // Simple ping-pong check
        let result: String = self
            .connection_manager
            .execute_command("", "PING", |mut conn| {
                redis::cmd("PING").query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        if result == "PONG" {
            Ok(())
        } else {
            Err(CacheError::ConnectionError(
                "Redis health check failed".to_string(),
            ))
        }
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;

        let len: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "RPUSH", |mut conn| {
                redis::cmd("RPUSH")
                    .arg(&prefixed_key)
                    .arg(serialized)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(len)
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

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Build command with all values
        let mut cmd = redis::cmd("RPUSH");
        cmd.arg(&prefixed_key);

        for value in values {
            let serialized = self.serialize(value).await?;
            cmd.arg(serialized);
        }

        let len: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "RPUSH", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(len)
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn list_push_left<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;

        let len: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "LPUSH", |mut conn| {
                redis::cmd("LPUSH")
                    .arg(&prefixed_key)
                    .arg(serialized)
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(len)
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

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Build command with all values
        let mut cmd = redis::cmd("LPUSH");
        cmd.arg(&prefixed_key);

        for value in values {
            let serialized = self.serialize(value).await?;
            cmd.arg(serialized);
        }

        let len: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "LPUSH", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(len)
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_pop_right<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result: Option<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "RPOP", |mut conn| {
                redis::cmd("RPOP").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            Some(data) => match self.deserialize(&data).await {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result: Option<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "LPOP", |mut conn| {
                redis::cmd("LPOP").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            Some(data) => match self.deserialize(&data).await {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        debug!("Getting list range for key: {}", prefixed_key);

        let data: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "LRANGE", |mut conn| {
                redis::cmd("LRANGE")
                    .arg(&prefixed_key)
                    .arg(start)
                    .arg(stop)
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut values = Vec::with_capacity(data.len());
        for item in data {
            match self.deserialize(&item).await {
                Ok(value) => values.push(value),
                Err(e) => {
                    error!(
                        "Failed to deserialize value for key {}: {:?}",
                        prefixed_key, e
                    );
                    return Err(e);
                }
            }
        }

        Ok(values)
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let len: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "LLEN", |mut conn| {
                redis::cmd("LLEN").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(len)
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn list_remove<K, V>(&self, key: K, count: isize, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;

        let removed: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "LREM", |mut conn| {
                redis::cmd("LREM")
                    .arg(&prefixed_key)
                    .arg(count)
                    .arg(serialized)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(removed)
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_trim<K>(&self, key: K, start: isize, stop: isize) -> CacheResult<()>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        self.connection_manager
            .execute_command(&prefixed_key, "LTRIM", |mut conn| {
                redis::cmd("LTRIM")
                    .arg(&prefixed_key)
                    .arg(start)
                    .arg(stop)
                    .query_async::<_, ()>(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn list_set<K, V>(&self, key: K, index: isize, value: &V) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serialize(value).await?;

        self.connection_manager
            .execute_command(&prefixed_key, "LSET", |mut conn| {
                redis::cmd("LSET")
                    .arg(&prefixed_key)
                    .arg(index)
                    .arg(serialized)
                    .query_async::<_, ()>(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))
    }

    // Hash Operations

    #[instrument(skip(self), level = "debug")]
    async fn hash_get<K, F, V>(&self, key: K, field: F) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result: Option<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HGET", |mut conn| {
                redis::cmd("HGET")
                    .arg(&prefixed_key)
                    .arg(&field_str)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            Some(data) => match self.deserialize(&data).await {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    }

    // Set Operations

    #[instrument(skip(self, values), level = "debug")]
    async fn set_add<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if values.is_empty() {
            return Ok(0);
        }

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let mut cmd = redis::cmd("SADD");
        cmd.arg(&prefixed_key);

        for value in values {
            let serialized = self.serialize(&value).await?;
            cmd.arg(serialized);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "SADD", |mut conn| cmd.query_async(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    // Sorted Set Operations

    #[instrument(skip(self, items), level = "debug")]
    async fn zset_add<K, V>(&self, key: K, items: Vec<(f64, V)>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if items.is_empty() {
            return Ok(0);
        }

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let mut cmd = redis::cmd("ZADD");
        cmd.arg(&prefixed_key);

        for (score, value) in items {
            let serialized = self.serialize(&value).await?;
            cmd.arg(score).arg(serialized);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "ZADD", |mut conn| cmd.query_async(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    // Additional methods will be implemented in subsequent edits
    // ...
}

impl Cache for RedisCache {}
