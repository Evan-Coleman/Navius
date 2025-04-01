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

// Define a simple CacheSerializer trait since it's not in navius_cache
/// Cache serializer trait
pub trait CacheSerializer: Send + Sync {
    /// Serialize a value to bytes
    async fn serialize<T: Serialize + Send + Sync>(&self, value: &T) -> CacheResult<Vec<u8>>;
    /// Deserialize bytes to a value
    async fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> CacheResult<T>;
}

/// JSON serializer for cache values
pub struct JsonSerializer;

impl CacheSerializer for JsonSerializer {
    async fn serialize<T: Serialize + Send + Sync>(&self, value: &T) -> CacheResult<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| CacheError::SerializationError(e.to_string()))
    }

    async fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> CacheResult<T> {
        serde_json::from_slice(bytes).map_err(|e| CacheError::SerializationError(e.to_string()))
    }
}

/// Redis cache implementation
pub struct RedisCache {
    /// Connection manager for Redis
    connection_manager: Arc<RedisConnectionManager>,
    /// Serializer for cache values
    serializer: Box<dyn CacheSerializer + Send + Sync>,
    /// Lua script manager
    lua_manager: Option<Arc<RedisLuaManager>>,
}

impl RedisCache {
    /// Create a new Redis cache with default configuration
    pub fn new(connection_manager: Arc<RedisConnectionManager>) -> Self {
        Self::with_serializer(connection_manager, JsonSerializer)
    }

    /// Create a new Redis cache with custom serializer
    pub fn with_serializer<S: CacheSerializer + Send + Sync + 'static>(
        connection_manager: Arc<RedisConnectionManager>,
        serializer: S,
    ) -> Self {
        // Create the Redis cache instance
        let mut cache = Self {
            connection_manager,
            serializer: Box::new(serializer),
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

    /// Get the serializer
    pub fn serializer(&self) -> &dyn CacheSerializer {
        self.serializer.as_ref()
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
                .serializer
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

        let raw_results: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "LRANGE", |mut conn| {
                redis::cmd("LRANGE")
                    .arg(&prefixed_key)
                    .arg(start)
                    .arg(stop)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut results = Vec::with_capacity(raw_results.len());
        for data in raw_results {
            let value = self.serializer.deserialize(&data).await?;
            results.push(value);
        }

        Ok(results)
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
}

#[async_trait]
impl CacheOperations for RedisCache {
    #[instrument(skip(self), level = "debug")]
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        debug!("Getting value for key: {}", prefixed_key);

        let result: Option<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "GET", |mut conn| {
                redis::cmd("GET").arg(&prefixed_key).query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            Some(data) => {
                debug!("Found value for key: {}", prefixed_key);
                match self.serializer.deserialize(&data).await {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => {
                        error!(
                            "Failed to deserialize value for key {}: {:?}",
                            prefixed_key, e
                        );
                        Err(e)
                    }
                }
            }
            None => {
                debug!("No value found for key: {}", prefixed_key);
                Ok(None)
            }
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
                Some(data) => match self.serializer.deserialize(&data).await {
                    Ok(value) => results.push(Some(value)),
                    Err(_) => results.push(None),
                },
                None => results.push(None),
            }
        }

        Ok(results)
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        debug!("Setting value for key: {}", prefixed_key);

        let serialized = self.serializer.serialize(value).await?;
        let ttl = options.and_then(|opt| opt.ttl);

        match ttl {
            Some(ttl) => self
                .connection_manager
                .execute_command(&prefixed_key, "SETEX", |mut conn| {
                    redis::cmd("SETEX")
                        .arg(&prefixed_key)
                        .arg(ttl.as_secs())
                        .arg(&serialized)
                        .query_async(&mut conn)
                })
                .await
                .map_err(|e| CacheError::from(e)),
            None => self
                .connection_manager
                .execute_command(&prefixed_key, "SET", |mut conn| {
                    redis::cmd("SET")
                        .arg(&prefixed_key)
                        .arg(&serialized)
                        .query_async(&mut conn)
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
            let serialized = self.serializer.serialize(&value).await?;

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
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        debug!("Deleting key: {}", prefixed_key);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "DEL", |mut conn| {
                redis::cmd("DEL").arg(&prefixed_key).query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result > 0)
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
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        debug!("Checking if key exists: {}", prefixed_key);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "EXISTS", |mut conn| {
                redis::cmd("EXISTS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result > 0)
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
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        debug!("Setting TTL for key: {} to {:?}", prefixed_key, ttl);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "EXPIRE", |mut conn| {
                redis::cmd("EXPIRE")
                    .arg(&prefixed_key)
                    .arg(ttl.as_secs())
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result > 0)
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

    // List Operations

    #[instrument(skip(self, value), level = "debug")]
    async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serializer.serialize(value).await?;

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
            let serialized = self.serializer.serialize(value).await?;
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
        let serialized = self.serializer.serialize(value).await?;

        let len: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "LPUSH", |mut conn| {
                redis::cmd("LPUSH")
                    .arg(&prefixed_key)
                    .arg(serialized)
                    .query_async(&mut conn)
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
            let serialized = self.serializer.serialize(value).await?;
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
                redis::cmd("RPOP").arg(&prefixed_key).query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            Some(data) => match self.serializer.deserialize(&data).await {
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
                redis::cmd("LPOP").arg(&prefixed_key).query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            Some(data) => match self.serializer.deserialize(&data).await {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
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
                redis::cmd("LLEN").arg(&prefixed_key).query_async(&mut conn)
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
        let serialized = self.serializer.serialize(value).await?;

        let removed: isize = self
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

        Ok(removed as usize)
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
            .map_err(|e| CacheError::from(e))?;

        Ok(())
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn list_set<K, V>(&self, key: K, index: isize, value: &V) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serializer.serialize(value).await?;

        self.connection_manager
            .execute_command(&prefixed_key, "LSET", |mut conn| {
                redis::cmd("LSET")
                    .arg(&prefixed_key)
                    .arg(index)
                    .arg(serialized)
                    .query_async::<_, ()>(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(())
    }

    // Hash Map Operations

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
            Some(data) => match self.serializer.deserialize(&data).await {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn hash_set<K, F, V>(&self, key: K, field: F, value: &V) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serializer.serialize(value).await?;

        let result: i32 = self
            .connection_manager
            .execute_command(&prefixed_key, "HSET", |mut conn| {
                redis::cmd("HSET")
                    .arg(&prefixed_key)
                    .arg(&field_str)
                    .arg(serialized)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result > 0)
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

        // Execute HMGET command
        let mut cmd = redis::cmd("HMGET");
        cmd.arg(&prefixed_key);
        for field in &field_strings {
            cmd.arg(field);
        }

        let raw_results: Vec<Option<Vec<u8>>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HMGET", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut results = Vec::with_capacity(raw_results.len());
        for raw in raw_results {
            match raw {
                Some(data) => match self.serializer.deserialize(&data).await {
                    Ok(value) => results.push(Some(value)),
                    Err(_) => results.push(None),
                },
                None => results.push(None),
            }
        }

        Ok(results)
    }

    #[instrument(skip(self, entries), level = "debug")]
    async fn hash_set_many<K, F, V>(&self, key: K, entries: Vec<(F, V)>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if entries.is_empty() {
            return Ok(());
        }

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Build the HSET command with all fields and values
        let mut cmd = redis::cmd("HSET");
        cmd.arg(&prefixed_key);

        for (field, value) in entries {
            let field_str = field.to_string();
            let serialized = self.serializer.serialize(&value).await?;
            cmd.arg(field_str).arg(serialized);
        }

        self.connection_manager
            .execute_command(&prefixed_key, "HSET", |mut conn| {
                cmd.query_async::<_, ()>(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let exists: i32 = self
            .connection_manager
            .execute_command(&prefixed_key, "HEXISTS", |mut conn| {
                redis::cmd("HEXISTS")
                    .arg(&prefixed_key)
                    .arg(&field_str)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(exists > 0)
    }

    #[instrument(skip(self, fields), level = "debug")]
    async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        F: CacheKey + 'static,
    {
        if fields.is_empty() {
            return Ok(0);
        }

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Convert fields to strings
        let field_strings: Vec<String> = fields.into_iter().map(|f| f.to_string()).collect();

        // Build HDEL command
        let mut cmd = redis::cmd("HDEL");
        cmd.arg(&prefixed_key);
        for field in &field_strings {
            cmd.arg(field);
        }

        let count: i32 = self
            .connection_manager
            .execute_command(&prefixed_key, "HDEL", |mut conn| cmd.query_async(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count as usize)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Get all fields and values as alternating strings and binary data
        let raw_results: Vec<(String, Vec<u8>)> = self
            .connection_manager
            .execute_command(&prefixed_key, "HGETALL", |mut conn| {
                redis::cmd("HGETALL")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut results = Vec::with_capacity(raw_results.len());
        for (field, data) in raw_results {
            match self.serializer.deserialize(&data).await {
                Ok(value) => results.push((field, value)),
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_keys<K>(&self, key: K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let keys: Vec<String> = self
            .connection_manager
            .execute_command(&prefixed_key, "HKEYS", |mut conn| {
                redis::cmd("HKEYS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(keys)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_values<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let raw_values: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HVALS", |mut conn| {
                redis::cmd("HVALS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut values = Vec::with_capacity(raw_values.len());
        for data in raw_values {
            let value = self.serializer.deserialize(&data).await?;
            values.push(value);
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

        let len: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "HLEN", |mut conn| {
                redis::cmd("HLEN").arg(&prefixed_key).query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(len)
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

        // Build SADD command
        let mut cmd = redis::cmd("SADD");
        cmd.arg(&prefixed_key);

        for value in values {
            let serialized = self.serializer.serialize(&value).await?;
            cmd.arg(serialized);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "SADD", |mut conn| cmd.query_async(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self, values), level = "debug")]
    async fn set_remove<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if values.is_empty() {
            return Ok(0);
        }

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Build SREM command
        let mut cmd = redis::cmd("SREM");
        cmd.arg(&prefixed_key);

        for value in values {
            let serialized = self.serializer.serialize(&value).await?;
            cmd.arg(serialized);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "SREM", |mut conn| cmd.query_async(&mut conn))
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
        let serialized = self.serializer.serialize(value).await?;

        let is_member: bool = self
            .connection_manager
            .execute_command(&prefixed_key, "SISMEMBER", |mut conn| {
                redis::cmd("SISMEMBER")
                    .arg(&prefixed_key)
                    .arg(serialized)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(is_member)
    }

    #[instrument(skip(self), level = "debug")]
    async fn set_members<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let raw_members: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "SMEMBERS", |mut conn| {
                redis::cmd("SMEMBERS")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut members = Vec::with_capacity(raw_members.len());
        for data in raw_members {
            let value = self.serializer.deserialize(&data).await?;
            members.push(value);
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

        let len: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "SCARD", |mut conn| {
                redis::cmd("SCARD")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(len)
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn set_intersection<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        // Convert keys to prefixed strings
        let prefixed_keys: Vec<String> = keys
            .into_iter()
            .map(|k| self.connection_manager.prefixed_key(&k.to_string()))
            .collect();

        // Build SINTER command
        let mut cmd = redis::cmd("SINTER");
        for key in &prefixed_keys {
            cmd.arg(key);
        }

        let raw_results: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_keys[0], "SINTER", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut results = Vec::with_capacity(raw_results.len());
        for data in raw_results {
            let value = self.serializer.deserialize(&data).await?;
            results.push(value);
        }

        Ok(results)
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn set_intersection_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        if keys.is_empty() {
            return Ok(0);
        }

        let dest_str = destination.to_string();
        let prefixed_dest = self.connection_manager.prefixed_key(&dest_str);

        // Convert keys to prefixed strings
        let prefixed_keys: Vec<String> = keys
            .into_iter()
            .map(|k| self.connection_manager.prefixed_key(&k.to_string()))
            .collect();

        // Build SINTERSTORE command
        let mut cmd = redis::cmd("SINTERSTORE");
        cmd.arg(&prefixed_dest);
        for key in &prefixed_keys {
            cmd.arg(key);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_dest, "SINTERSTORE", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn set_union<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        // Convert keys to prefixed strings
        let prefixed_keys: Vec<String> = keys
            .into_iter()
            .map(|k| self.connection_manager.prefixed_key(&k.to_string()))
            .collect();

        // Build SUNION command
        let mut cmd = redis::cmd("SUNION");
        for key in &prefixed_keys {
            cmd.arg(key);
        }

        let raw_results: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_keys[0], "SUNION", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut results = Vec::with_capacity(raw_results.len());
        for data in raw_results {
            let value = self.serializer.deserialize(&data).await?;
            results.push(value);
        }

        Ok(results)
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn set_union_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        if keys.is_empty() {
            return Ok(0);
        }

        let dest_str = destination.to_string();
        let prefixed_dest = self.connection_manager.prefixed_key(&dest_str);

        // Convert keys to prefixed strings
        let prefixed_keys: Vec<String> = keys
            .into_iter()
            .map(|k| self.connection_manager.prefixed_key(&k.to_string()))
            .collect();

        // Build SUNIONSTORE command
        let mut cmd = redis::cmd("SUNIONSTORE");
        cmd.arg(&prefixed_dest);
        for key in &prefixed_keys {
            cmd.arg(key);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_dest, "SUNIONSTORE", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn set_difference<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        // Convert keys to prefixed strings
        let prefixed_keys: Vec<String> = keys
            .into_iter()
            .map(|k| self.connection_manager.prefixed_key(&k.to_string()))
            .collect();

        // Build SDIFF command
        let mut cmd = redis::cmd("SDIFF");
        for key in &prefixed_keys {
            cmd.arg(key);
        }

        let raw_results: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_keys[0], "SDIFF", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut results = Vec::with_capacity(raw_results.len());
        for data in raw_results {
            let value = self.serializer.deserialize(&data).await?;
            results.push(value);
        }

        Ok(results)
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn set_difference_store<K, D>(&self, destination: D, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        D: CacheKey + 'static,
    {
        if keys.is_empty() {
            return Ok(0);
        }

        let dest_str = destination.to_string();
        let prefixed_dest = self.connection_manager.prefixed_key(&dest_str);

        // Convert keys to prefixed strings
        let prefixed_keys: Vec<String> = keys
            .into_iter()
            .map(|k| self.connection_manager.prefixed_key(&k.to_string()))
            .collect();

        // Build SDIFFSTORE command
        let mut cmd = redis::cmd("SDIFFSTORE");
        cmd.arg(&prefixed_dest);
        for key in &prefixed_keys {
            cmd.arg(key);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_dest, "SDIFFSTORE", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self), level = "debug")]
    async fn set_random_members<K, V>(&self, key: K, count: usize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let raw_results: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "SRANDMEMBER", |mut conn| {
                redis::cmd("SRANDMEMBER")
                    .arg(&prefixed_key)
                    .arg(count as isize)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut results = Vec::with_capacity(raw_results.len());
        for data in raw_results {
            let value = self.serializer.deserialize(&data).await?;
            results.push(value);
        }

        Ok(results)
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

        // Build ZADD command
        let mut cmd = redis::cmd("ZADD");
        cmd.arg(&prefixed_key);

        for (score, value) in items {
            let serialized = self.serializer.serialize(&value).await?;
            cmd.arg(score).arg(serialized);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "ZADD", |mut conn| cmd.query_async(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self, members), level = "debug")]
    async fn zset_remove<K, V>(&self, key: K, members: Vec<V>) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if members.is_empty() {
            return Ok(0);
        }

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Build ZREM command
        let mut cmd = redis::cmd("ZREM");
        cmd.arg(&prefixed_key);

        for member in members {
            let serialized = self.serializer.serialize(&member).await?;
            cmd.arg(serialized);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "ZREM", |mut conn| cmd.query_async(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self, member), level = "debug")]
    async fn zset_score<K, V>(&self, key: K, member: &V) -> CacheResult<Option<f64>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serializer.serialize(member).await?;

        let score: Option<f64> = self
            .connection_manager
            .execute_command(&prefixed_key, "ZSCORE", |mut conn| {
                redis::cmd("ZSCORE")
                    .arg(&prefixed_key)
                    .arg(serialized)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(score)
    }

    #[instrument(skip(self, member), level = "debug")]
    async fn zset_increment_score<K, V>(
        &self,
        key: K,
        member: &V,
        increment: f64,
    ) -> CacheResult<f64>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serializer.serialize(member).await?;

        let score: f64 = self
            .connection_manager
            .execute_command(&prefixed_key, "ZINCRBY", |mut conn| {
                redis::cmd("ZINCRBY")
                    .arg(&prefixed_key)
                    .arg(increment)
                    .arg(serialized)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(score)
    }

    #[instrument(skip(self), level = "debug")]
    async fn zset_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let raw_results: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "ZRANGE", |mut conn| {
                redis::cmd("ZRANGE")
                    .arg(&prefixed_key)
                    .arg(start)
                    .arg(stop)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut results = Vec::with_capacity(raw_results.len());
        for data in raw_results {
            let value = self.serializer.deserialize(&data).await?;
            results.push(value);
        }

        Ok(results)
    }

    #[instrument(skip(self), level = "debug")]
    async fn zset_range_with_scores<K, V>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Get results with scores using WITHSCORES
        let raw_results: Vec<(Vec<u8>, f64)> = self
            .connection_manager
            .execute_command(&prefixed_key, "ZRANGE", |mut conn| {
                redis::cmd("ZRANGE")
                    .arg(&prefixed_key)
                    .arg(start)
                    .arg(stop)
                    .arg("WITHSCORES")
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut results = Vec::with_capacity(raw_results.len());
        for (data, score) in raw_results {
            let value = self.serializer.deserialize(&data).await?;
            results.push((value, score));
        }

        Ok(results)
    }

    #[instrument(skip(self), level = "debug")]
    async fn zset_range_by_score<K, V>(&self, key: K, min: f64, max: f64) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let raw_results: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "ZRANGEBYSCORE", |mut conn| {
                redis::cmd("ZRANGEBYSCORE")
                    .arg(&prefixed_key)
                    .arg(min)
                    .arg(max)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut results = Vec::with_capacity(raw_results.len());
        for data in raw_results {
            let value = self.serializer.deserialize(&data).await?;
            results.push(value);
        }

        Ok(results)
    }

    #[instrument(skip(self), level = "debug")]
    async fn zset_range_by_score_with_scores<K, V>(
        &self,
        key: K,
        min: f64,
        max: f64,
    ) -> CacheResult<Vec<(V, f64)>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Get results with scores using WITHSCORES
        let raw_results: Vec<(Vec<u8>, f64)> = self
            .connection_manager
            .execute_command(&prefixed_key, "ZRANGEBYSCORE", |mut conn| {
                redis::cmd("ZRANGEBYSCORE")
                    .arg(&prefixed_key)
                    .arg(min)
                    .arg(max)
                    .arg("WITHSCORES")
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        let mut results = Vec::with_capacity(raw_results.len());
        for (data, score) in raw_results {
            let value = self.serializer.deserialize(&data).await?;
            results.push((value, score));
        }

        Ok(results)
    }

    #[instrument(skip(self, member), level = "debug")]
    async fn zset_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serializer.serialize(member).await?;

        let rank: Option<isize> = self
            .connection_manager
            .execute_command(&prefixed_key, "ZRANK", |mut conn| {
                redis::cmd("ZRANK")
                    .arg(&prefixed_key)
                    .arg(serialized)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(rank.map(|r| r as usize))
    }

    #[instrument(skip(self, member), level = "debug")]
    async fn zset_reverse_rank<K, V>(&self, key: K, member: &V) -> CacheResult<Option<usize>>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);
        let serialized = self.serializer.serialize(member).await?;

        let rank: Option<isize> = self
            .connection_manager
            .execute_command(&prefixed_key, "ZREVRANK", |mut conn| {
                redis::cmd("ZREVRANK")
                    .arg(&prefixed_key)
                    .arg(serialized)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(rank.map(|r| r as usize))
    }

    #[instrument(skip(self), level = "debug")]
    async fn zset_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "ZCARD", |mut conn| {
                redis::cmd("ZCARD")
                    .arg(&prefixed_key)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self), level = "debug")]
    async fn zset_count<K>(&self, key: K, min: f64, max: f64) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "ZCOUNT", |mut conn| {
                redis::cmd("ZCOUNT")
                    .arg(&prefixed_key)
                    .arg(min)
                    .arg(max)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self, keys, weights, aggregate), level = "debug")]
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
        let prefixed_dest = self.connection_manager.prefixed_key(&dest_str);

        // Convert keys to prefixed strings
        let prefixed_keys: Vec<String> = keys
            .into_iter()
            .map(|k| self.connection_manager.prefixed_key(&k.to_string()))
            .collect();

        // Build ZINTERSTORE command
        let mut cmd = redis::cmd("ZINTERSTORE");
        cmd.arg(&prefixed_dest);
        cmd.arg(prefixed_keys.len());

        for key in &prefixed_keys {
            cmd.arg(key);
        }

        // Add WEIGHTS if provided
        if let Some(w) = weights {
            if !w.is_empty() && w.len() == prefixed_keys.len() {
                cmd.arg("WEIGHTS");
                for weight in w {
                    cmd.arg(weight);
                }
            }
        }

        // Add AGGREGATE if provided
        if let Some(agg) = aggregate {
            if !agg.is_empty() && (agg == "SUM" || agg == "MIN" || agg == "MAX") {
                cmd.arg("AGGREGATE");
                cmd.arg(agg);
            }
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_dest, "ZINTERSTORE", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self, keys, weights, aggregate), level = "debug")]
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
        let prefixed_dest = self.connection_manager.prefixed_key(&dest_str);

        // Convert keys to prefixed strings
        let prefixed_keys: Vec<String> = keys
            .into_iter()
            .map(|k| self.connection_manager.prefixed_key(&k.to_string()))
            .collect();

        // Build ZUNIONSTORE command
        let mut cmd = redis::cmd("ZUNIONSTORE");
        cmd.arg(&prefixed_dest);
        cmd.arg(prefixed_keys.len());

        for key in &prefixed_keys {
            cmd.arg(key);
        }

        // Add WEIGHTS if provided
        if let Some(w) = weights {
            if !w.is_empty() && w.len() == prefixed_keys.len() {
                cmd.arg("WEIGHTS");
                for weight in w {
                    cmd.arg(weight);
                }
            }
        }

        // Add AGGREGATE if provided
        if let Some(agg) = aggregate {
            if !agg.is_empty() && (agg == "SUM" || agg == "MIN" || agg == "MAX") {
                cmd.arg("AGGREGATE");
                cmd.arg(agg);
            }
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_dest, "ZUNIONSTORE", |mut conn| {
                cmd.query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }
}

// Implement Cache trait
impl Cache for RedisCache {}
