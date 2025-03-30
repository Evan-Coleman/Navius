use async_trait::async_trait;
use redis::{AsyncCommands, FromRedisValue, cmd};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, instrument};

use navius_cache::{
    error::{CacheError, CacheResult},
    operations::Cache,
    serialization::{CacheSerializer, JsonSerializer},
};

use crate::{
    config::RedisCacheConfig,
    connection::RedisConnectionManager,
    error::{RedisCacheError, RedisCacheResult},
    lua::{RedisLuaManager, initialize_common_scripts},
    metrics,
};

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
    pub async fn list_range(
        &self,
        key: &str,
        start: isize,
        stop: isize,
    ) -> RedisCacheResult<Vec<Vec<u8>>> {
        let timer = metrics::TimedOperation::new(metrics::names::LIST_RANGE);
        let result = self
            .connection_manager
            .execute_command(key, "LRANGE", |mut conn| {
                let res: Vec<Vec<u8>> = redis::cmd("LRANGE")
                    .arg(key)
                    .arg(start)
                    .arg(stop)
                    .query(&mut conn)?;
                Ok(res)
            })
            .await;

        timer.record(&result);
        result
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
impl Cache for RedisCache {
    #[instrument(skip(self), level = "debug")]
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> CacheResult<Option<T>> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
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

    #[instrument(skip(self, value), level = "debug")]
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
        debug!("Setting value for key: {}", prefixed_key);

        let serialized = self.serializer.serialize(value).await?;

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

    #[instrument(skip(self), level = "debug")]
    async fn delete(&self, key: &str) -> CacheResult<bool> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
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

    #[instrument(skip(self), level = "debug")]
    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
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
    async fn increment(&self, key: &str, amount: i64) -> CacheResult<i64> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
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
    async fn decrement(&self, key: &str, amount: i64) -> CacheResult<i64> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
        debug!("Decrementing key: {} by {}", prefixed_key, amount);

        self.connection_manager
            .execute_command(&prefixed_key, "DECRBY", |mut conn| {
                redis::cmd("DECRBY")
                    .arg(&prefixed_key)
                    .arg(amount)
                    .query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))
    }

    #[instrument(skip(self), level = "debug")]
    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
        debug!("Getting TTL for key: {}", prefixed_key);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "TTL", |mut conn| {
                redis::cmd("TTL").arg(&prefixed_key).query_async(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        match result {
            -2 => Ok(None), // Key does not exist
            -1 => Ok(None), // Key exists but has no TTL
            ttl => Ok(Some(Duration::from_secs(ttl as u64))),
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn set_ttl(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
        let prefixed_key = self.connection_manager.prefixed_key(key);
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
}
