use async_trait::async_trait;
use navius_cache::error::{CacheError, CacheResult};
use redis::{AsyncCommands, Script};
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::{debug, error, instrument};

use crate::{
    connection::RedisConnectionManager,
    error::{RedisCacheError, RedisCacheResult},
    operations::RedisCache,
};

/// Lua scripting support for Redis operations
#[async_trait]
pub trait RedisLuaScripting {
    /// Register a new Lua script with the Redis server
    async fn register_script(&self, name: &str, script_body: &str) -> CacheResult<()>;

    /// Execute a Lua script with the given arguments
    async fn execute_script<T: DeserializeOwned + Send + Sync>(
        &self,
        name: &str,
        keys: &[&str],
        args: &[&str],
    ) -> CacheResult<T>;

    /// Execute a raw Lua script with the given arguments
    async fn execute_raw_script<T: DeserializeOwned + Send + Sync>(
        &self,
        script_body: &str,
        keys: &[&str],
        args: &[&str],
    ) -> CacheResult<T>;

    /// Check and increment a counter atomically with a maximum value
    /// Returns true if the counter was incremented successfully, false if it reached the maximum
    async fn check_and_increment_counter(
        &self,
        key: &str,
        max_value: i64,
        ttl: Option<Duration>,
    ) -> CacheResult<bool>;

    /// Set a value only if the key doesn't exist (atomic SETNX with TTL)
    async fn set_if_not_exists<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<bool>;

    /// Update a hash field only if its current value matches the expected value
    async fn update_hash_if_equals<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        field: &str,
        expected: &str,
        new_value: &T,
    ) -> CacheResult<bool>;

    /// Atomic increment and expire operation
    async fn increment_and_expire(
        &self,
        key: &str,
        increment_by: i64,
        ttl: Duration,
    ) -> CacheResult<i64>;
}

/// Redis Lua script manager
pub struct RedisLuaManager {
    /// Connection manager
    connection_manager: RedisConnectionManager,
    /// Script registry
    scripts: Arc<std::sync::Mutex<HashMap<String, Script>>>,
}

impl RedisLuaManager {
    /// Create a new Lua script manager
    pub fn new(connection_manager: RedisConnectionManager) -> Self {
        Self {
            connection_manager,
            scripts: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Get a script by name
    fn get_script(&self, name: &str) -> Option<Script> {
        let scripts = self.scripts.lock().unwrap();
        scripts.get(name).cloned()
    }

    /// Register a script
    fn register(&self, name: &str, script_body: &str) -> Script {
        let script = Script::new(script_body);
        let mut scripts = self.scripts.lock().unwrap();
        scripts.insert(name.to_string(), script.clone());
        script
    }
}

#[async_trait]
impl RedisLuaScripting for RedisCache {
    #[instrument(skip(self), level = "debug")]
    async fn register_script(&self, name: &str, script_body: &str) -> CacheResult<()> {
        // Create script manager if not exists
        if self.lua_manager().is_none() {
            return Err(CacheError::OperationError(
                "Lua manager not initialized".to_string(),
            ));
        }

        // Register the script with the manager
        self.lua_manager().unwrap().register(name, script_body);

        // Load the script into Redis to validate syntax
        let script = Script::new(script_body);
        let mut conn = self
            .connection_manager()
            .get_connection()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        // Load the script and check for errors
        match script.prepare_invoke().load_async(&mut conn).await {
            Ok(_) => {
                debug!("Successfully registered Lua script: {}", name);
                Ok(())
            }
            Err(e) => {
                error!("Failed to register Lua script {}: {}", name, e);
                Err(CacheError::OperationError(format!(
                    "Failed to register Lua script: {}",
                    e
                )))
            }
        }
    }

    #[instrument(skip(self, keys, args), level = "debug")]
    async fn execute_script<T: DeserializeOwned + Send + Sync>(
        &self,
        name: &str,
        keys: &[&str],
        args: &[&str],
    ) -> CacheResult<T> {
        // Get the script from the registry
        let script = match self
            .lua_manager()
            .and_then(|manager| manager.get_script(name))
        {
            Some(script) => script,
            None => {
                return Err(CacheError::OperationError(format!(
                    "Lua script '{}' not found in registry",
                    name
                )));
            }
        };

        // Execute the script
        self.execute_script_instance(script, keys, args).await
    }

    #[instrument(skip(self, script_body, keys, args), level = "debug")]
    async fn execute_raw_script<T: DeserializeOwned + Send + Sync>(
        &self,
        script_body: &str,
        keys: &[&str],
        args: &[&str],
    ) -> CacheResult<T> {
        let script = Script::new(script_body);
        self.execute_script_instance(script, keys, args).await
    }

    #[instrument(skip(self), level = "debug")]
    async fn check_and_increment_counter(
        &self,
        key: &str,
        max_value: i64,
        ttl: Option<Duration>,
    ) -> CacheResult<bool> {
        debug!(
            "Check and increment counter {} with max value {}",
            key, max_value
        );

        // Define the Lua script for atomic check and increment
        let script_body = r#"
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

        let prefixed_key = self.connection_manager().prefixed_key(key);
        let ttl_seconds = ttl.map(|t| t.as_secs().to_string()).unwrap_or_default();

        let result: i64 = self
            .execute_raw_script(
                script_body,
                &[&prefixed_key],
                &[&max_value.to_string(), &ttl_seconds],
            )
            .await?;

        Ok(result == 1)
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn set_if_not_exists<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<bool> {
        debug!("Set if not exists for key: {}", key);

        // Serialize the value
        let serialized = self.serializer().serialize(value).await?;
        let serialized_str = String::from_utf8(serialized)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        // Define the Lua script for atomic SETNX with TTL
        let script_body = r#"
            local result = redis.call('SETNX', KEYS[1], ARGV[1])
            if result == 1 and ARGV[2] ~= '' then
                redis.call('EXPIRE', KEYS[1], ARGV[2])
            end
            return result
        "#;

        let prefixed_key = self.connection_manager().prefixed_key(key);
        let ttl_seconds = ttl.map(|t| t.as_secs().to_string()).unwrap_or_default();

        let result: i64 = self
            .execute_raw_script(
                script_body,
                &[&prefixed_key],
                &[&serialized_str, &ttl_seconds],
            )
            .await?;

        Ok(result == 1)
    }

    #[instrument(skip(self, new_value), level = "debug")]
    async fn update_hash_if_equals<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        field: &str,
        expected: &str,
        new_value: &T,
    ) -> CacheResult<bool> {
        debug!("Update hash if equals for key: {}, field: {}", key, field);

        // Serialize the value
        let serialized = self.serializer().serialize(new_value).await?;
        let serialized_str = String::from_utf8(serialized)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        // Define the Lua script for atomic hash update
        let script_body = r#"
            local current = redis.call('HGET', KEYS[1], ARGV[1])
            if current == ARGV[2] then
                redis.call('HSET', KEYS[1], ARGV[1], ARGV[3])
                return 1
            else
                return 0
            end
        "#;

        let prefixed_key = self.connection_manager().prefixed_key(key);

        let result: i64 = self
            .execute_raw_script(
                script_body,
                &[&prefixed_key],
                &[field, expected, &serialized_str],
            )
            .await?;

        Ok(result == 1)
    }

    #[instrument(skip(self), level = "debug")]
    async fn increment_and_expire(
        &self,
        key: &str,
        increment_by: i64,
        ttl: Duration,
    ) -> CacheResult<i64> {
        debug!(
            "Increment and expire key: {} by {} with TTL {:?}",
            key, increment_by, ttl
        );

        // Define the Lua script for atomic increment and expire
        let script_body = r#"
            local count = redis.call('INCRBY', KEYS[1], ARGV[1])
            redis.call('EXPIRE', KEYS[1], ARGV[2])
            return count
        "#;

        let prefixed_key = self.connection_manager().prefixed_key(key);
        let ttl_seconds = ttl.as_secs().to_string();

        self.execute_raw_script(
            script_body,
            &[&prefixed_key],
            &[&increment_by.to_string(), &ttl_seconds],
        )
        .await
    }
}

impl RedisCache {
    /// Execute a Script instance
    async fn execute_script_instance<T: DeserializeOwned + Send + Sync>(
        &self,
        script: Script,
        keys: &[&str],
        args: &[&str],
    ) -> CacheResult<T> {
        // Get a connection
        let mut conn = self
            .connection_manager()
            .get_connection()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        // Map keys to prefixed keys
        let prefixed_keys: Vec<String> = keys
            .iter()
            .map(|k| self.connection_manager().prefixed_key(k))
            .collect();

        // Prepare the script invocation
        let mut invocation = script.prepare_invoke();

        // Add keys
        for key in &prefixed_keys {
            invocation = invocation.key(key);
        }

        // Add args
        for arg in args {
            invocation = invocation.arg(*arg);
        }

        // Execute the script
        match invocation.invoke_async(&mut conn).await {
            Ok(result) => Ok(result),
            Err(e) => {
                error!("Failed to execute Lua script: {}", e);
                Err(CacheError::OperationError(format!(
                    "Failed to execute Lua script: {}",
                    e
                )))
            }
        }
    }

    /// Get the Lua manager
    fn lua_manager(&self) -> Option<&RedisLuaManager> {
        // The Lua manager is stored in the connection_manager
        // This is a placeholder approach - in a real implementation,
        // you would likely have this as a field on RedisCache
        None
    }
}

/// Initialize Redis with common Lua scripts
pub async fn initialize_common_scripts(cache: &RedisCache) -> CacheResult<()> {
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
        .await?;

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
        .await?;

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
        .await?;

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
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisCacheConfig;
    use navius_cache::serialization::JsonSerializer;
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
            key_prefix: "test:lua:".to_string(),
            default_ttl: Duration::from_secs(300),
            max_connections: 10,
            database: 0,
            password: None,
            use_tls: false,
            connection_timeout_seconds: 5,
            command_timeout_seconds: 2,
            retry_commands: true,
            max_retries: 3,
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
