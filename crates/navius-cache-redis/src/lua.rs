use crate::connection::RedisConnectionManager;
use crate::error::{RedisCacheError, RedisCacheResult};
use crate::metrics;
use async_trait::async_trait;
use navius_cache::error::{CacheError, CacheResult};
use redis::aio::Connection;
use redis::{AsyncCommands, FromRedisValue, RedisError, Script, ScriptInvocation};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, instrument, trace};

// Forward declaration of RedisCache to avoid circular reference
use crate::operations::RedisCache;

/// Redis Lua scripting interface for atomic operations
pub trait RedisLuaScripting: Send + Sync {
    /// Register a new script
    async fn register_script(&self, name: &str, script_body: &str) -> CacheResult<()>;

    /// Execute a script
    async fn execute_script<T: FromRedisValue + Send + Sync>(
        &self,
        name: &str,
        keys: &[&str],
        args: &[&str],
    ) -> CacheResult<T>;

    /// Get a cached value with automatic deserialization
    async fn atomic_get<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        key: &str,
    ) -> CacheResult<Option<T>>;

    /// Set a value with automatic serialization
    async fn atomic_set<T: Serialize + Send + Sync + 'static>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()>;

    /// Update a value atomically
    async fn atomic_update<T: Serialize + DeserializeOwned + Send + Sync + 'static>(
        &self,
        key: &str,
        update_fn: Box<dyn FnOnce(Option<T>) -> T + Send + Sync>,
        ttl: Option<Duration>,
    ) -> CacheResult<T>;

    /// Increment a counter atomically
    async fn atomic_increment(&self, key: &str, amount: i64) -> CacheResult<i64>;

    /// Set a value with TTL if it doesn't exist
    async fn atomic_set_nx<T: Serialize + Send + Sync + 'static>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> CacheResult<bool>;
}

/// Script information for caching
#[derive(Debug, Clone)]
pub struct ScriptInfo {
    /// The script body
    pub script: String,
    /// The script hash
    pub hash: String,
}

/// Redis Lua script manager
#[derive(Debug)]
pub struct RedisLuaManager {
    /// Connection manager
    connection_manager: Arc<RedisConnectionManager>,
    /// Registered scripts
    scripts: RwLock<HashMap<String, Script>>,
}

impl RedisLuaManager {
    /// Create a new Lua script manager
    pub fn new(connection_manager: Arc<RedisConnectionManager>) -> Self {
        Self {
            connection_manager,
            scripts: RwLock::new(HashMap::new()),
        }
    }

    /// Register a script
    #[instrument(skip(self, script), level = "debug")]
    pub async fn register_script(&self, name: &str, script: &str) -> Result<(), RedisCacheError> {
        let script = redis::Script::new(script);
        let mut scripts = self.scripts.write().await;
        scripts.insert(name.to_string(), script);
        Ok(())
    }

    /// Get a prefixed key
    pub fn get_prefixed_key(&self, key: &str) -> String {
        self.connection_manager.prefix_key(key)
    }

    /// Get a script
    pub async fn get_script(&self, name: &str) -> Option<ScriptInfo> {
        let scripts = self.scripts.read().await;
        scripts.get(name).map(|s| ScriptInfo {
            script: s.get_body().to_string(),
            hash: s.get_hash().to_string(),
        })
    }

    /// Check if a script exists
    pub async fn script_exists(&self, script_name: &str) -> bool {
        let scripts = self.scripts.read().await;
        scripts.contains_key(script_name)
    }

    /// Get a script hash
    pub async fn get_script_hash(&self, script_name: &str) -> Option<String> {
        let scripts = self.scripts.read().await;
        scripts.get(script_name).map(|s| s.get_hash().to_string())
    }

    #[instrument(skip(self, script), level = "debug")]
    pub async fn load_script(&self, script: &str) -> Result<Script, RedisError> {
        let mut conn = self.connection_manager.get_connection().await?;
        let script = Script::new(script);
        script
            .prepare_invoke()
            .invoke_async::<_, ()>(&mut *conn)
            .await?;

        let mut scripts = self.scripts.write().await;
        scripts.insert(script.get_name().to_string(), script.clone());

        Ok(script)
    }

    #[instrument(skip(self, keys, args), level = "debug")]
    pub async fn execute_script<T: FromRedisValue>(
        &self,
        name: &str,
        keys: &[&str],
        args: &[&str],
    ) -> Result<T, RedisCacheError> {
        let scripts = self.scripts.read().await;
        let script = scripts.get(name).ok_or_else(|| {
            RedisCacheError::OperationError(format!("Script '{}' not found", name))
        })?;

        let timer = metrics::TimedOperation::new("EVALSHA");
        let result = self
            .connection_manager
            .execute_command(name, "EVALSHA", |mut conn| async move {
                script
                    .key(keys)
                    .arg(args)
                    .invoke_async(&mut conn)
                    .await
                    .map_err(|e| RedisCacheError::OperationError(e.to_string()))
            })
            .await;

        timer.record(&result);
        result
    }

    #[instrument(skip(self, keys, args), level = "debug")]
    pub async fn execute_script_with_deserialization<T: DeserializeOwned>(
        &self,
        script_name: &str,
        keys: &[&str],
        args: &[&str],
    ) -> Result<T, RedisCacheError> {
        let bytes = self
            .execute_script::<Vec<u8>>(script_name, keys, args)
            .await?;

        self.connection_manager.deserialize(&bytes).await
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn clear_scripts(&self) -> Result<(), RedisCacheError> {
        let mut scripts = self.scripts.write().await;
        scripts.clear();
        Ok(())
    }
}

/// Helper function to execute a Lua script with metrics
#[instrument(skip(connection_manager, script), level = "debug")]
pub async fn execute_script_with_metrics<T: FromRedisValue>(
    connection_manager: &RedisConnectionManager,
    script_name: &str,
    script: ScriptInvocation<'_>,
    key: &str,
) -> RedisCacheResult<T> {
    let timer = metrics::TimedOperation::new(metrics::names::SCRIPT_EXECUTE);

    let result = connection_manager
        .execute_command(key, "EVALSHA", |conn| async move {
            script.invoke_async(&mut conn).await
        })
        .await;

    // Record metrics
    timer.record(&result);

    result
}

#[async_trait]
impl RedisLuaScripting for RedisLuaManager {
    #[instrument(skip(self), level = "debug")]
    async fn register_script(&self, name: &str, script_body: &str) -> CacheResult<()> {
        // Register the script with the manager
        self.register_script(name, script_body).await?;

        // Load the script into Redis to validate syntax
        let script = Script::new(script_body);
        let mut conn = self.connection_manager.get_connection().await?;

        // Try to preload the script
        script
            .prepare_invoke()
            .invoke_async(&mut conn)
            .await
            .map_err(|e| CacheError::OperationError(format!("Failed to load script: {}", e)))?;

        debug!("Script '{}' registered and loaded", name);
        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn execute_script<T: FromRedisValue + Send + Sync>(
        &self,
        name: &str,
        keys: &[&str],
        args: &[&str],
    ) -> CacheResult<T> {
        trace!(
            "Executing Lua script '{}' with {} keys and {} args",
            name,
            keys.len(),
            args.len()
        );

        let script_info = match self.get_script(name).await {
            Some(script) => script,
            None => {
                return Err(CacheError::OperationError(format!(
                    "Script '{}' not found",
                    name
                )));
            }
        };

        let script = Script::new(&script_info.script);
        let invocation = script.prepare_invoke().key(keys).arg(args);

        execute_script_with_metrics(
            &self.connection_manager,
            name,
            invocation,
            keys.first().unwrap_or(&"script"),
        )
        .await
        .map_err(|e| CacheError::OperationError(format!("Failed to execute script: {}", e)))
    }

    #[instrument(skip(self), level = "debug")]
    async fn atomic_get<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        key: &str,
    ) -> CacheResult<Option<T>> {
        let script_name = "atomic_get";
        trace!("Executing atomic get for key: {}", key);

        // Set up the script if not already registered
        if !self.script_exists(script_name).await {
            let script = r#"
            local value = redis.call('GET', KEYS[1])
            if not value then
                return nil
            end
            return value
            "#;

            self.register_script(script_name, script).await?;
        }

        let prefixed_key = self.get_prefixed_key(key);

        let result: Option<Vec<u8>> = self
            .execute_script(script_name, &[&prefixed_key], &[])
            .await?;

        match result {
            Some(val) => {
                // Deserialize the value
                self.connection_manager
                    .deserialize(&val)
                    .await
                    .map(Some)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn atomic_set<T: Serialize + Send + Sync + 'static>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let script_name = "atomic_set";
        trace!("Executing atomic set for key: {}", key);

        // Set up the script if not already registered
        if !self.script_exists(script_name).await {
            let script = r#"
            if ARGV[2] ~= '' then
                redis.call('SETEX', KEYS[1], ARGV[2], ARGV[1])
            else
                redis.call('SET', KEYS[1], ARGV[1])
            end
            return 1
            "#;

            self.register_script(script_name, script).await?;
        }

        let prefixed_key = self.get_prefixed_key(key);
        let ttl_seconds = ttl.map(|t| t.as_secs().to_string()).unwrap_or_default();

        // Serialize the value
        let serialized = self.connection_manager.serialize(value).await?;

        let _: i32 = self
            .execute_script(
                script_name,
                &[&prefixed_key],
                &[
                    std::str::from_utf8(&serialized)
                        .map_err(|e| CacheError::SerializationError(e.to_string()))?,
                    &ttl_seconds,
                ],
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, update_fn), level = "debug")]
    async fn atomic_update<T: Serialize + DeserializeOwned + Send + Sync + 'static>(
        &self,
        key: &str,
        update_fn: Box<dyn FnOnce(Option<T>) -> T + Send + Sync>,
        ttl: Option<Duration>,
    ) -> CacheResult<T> {
        trace!("Executing atomic update for key: {}", key);

        // First, get the current value
        let current_value: Option<T> = self.atomic_get(key).await?;

        // Apply the update function
        let new_value = update_fn(current_value);

        // Set the new value
        let script_name = "atomic_update";

        // Set up the script if not already registered
        if !self.script_exists(script_name).await {
            let script = r#"
            if ARGV[2] ~= '' then
                redis.call('SETEX', KEYS[1], ARGV[2], ARGV[1])
            else
                redis.call('SET', KEYS[1], ARGV[1])
            end
            return ARGV[1]
            "#;

            self.register_script(script_name, script).await?;
        }

        let prefixed_key = self.get_prefixed_key(key);
        let ttl_seconds = ttl.map(|t| t.as_secs().to_string()).unwrap_or_default();

        // Serialize the value
        let serialized = self.connection_manager.serialize(&new_value).await?;

        let _: Vec<u8> = self
            .execute_script(
                script_name,
                &[&prefixed_key],
                &[
                    std::str::from_utf8(&serialized)
                        .map_err(|e| CacheError::SerializationError(e.to_string()))?,
                    &ttl_seconds,
                ],
            )
            .await?;

        Ok(new_value)
    }

    #[instrument(skip(self), level = "debug")]
    async fn atomic_increment(&self, key: &str, amount: i64) -> CacheResult<i64> {
        let script_name = "atomic_increment";
        trace!("Executing atomic increment for key: {} by {}", key, amount);

        // Set up the script if not already registered
        if !self.script_exists(script_name).await {
            let script = r#"
            return redis.call('INCRBY', KEYS[1], ARGV[1])
            "#;

            self.register_script(script_name, script).await?;
        }

        let prefixed_key = self.get_prefixed_key(key);

        let result: i64 = self
            .execute_script(script_name, &[&prefixed_key], &[&amount.to_string()])
            .await?;

        Ok(result)
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn atomic_set_nx<T: Serialize + Send + Sync + 'static>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> CacheResult<bool> {
        let script_name = "atomic_set_nx";
        trace!("Executing atomic set_nx for key: {}", key);

        // Set up the script if not already registered
        if !self.script_exists(script_name).await {
            let script = r#"
            local result = redis.call('SETNX', KEYS[1], ARGV[1])
            if result == 1 then
                redis.call('EXPIRE', KEYS[1], ARGV[2])
                return 1
            end
            return 0
            "#;

            self.register_script(script_name, script).await?;
        }

        let prefixed_key = self.get_prefixed_key(key);
        let ttl_seconds = ttl.as_secs().to_string();

        // Serialize the value
        let serialized = self.connection_manager.serialize(value).await?;

        let result: i32 = self
            .execute_script(
                script_name,
                &[&prefixed_key],
                &[
                    std::str::from_utf8(&serialized)
                        .map_err(|e| CacheError::SerializationError(e.to_string()))?,
                    &ttl_seconds,
                ],
            )
            .await?;

        Ok(result == 1)
    }
}

/// Initialize Redis with common Lua scripts
pub async fn initialize_common_scripts(cache: &RedisCache) -> CacheResult<()> {
    if let Some(lua_manager) = cache.lua_manager() {
        // Register script for atomic check and increment
        cache
            .register_script(
                "check_and_increment",
                r#"
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
                "#,
            )
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        // Register script for atomic SETNX with TTL
        cache
            .register_script(
                "set_if_not_exists",
                r#"
                local result = redis.call('SETNX', KEYS[1], ARGV[1])
                if result == 1 and ARGV[2] ~= '' then
                    redis.call('EXPIRE', KEYS[1], ARGV[2])
                end
                return result
                "#,
            )
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        // Register script for atomic hash update
        cache
            .register_script(
                "update_hash_if_equals",
                r#"
                local current = redis.call('HGET', KEYS[1], ARGV[1])
                if current == ARGV[2] then
                    redis.call('HSET', KEYS[1], ARGV[1], ARGV[3])
                    return 1
                else
                    return 0
                end
                "#,
            )
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        // Register script for atomic increment and expire
        cache
            .register_script(
                "increment_and_expire",
                r#"
                local count = redis.call('INCRBY', KEYS[1], ARGV[1])
                redis.call('EXPIRE', KEYS[1], ARGV[2])
                return count
                "#,
            )
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        Ok(())
    } else {
        Err(CacheError::OperationError(
            "Lua scripting not enabled".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisCacheConfig;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestUser {
        id: u64,
        name: String,
    }

    #[tokio::test]
    async fn test_lua_scripting() {
        // Create configuration
        let config = RedisCacheConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            key_prefix: Some("test:lua:".to_string()),
            command_timeout_seconds: 2,
            ..Default::default()
        };

        // Create connection manager and cache
        let connection_manager = match RedisConnectionManager::new(
            &config.url,
            config.key_prefix.clone(),
            Duration::from_secs(config.command_timeout_seconds),
        ) {
            Ok(manager) => manager,
            Err(_) => {
                println!("Skipping test_lua_scripting - Redis not available");
                return;
            }
        };

        let cache =
            RedisCache::new(Arc::new(connection_manager)).expect("Failed to create Redis cache");

        // Test script registration and execution
        let script_name = "test_script";
        let script_body = "return {KEYS[1], ARGV[1], ARGV[2]}";

        cache
            .register_script(script_name, script_body)
            .await
            .expect("Failed to register script");

        let result: Vec<String> = cache
            .execute_script(script_name, &["key1"], &["arg1", "arg2"])
            .await
            .expect("Failed to execute script");

        assert_eq!(result.len(), 3);
        assert!(result[0].contains("key1"));
        assert_eq!(result[1], "arg1");
        assert_eq!(result[2], "arg2");

        // Clean up
        cache.flush().await.expect("Failed to flush cache");
    }
}
