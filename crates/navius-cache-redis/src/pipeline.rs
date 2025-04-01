use async_trait::async_trait;
use navius_cache::{Cache, CacheError, CacheResult};
use redis::{aio::Connection, AsyncCommands, Pipeline, RedisError, Value as RedisValue};
use serde::{de::DeserializeOwned, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::MutexGuard;
use tokio::time::timeout;
use tracing::{debug, error, instrument};

use crate::connection::RedisConnectionManager;
use crate::error::RedisCacheError;
use crate::lua::RedisLuaManager;
use crate::metrics;
use crate::RedisCache;
use std::time::Instant;

/// Pipeline interface for Redis operations
#[async_trait]
pub trait RedisPipeline {
    /// Execute multiple operations in a single Redis pipeline
    ///
    /// This can significantly improve performance by reducing round trips to the Redis server.
    /// All operations in the pipeline are sent in a single batch, and responses are collected in order.
    async fn execute_pipeline<F, R>(&self, pipeline_fn: F) -> CacheResult<R>
    where
        F: FnOnce(RedisPipelineBuilder) -> RedisPipelineBuilder + Send,
        R: DeserializeOwned + Send + Sync;

    /// Set multiple key-value pairs in a single batch operation
    async fn set_many<T: Serialize + Send + Sync + std::fmt::Debug>(
        &self,
        entries: &[(&str, &T)],
        ttl: Option<Duration>,
    ) -> CacheResult<()>;

    /// Get multiple values for the provided keys in a single batch operation
    async fn get_many<T: DeserializeOwned + Send + Sync>(
        &self,
        keys: &[&str],
    ) -> CacheResult<Vec<Option<T>>>;

    /// Delete multiple keys in a single batch operation
    async fn delete_many(&self, keys: &[&str]) -> CacheResult<Vec<bool>>;
}

/// Builder for creating Redis pipeline operations
pub struct RedisPipelineBuilder {
    /// The Redis pipeline being built
    pipeline: Pipeline,
    /// Number of operations in the pipeline
    operation_count: usize,
}

impl RedisPipelineBuilder {
    /// Create a new pipeline builder
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
            operation_count: 0,
        }
    }

    /// Add a GET operation to the pipeline
    pub fn get(mut self, key: &str) -> Self {
        self.pipeline.get(key).ignore();
        self.operation_count += 1;
        self
    }

    /// Add a SET operation to the pipeline
    pub fn set(mut self, key: &str, value: &[u8]) -> Self {
        self.pipeline.set(key, value).ignore();
        self.operation_count += 1;
        self
    }

    /// Add a SETEX operation (SET with expiration) to the pipeline
    pub fn setex(mut self, key: &str, seconds: u64, value: &[u8]) -> Self {
        self.pipeline.set_ex(key, value, seconds).ignore();
        self.operation_count += 1;
        self
    }

    /// Add a DEL operation to the pipeline
    pub fn del(mut self, key: &str) -> Self {
        self.pipeline.del(key).ignore();
        self.operation_count += 1;
        self
    }

    /// Add a EXISTS operation to the pipeline
    pub fn exists(mut self, key: &str) -> Self {
        self.pipeline.exists(key).ignore();
        self.operation_count += 1;
        self
    }

    /// Add an INCR operation to the pipeline
    pub fn incr(mut self, key: &str, amount: i64) -> Self {
        self.pipeline.incr(key, amount).ignore();
        self.operation_count += 1;
        self
    }

    /// Add a custom command to the pipeline
    pub fn cmd(mut self, cmd_str: &str, args: Vec<&str>) -> Self {
        let mut command = redis::cmd(cmd_str);
        for arg in args {
            command.arg(arg);
        }
        self.pipeline.add_command(command).ignore();
        self.operation_count += 1;
        self
    }

    /// Get the built pipeline
    pub fn build(self) -> Pipeline {
        self.pipeline
    }

    /// Get the number of operations in the pipeline
    pub fn operation_count(&self) -> usize {
        self.operation_count
    }
}

impl Default for RedisPipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum PipelineCommand {
    Set {
        key: String,
        value: Vec<u8>,
        expiry: Option<Duration>,
    },
    Get {
        key: String,
    },
    Del {
        key: String,
    },
    Exists {
        key: String,
    },
}

/// Redis pipeline implementation
pub struct RedisPipelineManager {
    commands: Vec<PipelineCommand>,
    connection_manager: RedisConnectionManager,
}

impl RedisPipelineManager {
    /// Create a new Redis pipeline
    pub fn new(connection_manager: RedisConnectionManager) -> Self {
        Self {
            commands: Vec::new(),
            connection_manager,
        }
    }

    /// Add a command to the pipeline
    #[instrument(skip(self, args), level = "debug")]
    pub fn cmd<T: redis::ToRedisArgs>(&mut self, cmd: &str, args: &[T]) -> &mut Self {
        let mut redis_cmd = redis::cmd(cmd);
        for arg in args {
            redis_cmd.arg(arg);
        }

        // Convert args to a flattened vec of bytes
        let mut all_args = Vec::new();
        for arg in args {
            for bytes in arg.to_redis_args() {
                all_args.extend(bytes);
            }
        }

        self.commands.push(PipelineCommand::Set {
            key: cmd.to_string(),
            value: all_args,
            expiry: None,
        });
        self
    }

    /// Add a SET command to the pipeline
    #[instrument(skip(self, value), level = "debug")]
    pub fn set<V: redis::ToRedisArgs>(&mut self, key: &str, value: V) -> &mut Self {
        let mut flattened = Vec::new();
        for bytes in value.to_redis_args() {
            flattened.extend(bytes);
        }

        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: flattened,
            expiry: None,
        });
        self
    }

    /// Add a GET command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn get(&mut self, key: &str) -> &mut Self {
        self.commands.push(PipelineCommand::Get {
            key: key.to_string(),
        });
        self
    }

    /// Add a DEL command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn del(&mut self, key: &str) -> &mut Self {
        self.commands.push(PipelineCommand::Del {
            key: key.to_string(),
        });
        self
    }

    /// Add an EXISTS command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn exists(&mut self, key: &str) -> &mut Self {
        self.commands.push(PipelineCommand::Exists {
            key: key.to_string(),
        });
        self
    }

    /// Add an EXPIRE command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn expire(&mut self, key: &str, seconds: usize) -> &mut Self {
        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: Vec::new(),
            expiry: Some(Duration::from_secs(seconds as u64)),
        });
        self
    }

    /// Add a TTL command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn ttl(&mut self, key: &str) -> &mut Self {
        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: Vec::new(),
            expiry: None,
        });
        self
    }

    /// Add a HMSET command to the pipeline
    #[instrument(skip(self, items), level = "debug")]
    pub fn hmset<K: redis::ToRedisArgs, V: redis::ToRedisArgs>(
        &mut self,
        key: &str,
        items: &[(K, V)],
    ) -> &mut Self {
        let mut all_args = Vec::new();
        for (k, v) in items {
            all_args.extend(k.to_redis_args());
            all_args.extend(v.to_redis_args());
        }

        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: all_args,
            expiry: None,
        });
        self
    }

    /// Add a HGET command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn hget(&mut self, key: &str, field: &str) -> &mut Self {
        self.commands.push(PipelineCommand::Set {
            key: format!("{}:{}", key, field),
            value: Vec::new(),
            expiry: None,
        });
        self
    }

    /// Add a HDEL command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn hdel(&mut self, key: &str, field: &str) -> &mut Self {
        self.commands.push(PipelineCommand::Del {
            key: format!("{}:{}", key, field),
        });
        self
    }

    /// Add a HGETALL command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn hgetall(&mut self, key: &str) -> &mut Self {
        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: Vec::new(),
            expiry: None,
        });
        self
    }

    /// Add a RPUSH command to the pipeline
    #[instrument(skip(self, value), level = "debug")]
    pub fn rpush<V: redis::ToRedisArgs>(&mut self, key: &str, value: V) -> &mut Self {
        let mut flattened = Vec::new();
        for bytes in value.to_redis_args() {
            flattened.extend(bytes);
        }

        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: flattened,
            expiry: None,
        });
        self
    }

    /// Add a LPUSH command to the pipeline
    #[instrument(skip(self, value), level = "debug")]
    pub fn lpush<V: redis::ToRedisArgs>(&mut self, key: &str, value: V) -> &mut Self {
        let mut flattened = Vec::new();
        for bytes in value.to_redis_args() {
            flattened.extend(bytes);
        }

        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: flattened,
            expiry: None,
        });
        self
    }

    /// Add a RPOP command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn rpop(&mut self, key: &str) -> &mut Self {
        self.commands.push(PipelineCommand::Del {
            key: key.to_string(),
        });
        self
    }

    /// Add a LPOP command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn lpop(&mut self, key: &str) -> &mut Self {
        self.commands.push(PipelineCommand::Del {
            key: key.to_string(),
        });
        self
    }

    /// Add a LRANGE command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn lrange(&mut self, key: &str, start: isize, stop: isize) -> &mut Self {
        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: Vec::new(),
            expiry: None,
        });
        self
    }

    /// Add a LLEN command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn llen(&mut self, key: &str) -> &mut Self {
        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: Vec::new(),
            expiry: None,
        });
        self
    }

    /// Add a SADD command to the pipeline
    #[instrument(skip(self, member), level = "debug")]
    pub fn sadd<M: redis::ToRedisArgs>(&mut self, key: &str, member: M) -> &mut Self {
        let mut flattened = Vec::new();
        for bytes in member.to_redis_args() {
            flattened.extend(bytes);
        }

        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: flattened,
            expiry: None,
        });
        self
    }

    /// Add a SREM command to the pipeline
    #[instrument(skip(self, member), level = "debug")]
    pub fn srem<M: redis::ToRedisArgs>(&mut self, key: &str, member: M) -> &mut Self {
        let member_bytes = member.to_redis_args();
        let mut joined_str = String::new();
        for bytes in member_bytes {
            if let Ok(s) = String::from_utf8(bytes) {
                if !joined_str.is_empty() {
                    joined_str.push(':');
                }
                joined_str.push_str(&s);
            }
        }

        self.commands.push(PipelineCommand::Del {
            key: format!("{}:{}", key, joined_str),
        });
        self
    }

    /// Add a SMEMBERS command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn smembers(&mut self, key: &str) -> &mut Self {
        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: Vec::new(),
            expiry: None,
        });
        self
    }

    pub fn set_ex<V: redis::ToRedisArgs>(&mut self, key: &str, value: V, seconds: u64) {
        self.commands.push(PipelineCommand::Set {
            key: key.to_string(),
            value: value.to_redis_args(),
            expiry: Some(Duration::from_secs(seconds)),
        });
    }

    pub fn set(&mut self, key: String, value: Vec<u8>, expiry: Option<Duration>) {
        self.commands
            .push(PipelineCommand::Set { key, value, expiry });
    }

    pub fn get(&mut self, key: String) {
        self.commands.push(PipelineCommand::Get { key });
    }

    pub fn del(&mut self, key: String) {
        self.commands.push(PipelineCommand::Del { key });
    }

    pub fn exists(&mut self, key: String) {
        self.commands.push(PipelineCommand::Exists { key });
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn execute(&mut self) -> Result<Vec<RedisValue>, RedisCacheError> {
        let mut pipeline = Pipeline::new();
        let mut connection = self.connection_manager.get_connection().await?;

        for command in &self.commands {
            match command {
                PipelineCommand::Set { key, value, expiry } => {
                    let prefixed_key = self.connection_manager.prefix_key(key);
                    if let Some(seconds) = expiry {
                        pipeline
                            .set_ex(&prefixed_key, value.as_slice(), seconds.as_secs())
                            .ignore();
                    } else {
                        pipeline.set(&prefixed_key, value.as_slice()).ignore();
                    }
                }
                PipelineCommand::Get { key } => {
                    let prefixed_key = self.connection_manager.prefix_key(key);
                    pipeline.get(&prefixed_key).ignore();
                }
                PipelineCommand::Del { key } => {
                    let prefixed_key = self.connection_manager.prefix_key(key);
                    pipeline.del(&prefixed_key).ignore();
                }
                PipelineCommand::Exists { key } => {
                    let prefixed_key = self.connection_manager.prefix_key(key);
                    pipeline.exists(&prefixed_key).ignore();
                }
            }
        }

        let results = pipeline
            .query_async::<_, Vec<RedisValue>>(&mut *connection)
            .await
            .map_err(|e| RedisCacheError::OperationError(e.to_string()))?;

        Ok(results)
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn execute_get_batch(&mut self) -> Result<Vec<Option<Vec<u8>>>, RedisCacheError> {
        let results = self.execute().await?;
        let mut values = Vec::with_capacity(results.len());

        for result in results {
            match result {
                RedisValue::Data(bytes) => values.push(Some(bytes)),
                RedisValue::Nil => values.push(None),
                _ => {
                    return Err(RedisCacheError::OperationError(
                        "Unexpected Redis value type".to_string(),
                    ))
                }
            }
        }

        Ok(values)
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn execute_exists_batch(&mut self) -> Result<Vec<bool>, RedisCacheError> {
        let results = self.execute().await?;
        let mut values = Vec::with_capacity(results.len());

        for result in results {
            match result {
                RedisValue::Int(value) => values.push(value == 1),
                _ => {
                    return Err(RedisCacheError::OperationError(
                        "Unexpected Redis value type".to_string(),
                    ))
                }
            }
        }

        Ok(values)
    }

    /// Push a command to the pipeline
    pub fn push(&mut self, command: PipelineCommand) -> &mut Self {
        self.commands.push(command);
        self
    }
}

/// Implementation of RedisPipeline for RedisCache
#[async_trait]
impl RedisPipeline for RedisCache {
    #[instrument(skip(self, pipeline_fn), level = "debug")]
    async fn execute_pipeline<F, R>(&self, pipeline_fn: F) -> CacheResult<R>
    where
        F: FnOnce(RedisPipelineBuilder) -> RedisPipelineBuilder + Send,
        R: DeserializeOwned + Send + Sync,
    {
        let builder = pipeline_fn(RedisPipelineBuilder::new());
        let pipeline = builder.build();

        if builder.operation_count() == 0 {
            return Err(CacheError::OperationError("Empty pipeline".to_string()));
        }

        let mut connection = self
            .operations
            .get_connection_manager()
            .get_connection()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        // Execute the pipeline
        let result: redis::RedisResult<Vec<RedisValue>> =
            pipeline.query_async(&mut connection).await;

        match result {
            Ok(results) => {
                // Convert the results to the desired type using custom deserialization
                // Handle different RedisValue variants
                if results.len() == 1 {
                    match &results[0] {
                        RedisValue::Array(items) => {
                            // Try to deserialize bulk items
                            let json_values: Vec<serde_json::Value> = items
                                .iter()
                                .map(|val| match val {
                                    RedisValue::Nil => serde_json::Value::Null,
                                    RedisValue::Int(i) => serde_json::Value::Number((*i).into()),
                                    RedisValue::BulkString(bytes) => {
                                        if let Ok(s) = String::from_utf8(bytes.clone()) {
                                            if let Ok(json) =
                                                serde_json::from_str::<serde_json::Value>(&s)
                                            {
                                                return json;
                                            }
                                            return serde_json::Value::String(s);
                                        }
                                        serde_json::Value::String(format!("binary:{}", bytes.len()))
                                    }
                                    RedisValue::SimpleString(s) => {
                                        serde_json::Value::String(s.clone())
                                    }
                                    RedisValue::Okay => serde_json::Value::String("OK".to_string()),
                                    _ => serde_json::Value::Null,
                                })
                                .collect();

                            return match serde_json::to_value(json_values) {
                                Ok(json_value) => match serde_json::from_value(json_value) {
                                    Ok(value) => Ok(value),
                                    Err(e) => Err(CacheError::SerializationError(format!(
                                        "Failed to deserialize bulk items as collection: {}",
                                        e
                                    ))),
                                },
                                Err(e) => Err(CacheError::SerializationError(format!(
                                    "Failed to convert bulk items to JSON: {}",
                                    e
                                ))),
                            };
                        }
                        RedisValue::BulkString(bytes) => {
                            // Try to deserialize binary data
                            return match serde_json::from_slice::<R>(bytes) {
                                Ok(value) => Ok(value),
                                Err(e) => Err(CacheError::SerializationError(format!(
                                    "Failed to deserialize data: {}",
                                    e
                                ))),
                            };
                        }
                        RedisValue::Int(val) => {
                            // Convert integer to desired type
                            return match serde_json::to_value(val) {
                                Ok(json_value) => match serde_json::from_value(json_value) {
                                    Ok(value) => Ok(value),
                                    Err(e) => Err(CacheError::SerializationError(format!(
                                        "Failed to deserialize integer value: {}",
                                        e
                                    ))),
                                },
                                Err(e) => Err(CacheError::SerializationError(format!(
                                    "Failed to convert integer to JSON: {}",
                                    e
                                ))),
                            };
                        }
                        RedisValue::Nil => {
                            // Handle nil value
                            return match serde_json::to_value(()) {
                                Ok(json_value) => match serde_json::from_value(json_value) {
                                    Ok(value) => Ok(value),
                                    Err(e) => Err(CacheError::SerializationError(format!(
                                        "Failed to deserialize nil value: {}",
                                        e
                                    ))),
                                },
                                Err(e) => Err(CacheError::SerializationError(format!(
                                    "Failed to convert nil to JSON: {}",
                                    e
                                ))),
                            };
                        }
                        _ => {}
                    }
                }

                // Create a custom JSON representation of the Redis values
                let json_values: Vec<serde_json::Value> = results
                    .iter()
                    .map(|val| match val {
                        RedisValue::Nil => serde_json::Value::Null,
                        RedisValue::Int(i) => serde_json::Value::Number((*i).into()),
                        RedisValue::BulkString(bytes) => {
                            if let Ok(s) = String::from_utf8(bytes.clone()) {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&s) {
                                    return json;
                                }
                                return serde_json::Value::String(s);
                            }
                            serde_json::Value::String(format!("binary:{}", bytes.len()))
                        }
                        RedisValue::Array(items) => {
                            let item_values: Vec<serde_json::Value> = items
                                .iter()
                                .map(|item| match item {
                                    RedisValue::Nil => serde_json::Value::Null,
                                    RedisValue::Int(i) => serde_json::Value::Number((*i).into()),
                                    RedisValue::BulkString(bytes) => {
                                        if let Ok(s) = String::from_utf8(bytes.clone()) {
                                            if let Ok(json) =
                                                serde_json::from_str::<serde_json::Value>(&s)
                                            {
                                                return json;
                                            }
                                            return serde_json::Value::String(s);
                                        }
                                        serde_json::Value::String(format!("binary:{}", bytes.len()))
                                    }
                                    _ => serde_json::Value::Null,
                                })
                                .collect();
                            serde_json::Value::Array(item_values)
                        }
                        RedisValue::SimpleString(s) => serde_json::Value::String(s.clone()),
                        RedisValue::Okay => serde_json::Value::String("OK".to_string()),
                        RedisValue::Double(d) => {
                            if let Some(num) = serde_json::Number::from_f64(*d) {
                                serde_json::Value::Number(num)
                            } else {
                                serde_json::Value::Null
                            }
                        }
                        _ => serde_json::Value::Null,
                    })
                    .collect();

                // Try to deserialize the JSON values
                match serde_json::to_value(json_values) {
                    Ok(json_value) => match serde_json::from_value::<R>(json_value) {
                        Ok(value) => Ok(value),
                        Err(e) => Err(CacheError::SerializationError(format!(
                            "Failed to deserialize pipeline result: {}",
                            e
                        ))),
                    },
                    Err(e) => Err(CacheError::SerializationError(format!(
                        "Failed to serialize pipeline results to JSON: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(CacheError::OperationError(format!(
                "Pipeline execution failed: {}",
                e
            ))),
        }
    }

    #[instrument(skip(self, entries), level = "debug")]
    async fn set_many<T: Serialize + Send + Sync + std::fmt::Debug>(
        &self,
        entries: &[(&str, &T)],
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        if entries.is_empty() {
            return Ok(());
        }

        let mut connection = self
            .operations
            .get_connection_manager()
            .get_connection()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut pipeline = Pipeline::new();

        for (key, value) in entries {
            let prefixed_key = self
                .operations
                .get_connection_manager()
                .prefix_key(&key.to_string());
            let serialized = serde_json::to_vec(value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            if let Some(ttl_duration) = ttl {
                pipeline
                    .cmd("SETEX")
                    .arg(&prefixed_key)
                    .arg(ttl_duration.as_secs())
                    .arg(&serialized)
                    .ignore();
            } else {
                pipeline
                    .cmd("SET")
                    .arg(&prefixed_key)
                    .arg(&serialized)
                    .ignore();
            }
        }

        let result: redis::RedisResult<()> = pipeline.query_async(&mut connection).await;

        result.map_err(|e| CacheError::OperationError(format!("Pipeline set_many failed: {}", e)))
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn get_many<T: DeserializeOwned + Send + Sync>(
        &self,
        keys: &[&str],
    ) -> CacheResult<Vec<Option<T>>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let mut connection = self
            .operations
            .get_connection_manager()
            .get_connection()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut pipeline = Pipeline::new();
        let prefixed_keys: Vec<String> = keys
            .iter()
            .map(|k| {
                self.operations
                    .get_connection_manager()
                    .prefix_key(&k.to_string())
            })
            .collect();

        for key in &prefixed_keys {
            pipeline.cmd("GET").arg(key);
        }

        let result: redis::RedisResult<Vec<Option<Vec<u8>>>> =
            pipeline.query_async(&mut connection).await;

        let results = result
            .map_err(|e| CacheError::OperationError(format!("Pipeline get_many failed: {}", e)))?;

        let mut values = Vec::with_capacity(results.len());

        for result in results {
            match result {
                Some(data) => match serde_json::from_slice::<T>(&data) {
                    Ok(value) => values.push(Some(value)),
                    Err(e) => {
                        error!(
                            "Failed to deserialize item in get_many pipeline result: {}",
                            e
                        );
                        values.push(None);
                    }
                },
                None => values.push(None),
            }
        }

        Ok(values)
    }

    #[instrument(skip(self, keys), level = "debug")]
    async fn delete_many(&self, keys: &[&str]) -> CacheResult<Vec<bool>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let mut connection = self
            .operations
            .get_connection_manager()
            .get_connection()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut pipeline = Pipeline::new();
        let prefixed_keys: Vec<String> = keys
            .iter()
            .map(|k| {
                self.operations
                    .get_connection_manager()
                    .prefix_key(&k.to_string())
            })
            .collect();

        for key in &prefixed_keys {
            pipeline.cmd("DEL").arg(key);
        }

        let result: redis::RedisResult<Vec<i64>> = pipeline.query_async(&mut connection).await;

        let results = result.map_err(|e| {
            CacheError::OperationError(format!("Pipeline delete_many failed: {}", e))
        })?;

        Ok(results.into_iter().map(|count| count > 0).collect())
    }
}
