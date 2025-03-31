use async_trait::async_trait;
use navius_cache::{
    error::{CacheError, CacheResult},
    operations::Cache,
    serialization::CacheSerializer,
};
use redis::{AsyncCommands, Pipeline, aio::ConnectionManager, pipe};
use serde::{Serialize, de::DeserializeOwned};
use std::{sync::Arc, time::Duration};
use tracing::{debug, error, instrument};

use crate::{
    error::{RedisCacheError, RedisCacheResult},
    operations::RedisCache,
};

use crate::connection::RedisConnectionManager;
use crate::metrics;
use std::time::Instant;

/// Batched operations for the Redis cache
#[async_trait]
pub trait RedisPipeline: Cache {
    /// Execute multiple operations in a single Redis pipeline
    ///
    /// This can significantly improve performance by reducing round trips to the Redis server.
    /// All operations in the pipeline are sent in a single batch, and responses are collected in order.
    async fn execute_pipeline<F, R>(&self, pipeline_fn: F) -> CacheResult<R>
    where
        F: FnOnce(RedisPipelineBuilder) -> RedisPipelineBuilder + Send,
        R: DeserializeOwned + Send + Sync;

    /// Set multiple key-value pairs in a single batch operation
    async fn set_many<T: Serialize + Send + Sync>(
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
            pipeline: pipe(),
            operation_count: 0,
        }
    }

    /// Add a GET operation to the pipeline
    pub fn get(mut self, key: &str) -> Self {
        self.pipeline = self.pipeline.cmd("GET").arg(key);
        self.operation_count += 1;
        self
    }

    /// Add a SET operation to the pipeline
    pub fn set(mut self, key: &str, value: &[u8]) -> Self {
        self.pipeline = self.pipeline.cmd("SET").arg(key).arg(value);
        self.operation_count += 1;
        self
    }

    /// Add a SETEX operation (SET with expiration) to the pipeline
    pub fn setex(mut self, key: &str, seconds: u64, value: &[u8]) -> Self {
        self.pipeline = self.pipeline.cmd("SETEX").arg(key).arg(seconds).arg(value);
        self.operation_count += 1;
        self
    }

    /// Add a DEL operation to the pipeline
    pub fn del(mut self, key: &str) -> Self {
        self.pipeline = self.pipeline.cmd("DEL").arg(key);
        self.operation_count += 1;
        self
    }

    /// Add a EXISTS operation to the pipeline
    pub fn exists(mut self, key: &str) -> Self {
        self.pipeline = self.pipeline.cmd("EXISTS").arg(key);
        self.operation_count += 1;
        self
    }

    /// Add an INCR operation to the pipeline
    pub fn incr(mut self, key: &str, amount: i64) -> Self {
        self.pipeline = self.pipeline.cmd("INCRBY").arg(key).arg(amount);
        self.operation_count += 1;
        self
    }

    /// Add a custom command to the pipeline
    pub fn cmd(mut self, cmd: &str, args: Vec<&str>) -> Self {
        let mut command = self.pipeline.cmd(cmd);
        for arg in args {
            command = command.arg(arg);
        }
        self.pipeline = command;
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

/// Pipeline trait for batching Redis operations
pub trait Pipeline {
    /// Execute the pipeline
    async fn execute(self) -> RedisCacheResult<()>;
}

/// Redis pipeline implementation
#[derive(Debug)]
pub struct RedisPipeline {
    /// Connection manager
    connection_manager: Arc<RedisConnectionManager>,
    /// Redis pipeline
    pipeline: RedisPipeline,
    /// Key for tracking which entity this pipeline is operating on
    key: String,
}

impl RedisPipeline {
    /// Create a new Redis pipeline
    pub fn new(connection_manager: Arc<RedisConnectionManager>, key: &str) -> Self {
        Self {
            connection_manager,
            pipeline: RedisPipeline::new(),
            key: key.to_string(),
        }
    }

    /// Add a command to the pipeline
    #[instrument(skip(self, args), level = "debug")]
    pub fn cmd<T: redis::ToRedisArgs>(&mut self, cmd: &str, args: &[T]) -> &mut Self {
        let mut redis_cmd = redis::cmd(cmd);
        for arg in args {
            redis_cmd.arg(arg);
        }
        self.pipeline.add_command(redis_cmd);
        self
    }

    /// Add a SET command to the pipeline
    #[instrument(skip(self, value), level = "debug")]
    pub fn set<V: redis::ToRedisArgs>(&mut self, key: &str, value: V) -> &mut Self {
        self.pipeline.set(key, value);
        self
    }

    /// Add a GET command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn get(&mut self, key: &str) -> &mut Self {
        self.pipeline.get(key);
        self
    }

    /// Add a DEL command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn del(&mut self, key: &str) -> &mut Self {
        self.pipeline.del(key);
        self
    }

    /// Add an EXISTS command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn exists(&mut self, key: &str) -> &mut Self {
        self.pipeline.exists(key);
        self
    }

    /// Add an EXPIRE command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn expire(&mut self, key: &str, seconds: usize) -> &mut Self {
        self.pipeline.expire(key, seconds);
        self
    }

    /// Add a TTL command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn ttl(&mut self, key: &str) -> &mut Self {
        self.pipeline.ttl(key);
        self
    }

    /// Add a HMSET command to the pipeline
    #[instrument(skip(self, items), level = "debug")]
    pub fn hmset<K: redis::ToRedisArgs, V: redis::ToRedisArgs>(
        &mut self,
        key: &str,
        items: &[(K, V)],
    ) -> &mut Self {
        self.pipeline.hmset(key, items);
        self
    }

    /// Add a HGET command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn hget(&mut self, key: &str, field: &str) -> &mut Self {
        self.pipeline.hget(key, field);
        self
    }

    /// Add a HDEL command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn hdel(&mut self, key: &str, field: &str) -> &mut Self {
        self.pipeline.hdel(key, field);
        self
    }

    /// Add a HGETALL command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn hgetall(&mut self, key: &str) -> &mut Self {
        self.pipeline.hgetall(key);
        self
    }

    /// Add a RPUSH command to the pipeline
    #[instrument(skip(self, value), level = "debug")]
    pub fn rpush<V: redis::ToRedisArgs>(&mut self, key: &str, value: V) -> &mut Self {
        self.pipeline.rpush(key, value);
        self
    }

    /// Add a LPUSH command to the pipeline
    #[instrument(skip(self, value), level = "debug")]
    pub fn lpush<V: redis::ToRedisArgs>(&mut self, key: &str, value: V) -> &mut Self {
        self.pipeline.lpush(key, value);
        self
    }

    /// Add a RPOP command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn rpop(&mut self, key: &str) -> &mut Self {
        self.pipeline.rpop(key);
        self
    }

    /// Add a LPOP command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn lpop(&mut self, key: &str) -> &mut Self {
        self.pipeline.lpop(key);
        self
    }

    /// Add a LRANGE command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn lrange(&mut self, key: &str, start: isize, stop: isize) -> &mut Self {
        self.pipeline.lrange(key, start, stop);
        self
    }

    /// Add a LLEN command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn llen(&mut self, key: &str) -> &mut Self {
        self.pipeline.llen(key);
        self
    }

    /// Add a SADD command to the pipeline
    #[instrument(skip(self, member), level = "debug")]
    pub fn sadd<M: redis::ToRedisArgs>(&mut self, key: &str, member: M) -> &mut Self {
        self.pipeline.sadd(key, member);
        self
    }

    /// Add a SREM command to the pipeline
    #[instrument(skip(self, member), level = "debug")]
    pub fn srem<M: redis::ToRedisArgs>(&mut self, key: &str, member: M) -> &mut Self {
        self.pipeline.srem(key, member);
        self
    }

    /// Add a SMEMBERS command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn smembers(&mut self, key: &str) -> &mut Self {
        self.pipeline.smembers(key);
        self
    }
}

impl Pipeline for RedisPipeline {
    /// Execute the pipeline
    #[instrument(skip(self), level = "debug")]
    async fn execute(self) -> RedisCacheResult<()> {
        let timer = metrics::TimedOperation::new(metrics::names::PIPELINE_EXECUTE);
        let start = Instant::now();
        let key = self.key.clone();
        let pipeline = self.pipeline;

        debug!("Executing Redis pipeline");

        let result = self
            .connection_manager
            .execute_command(&key, "PIPELINE", move |mut conn| {
                // Execute the pipeline
                pipeline.query(&mut conn)?;
                Ok(())
            })
            .await;

        // Record metrics
        timer.record(&result);

        result
    }
}

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
            .connection_manager()
            .get_connection()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        match pipeline
            .query_async::<_, Vec<redis::Value>>(&mut connection)
            .await
        {
            Ok(results) => {
                let result_bytes = serde_json::to_vec(&results)
                    .map_err(|e| CacheError::SerializationError(e.to_string()))?;

                match self.serializer().deserialize(&result_bytes).await {
                    Ok(value) => Ok(value),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(CacheError::OperationError(format!(
                "Pipeline execution failed: {}",
                e
            ))),
        }
    }

    #[instrument(skip(self, entries), level = "debug")]
    async fn set_many<T: Serialize + Send + Sync>(
        &self,
        entries: &[(&str, &T)],
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        if entries.is_empty() {
            return Ok(());
        }

        let mut connection = self
            .connection_manager()
            .get_connection()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut pipeline = pipe();

        for (key, value) in entries {
            let prefixed_key = self.connection_manager().prefixed_key(key);
            let serialized = self.serializer().serialize(value).await?;

            if let Some(ttl) = ttl {
                pipeline
                    .cmd("SETEX")
                    .arg(&prefixed_key)
                    .arg(ttl.as_secs())
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

        pipeline
            .query_async(&mut connection)
            .await
            .map_err(|e| CacheError::OperationError(format!("Pipeline set_many failed: {}", e)))
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
            .connection_manager()
            .get_connection()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut pipeline = pipe();
        let prefixed_keys: Vec<String> = keys
            .iter()
            .map(|k| self.connection_manager().prefixed_key(k))
            .collect();

        for key in &prefixed_keys {
            pipeline.cmd("GET").arg(key);
        }

        let results: Vec<Option<Vec<u8>>> = pipeline
            .query_async(&mut connection)
            .await
            .map_err(|e| CacheError::OperationError(format!("Pipeline get_many failed: {}", e)))?;

        let mut values = Vec::with_capacity(results.len());

        for result in results {
            match result {
                Some(data) => match self.serializer().deserialize(&data).await {
                    Ok(value) => values.push(Some(value)),
                    Err(_) => values.push(None),
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
            .connection_manager()
            .get_connection()
            .await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        let mut pipeline = pipe();
        let prefixed_keys: Vec<String> = keys
            .iter()
            .map(|k| self.connection_manager().prefixed_key(k))
            .collect();

        for key in &prefixed_keys {
            pipeline.cmd("DEL").arg(key);
        }

        let results: Vec<i64> = pipeline.query_async(&mut connection).await.map_err(|e| {
            CacheError::OperationError(format!("Pipeline delete_many failed: {}", e))
        })?;

        Ok(results.into_iter().map(|count| count > 0).collect())
    }
}
