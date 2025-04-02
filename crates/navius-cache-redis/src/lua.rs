use crate::connection::RedisConnectionManager;
use crate::error::{RedisCacheError, RedisCacheResult};
use crate::metrics;
use async_trait::async_trait;
use navius_cache::error::{CacheError, CacheResult};
use redis::{AsyncCommands, FromRedisValue, RedisError, Script, ScriptInvocation};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, error, instrument, trace, warn};

// Forward declaration of RedisCache to avoid circular reference
use crate::operations::RedisCache;

/// Redis Lua scripting trait
#[async_trait]
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
    async fn atomic_get<T: DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
    ) -> CacheResult<Option<T>>;

    /// Set a value with automatic serialization
    async fn atomic_set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()>;

    /// Update a value atomically
    async fn atomic_update<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
        update_fn: Box<dyn FnOnce(Option<T>) -> T + Send + Sync>,
        ttl: Option<Duration>,
    ) -> CacheResult<T>;

    /// Increment a counter atomically
    async fn atomic_increment(&self, key: &str, amount: i64) -> CacheResult<i64>;

    /// Set a value with TTL if it doesn't exist
    async fn atomic_set_nx<T: Serialize + Send + Sync>(
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

/// Trait for executing Lua scripts
#[async_trait::async_trait]
pub trait LuaScript {
    /// Execute a Lua script
    async fn execute_script<T: FromRedisValue + Send + 'static>(
        &self,
        script_index: usize,
        keys: &[&str],
        args: &[&str],
    ) -> Result<T, CacheError>;
}

/// Redis Lua script manager
pub struct RedisLuaManager {
    /// Redis connection manager
    connection_manager: Arc<RedisConnectionManager<String>>,
    /// Lua scripts
    scripts: Arc<RwLock<HashMap<String, Script>>>,
}

impl RedisLuaManager {
    /// Create a new Lua script manager
    pub async fn new(config: RedisCacheConfig) -> Self {
        let connection_manager = Arc::new(RedisConnectionManager::new(config).await.unwrap());
        Self {
            connection_manager,
            scripts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a Lua script
    pub fn add_script(&mut self, script: &str) -> Result<(), RedisCacheError> {
        let script = Script::new(script);
        self.scripts
            .write()
            .unwrap()
            .insert(script.get_name().to_string(), script);
        Ok(())
    }

    /// Execute a Lua script
    #[instrument(skip(self, keys, args))]
    pub async fn execute_script<T: FromRedisValue + Send + 'static>(
        &self,
        script_index: usize,
        keys: &[&str],
        args: &[&str],
    ) -> Result<T, CacheError> {
        let scripts = self.scripts.read().unwrap();
        if script_index >= scripts.len() {
            return Err(RedisCacheError::ScriptError(format!(
                "Invalid script index: {}",
                script_index
            ))
            .into());
        }

        let script = scripts.iter().nth(script_index).unwrap().1;
        debug!(
            "Executing script {} with keys {:?} and args {:?}",
            script_index, keys, args
        );

        match self
            .connection_manager
            .execute_command("script", "EVALSHA", |mut conn| async move {
                script.key(keys).arg(args).invoke_async(&mut conn).await
            })
            .await
        {
            Ok(result) => Ok(result),
            Err(err) => Err(RedisCacheError::ScriptError(format!(
                "Failed to execute script {}: {}",
                script_index, err
            ))
            .into()),
        }
    }
}

/// Execute a Lua script with metrics instrumentation
///
/// This is a helper function to execute a Lua script and record metrics
/// for the execution time and result.
pub async fn execute_script_with_metrics<'a, T: FromRedisValue + std::marker::Send + 'static>(
    connection_manager: &Arc<RedisConnectionManager<redis::Value>>,
    script_name: &str,
    script: &'a Script,
    key: &'a str,
    keys: &'a [&'a str],
    args: &'a [&'a str],
) -> Result<T, RedisCacheError> {
    let timer = metrics::TimedOperation::new(script_name);

    let result = connection_manager
        .execute_command(key, script_name, |mut conn| async move {
            script.invoke_async(&mut conn).await
        })
        .await;

    timer.record(&result);
    result.map_err(|e| e.into())
}

#[async_trait]
impl RedisLuaScripting for RedisLuaManager {
    #[instrument(skip(self), level = "debug")]
    async fn register_script(&self, name: &str, script_body: &str) -> CacheResult<()> {
        // Register the script with the manager
        self.add_script(script_body)?;

        // Load the script into Redis to validate syntax
        let script = Script::new(script_body);
        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(|e| CacheError::OperationError(format!("Failed to get connection: {}", e)))?;

        // Try to preload the script
        script
            .prepare_invoke()
            .invoke_async(&mut conn)
            .await
            .map_err(|e| CacheError::OperationError(format!("Failed to load script: {}", e)))?;

        debug!("Script '{}' registered and loaded", name);
        Ok(())
    }

    /// Execute a registered script by name.
    #[instrument(skip(self, script_name, keys, args), level = "debug")]
    pub async fn execute_script<T: FromRedisValue + Send>(
        &self,
        script_name: &str,
        keys: &[&str],
        args: &[&str],
    ) -> RedisCacheResult<T> {
        let script_index = self
            .scripts
            .read()
            .unwrap()
            .iter()
            .position(|(name, _)| name == script_name)
            .ok_or_else(|| {
                RedisCacheError::ScriptError(format!("Script '{}' not registered.", script_name))
            })?;

        // Pass &script instead of script
        execute_script_with_metrics(
            &self.connection_manager,
            script_name,
            self.scripts.read().unwrap().get(script_name).unwrap(),
            keys.first().unwrap_or(&"script"),
            keys,
            args,
        )
        .await
    }

    #[instrument(skip(self), level = "debug")]
    async fn atomic_get<T: DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
    ) -> CacheResult<Option<T>> {
        let script_name = "atomic_get";
        trace!("Executing atomic get for key: {}", key);

        // Set up the script if not already registered
        if self.scripts.read().unwrap().is_empty() {
            let script = r#"
            local value = redis.call('GET', KEYS[1])
            if not value then
                return nil
            end
            return value
            "#;

            self.register_script(script_name, script)?;
        }

        let prefixed_key = self.connection_manager.prefixed_key(key);

        let result: Option<String> = self
            .execute_script(script_name, &[&prefixed_key], &[])
            .await?;

        match result {
            Some(val) => {
                // Deserialize the value
                let bytes = val.as_bytes();
                self.connection_manager
                    .deserialize::<T>(bytes)
                    .map(Some)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn atomic_set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let script_name = "atomic_set";
        trace!("Executing atomic set for key: {}", key);

        // Set up the script if not already registered
        if self.scripts.read().unwrap().is_empty() {
            let script = r#"
            if ARGV[2] ~= '' then
                redis.call('SETEX', KEYS[1], ARGV[2], ARGV[1])
            else
                redis.call('SET', KEYS[1], ARGV[1])
            end
            return 1
            "#;

            self.register_script(script_name, script)?;
        }

        let prefixed_key = self.connection_manager.prefixed_key(key);
        let ttl_seconds = ttl.map(|t| t.as_secs().to_string()).unwrap_or_default();

        // Serialize the value
        let serialized = self.connection_manager.serialize(value).await?;
        let serialized_str = String::from_utf8(serialized)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let _: i32 = self
            .execute_script(
                script_name,
                &[&prefixed_key],
                &[&serialized_str, &ttl_seconds],
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, update_fn), level = "debug")]
    async fn atomic_update<T: Serialize + DeserializeOwned + Send + Sync>(
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
        if self.scripts.read().unwrap().is_empty() {
            let script = r#"
            if ARGV[2] ~= '' then
                redis.call('SETEX', KEYS[1], ARGV[2], ARGV[1])
            else
                redis.call('SET', KEYS[1], ARGV[1])
            end
            return ARGV[1]
            "#;

            self.register_script(script_name, script)?;
        }

        let prefixed_key = self.connection_manager.prefixed_key(key);
        let ttl_seconds = ttl.map(|t| t.as_secs().to_string()).unwrap_or_default();

        // Serialize the value
        let serialized = self.connection_manager.serialize(&new_value).await?;
        let serialized_str = String::from_utf8(serialized)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let _: String = self
            .execute_script(
                script_name,
                &[&prefixed_key],
                &[&serialized_str, &ttl_seconds],
            )
            .await?;

        Ok(new_value)
    }

    #[instrument(skip(self), level = "debug")]
    async fn atomic_increment(&self, key: &str, amount: i64) -> CacheResult<i64> {
        let script_name = "atomic_increment";
        trace!("Executing atomic increment for key: {} by {}", key, amount);

        // Set up the script if not already registered
        if self.scripts.read().unwrap().is_empty() {
            let script = r#"
            return redis.call('INCRBY', KEYS[1], ARGV[1])
            "#;

            self.register_script(script_name, script)?;
        }

        let prefixed_key = self.connection_manager.prefixed_key(key);

        let result: i64 = self
            .execute_script(script_name, &[&prefixed_key], &[&amount.to_string()])
            .await?;

        Ok(result)
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn atomic_set_nx<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> CacheResult<bool> {
        let script_name = "atomic_set_nx";
        trace!("Executing atomic set_nx for key: {}", key);

        // Set up the script if not already registered
        if self.scripts.read().unwrap().is_empty() {
            let script = r#"
            local result = redis.call('SETNX', KEYS[1], ARGV[1])
            if result == 1 then
                redis.call('EXPIRE', KEYS[1], ARGV[2])
                return 1
            end
            return 0
            "#;

            self.register_script(script_name, script)?;
        }

        let prefixed_key = self.connection_manager.prefixed_key(key);
        let ttl_seconds = ttl.as_secs().to_string();

        // Serialize the value
        let serialized = self.connection_manager.serialize(value).await?;
        let serialized_str = String::from_utf8(serialized)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let result: i32 = self
            .execute_script(
                script_name,
                &[&prefixed_key],
                &[&serialized_str, &ttl_seconds],
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
    use crate::connection::RedisConnectionManager;
    use navius_cache::serialization::JsonSerializer;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use std::time::Duration;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestUser {
        id: u64,
        name: String,
    }

    // Helper function to create a mock Redis connection manager for tests
    async fn create_test_manager() -> Arc<RedisConnectionManager<redis::Value>> {
        let config = RedisCacheConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            key_prefix: "test_lua:".to_string(),
            default_ttl: Duration::from_secs(300),
            max_connections: 20,
            min_connections: 5,
            database: 0,
            password: None,
            use_tls: false,
            connection_timeout_seconds: 3,
            command_timeout_seconds: 1,
            idle_timeout_seconds: 30,
            max_lifetime_seconds: 120,
            retry_commands: true,
            max_retries: 3,
            health_check_interval_seconds: 15,
            circuit_breaker_threshold: 3,
            circuit_reset_timeout_seconds: 3,
            enable_metrics: true,
        };

        Arc::new(RedisConnectionManager::new(config).await.unwrap())
    }

    #[tokio::test]
    #[ignore] // Requires running Redis instance
    async fn test_lua_scripting() {
        // Create configuration
        let config = RedisCacheConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            key_prefix: "test:lua:".to_string(),
            default_ttl: Duration::from_secs(300),
            max_connections: 20,
            min_connections: 5,
            database: 0,
            password: None,
            use_tls: false,
            connection_timeout_seconds: 3,
            command_timeout_seconds: 1,
            idle_timeout_seconds: 30,
            max_lifetime_seconds: 120,
            retry_commands: true,
            max_retries: 3,
            health_check_interval_seconds: 15,
            circuit_breaker_threshold: 3,
            circuit_reset_timeout_seconds: 3,
            enable_metrics: true,
        };

        // Create connection manager and cache
        let connection_manager = match RedisConnectionManager::new(config).await {
            Ok(manager) => manager,
            Err(_) => {
                println!("Skipping test_lua_scripting - Redis not available");
                return;
            }
        };

        let cache = RedisCache::with_serializer(connection_manager, JsonSerializer)
            .expect("Failed to create Redis cache");

        // Test execute_raw_script
        let result: i64 = cache
            .execute_raw_script(
                "return tonumber(ARGV[1]) + tonumber(ARGV[2])",
                &[],
                &["5", "7"],
            )
            .await
            .expect("Failed to execute Lua script");
        assert_eq!(result, 12);

        // Test check_and_increment_counter
        let key = "counter:test";
        let max_value = 5;
        let ttl = Some(Duration::from_secs(60));

        // Reset the counter
        cache.delete(key).await.unwrap();

        // Increment counter 5 times
        for i in 0..5 {
            let result = cache
                .check_and_increment_counter(key, max_value, ttl)
                .await
                .expect("Failed to check and increment counter");
            assert!(result, "Counter increment {} failed", i);
        }

        // Sixth increment should fail
        let result = cache
            .check_and_increment_counter(key, max_value, ttl)
            .await
            .expect("Failed to check and increment counter");
        assert!(!result, "Counter should have reached max value");

        // Test set_if_not_exists
        let key = "user:unique";
        cache.delete(key).await.unwrap();

        let user = TestUser {
            id: 1,
            name: "Test User".to_string(),
        };

        // First set should succeed
        let result = cache
            .set_if_not_exists(key, &user, Some(Duration::from_secs(60)))
            .await
            .expect("Failed to set if not exists");
        assert!(result, "First set should succeed");

        // Second set should fail
        let result = cache
            .set_if_not_exists(key, &user, Some(Duration::from_secs(60)))
            .await
            .expect("Failed to set if not exists");
        assert!(!result, "Second set should fail");

        // Clean up
        cache.delete(key).await.unwrap();
        cache.delete("counter:test").await.unwrap();
    }
}
