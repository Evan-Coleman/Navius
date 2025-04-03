use crate::connection::RedisConnectionPool;
use crate::error::RedisCacheError;
use redis::{AsyncCommands, FromRedisValue, RedisResult, ScriptInvocation, ToRedisArgs};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, trace};

/// Helper for managing Lua scripts
pub struct LuaScript {
    /// Name of the script for debugging and metrics
    name: String,
    /// The Redis Lua script
    script: Arc<String>,
}

impl LuaScript {
    /// Create a new Lua script
    pub fn new<S: Into<String>>(name: S, script_source: &str) -> Self {
        Self {
            name: name.into(),
            script: Arc::new(script_source.to_string()),
        }
    }

    /// Execute the script with the given keys and arguments
    pub async fn execute<R, K, A>(
        &self,
        pool: &RedisConnectionPool,
        keys: K,
        args: A,
    ) -> Result<R, RedisCacheError>
    where
        R: FromRedisValue + Send + 'static,
        K: ToRedisArgs + Send + Sync + 'static + Clone,
        A: ToRedisArgs + Send + Sync + 'static + Clone,
    {
        let script_copy = self.script.clone();

        pool.execute(move |conn| {
            Box::pin(async move {
                let script = redis::Script::new(&*script_copy);
                let mut invocation = script.prepare_invoke();
                invocation.key(keys).arg(args).invoke_async(conn).await
            })
        })
        .await
    }

    // Make simpler methods like set_nx_ex that avoid lifetime issues
    async fn set_nx_ex<K>(
        &self,
        pool: &RedisConnectionPool,
        key: K,
        value: String,
        seconds: usize,
    ) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
    {
        let key_str = pool.prefixed_key(key);
        let script = LuaScript::new("set_nx_ex", SET_NX_EX_SCRIPT);

        let args: Vec<String> = vec![seconds.to_string(), value];
        let keys: Vec<String> = vec![key_str];

        let result: i64 = script.execute(pool, keys, args).await?;
        Ok(result == 1)
    }

    // Simplified version of expire_if_exists
    async fn expire_if_exists<K>(
        &self,
        pool: &RedisConnectionPool,
        key: K,
        seconds: usize,
    ) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync + 'static,
    {
        let key_str = pool.prefixed_key(key);
        let script = LuaScript::new("expire_if_exists", EXPIRE_IF_EXISTS_SCRIPT);

        let args: Vec<String> = vec![seconds.to_string()];
        let keys: Vec<String> = vec![key_str];

        let result: i64 = script.execute(pool, keys, args).await?;
        Ok(result == 1)
    }
}

/// Script to set a value with expiry only if it doesn't exist
pub static SET_NX_EX_SCRIPT: &str = r#"
if redis.call('exists', KEYS[1]) == 0 then
    return redis.call('setex', KEYS[1], ARGV[1], ARGV[2])
else
    return nil
end
"#;

/// Script to update the expiry only if the key exists
pub static EXPIRE_IF_EXISTS_SCRIPT: &str = r#"
if redis.call('exists', KEYS[1]) == 1 then
    return redis.call('expire', KEYS[1], ARGV[1])
else
    return 0
end
"#;

/// Script to increment a value atomically and set expiry
pub static INCR_EX_SCRIPT: &str = r#"
local current = redis.call('incr', KEYS[1])
redis.call('expire', KEYS[1], ARGV[1])
return current
"#;

/// Script to delete keys by pattern
pub static DELETE_BY_PATTERN_SCRIPT: &str = r#"
local keys = redis.call('keys', ARGV[1])
local count = 0
if #keys > 0 then
    count = redis.call('del', unpack(keys))
end
return count
"#;

/// Script to set multiple hash fields with a single expiry
pub static HMSET_EX_SCRIPT: &str = r#"
redis.call('hmset', KEYS[1], unpack(ARGV, 2))
return redis.call('expire', KEYS[1], ARGV[1])
"#;

/// Helper trait for commonly used Lua scripts
pub trait LuaScriptOperations {
    /// Set a value with expiry only if it doesn't exist
    async fn set_nx_ex<K, V>(
        &self,
        key: K,
        value: V,
        seconds: usize,
    ) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
        V: ToRedisArgs + Send + Sync;

    /// Update the expiry only if the key exists
    async fn expire_if_exists<K>(&self, key: K, seconds: usize) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync;

    /// Increment a value atomically and set expiry
    async fn incr_ex<K>(&self, key: K, seconds: usize) -> Result<i64, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync;

    /// Delete keys by pattern
    async fn delete_by_pattern<P>(&self, pattern: P) -> Result<i64, RedisCacheError>
    where
        P: AsRef<str> + Debug + Send + Sync;

    /// Set multiple hash fields with a single expiry
    async fn hmset_ex<K, F, V>(
        &self,
        key: K,
        fields: &[(F, V)],
        seconds: usize,
    ) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
        F: AsRef<str> + Debug + Send + Sync,
        V: ToRedisArgs + Send + Sync;
}

impl LuaScriptOperations for RedisConnectionPool {
    async fn set_nx_ex<K, V>(
        &self,
        key: K,
        value: V,
        seconds: usize,
    ) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let key_str = self.prefixed_key(key);
        let script = LuaScript::new("set_nx_ex", SET_NX_EX_SCRIPT);

        // Convert value to a string representation for Redis
        let mut value_bytes = Vec::new();
        value.write_redis_args(&mut value_bytes);
        let value_str = String::from_utf8_lossy(&value_bytes[0]).to_string();

        let result: Option<String> = script
            .execute(self, vec![key_str], vec![seconds.to_string(), value_str])
            .await?;

        Ok(result.is_some())
    }

    async fn expire_if_exists<K>(&self, key: K, seconds: usize) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
    {
        let key_str = self.prefixed_key(key);
        let script = LuaScript::new("expire_if_exists", EXPIRE_IF_EXISTS_SCRIPT);

        let result: i64 = script
            .execute(self, vec![key_str], vec![seconds.to_string()])
            .await?;

        Ok(result == 1)
    }

    async fn incr_ex<K>(&self, key: K, seconds: usize) -> Result<i64, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
    {
        let key_str = self.prefixed_key(key);
        let script = LuaScript::new("incr_ex", INCR_EX_SCRIPT);

        let result: i64 = script
            .execute(self, vec![key_str], vec![seconds.to_string()])
            .await?;

        Ok(result)
    }

    async fn delete_by_pattern<P>(&self, pattern: P) -> Result<i64, RedisCacheError>
    where
        P: AsRef<str> + Debug + Send + Sync,
    {
        // Handle the key prefix properly
        let pattern_str = pattern.as_ref();
        let prefixed_pattern = match &self.config().key_prefix {
            Some(prefix) if !prefix.is_empty() => {
                format!("{}:{}", prefix, pattern_str)
            }
            _ => pattern_str.to_string(),
        };

        let script = LuaScript::new("delete_by_pattern", DELETE_BY_PATTERN_SCRIPT);

        let result: i64 = script
            .execute(self, Vec::<String>::new(), vec![prefixed_pattern])
            .await?;

        Ok(result)
    }

    async fn hmset_ex<K, F, V>(
        &self,
        key: K,
        fields: &[(F, V)],
        seconds: usize,
    ) -> Result<bool, RedisCacheError>
    where
        K: AsRef<str> + Debug + Send + Sync,
        F: AsRef<str> + Debug + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let key_str = self.prefixed_key(key);
        let script = LuaScript::new("hmset_ex", HMSET_EX_SCRIPT);

        let mut args = Vec::new();
        args.push(seconds.to_string());

        for (field, value) in fields {
            let field_str = field.as_ref().to_string();
            args.push(field_str);

            // Convert value to a string representation for Redis
            let mut value_bytes = Vec::new();
            value.write_redis_args(&mut value_bytes);
            if !value_bytes.is_empty() {
                let value_str = String::from_utf8_lossy(&value_bytes[0]).to_string();
                args.push(value_str);
            }
        }

        let result: i64 = script.execute(self, vec![key_str], args).await?;

        Ok(result == 1)
    }
}

#[cfg(test)]
mod tests {
    // Tests for LuaScript are disabled until we can properly mock Redis
    // These tests require a running Redis instance and proper cleanup
}
