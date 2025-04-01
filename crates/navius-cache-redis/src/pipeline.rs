use async_trait::async_trait;
use navius_cache::operations::{Cache, CacheKey};
use redis::{
    aio::MultiplexedConnection as Connection, pipe, AsyncCommands, Pipeline as RedisPipeline,
    Value as RedisValue,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{sync::Arc, time::Duration};
use tracing::{debug, error, instrument};

use crate::{
    error::{RedisCacheError, RedisCacheResult},
    operations::RedisCache,
};

use crate::connection::RedisConnectionManager;
use crate::metrics;
use std::time::Instant;

/// Trait that defines pipeline operations
pub trait RedisCommandPipeline: Cache {
    /// Creates a new pipeline with the initial operation.
    fn pipeline(&self) -> RedisPipelineBuilder;

    /// Executes a pipeline with multiple commands.
    async fn execute_pipeline(&self, pipeline: RedisPipelineBuilder) -> RedisCacheResult<()>;
}

/// Builder for Redis pipelines
#[derive(Default)]
pub struct RedisPipelineBuilder {
    pipeline: RedisPipeline,
}

impl RedisPipelineBuilder {
    /// Creates a new pipeline builder
    pub fn new() -> Self {
        Self { pipeline: pipe() }
    }

    /// Adds a SET command to the pipeline
    #[instrument(skip(self, value), level = "debug")]
    pub fn set<K, V>(mut self, key: K, value: &V) -> Self
    where
        K: CacheKey,
        V: Serialize + Send + Sync,
    {
        let key_str = key.to_string();
        // Serialize the value
        let serialized = serde_json::to_string(value).unwrap_or_default();
        self.pipeline.cmd("SET").arg(&key_str).arg(serialized);
        self
    }

    /// Adds a GET command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn get<K>(mut self, key: K) -> Self
    where
        K: CacheKey,
    {
        let key_str = key.to_string();
        self.pipeline.cmd("GET").arg(&key_str);
        self
    }

    /// Adds a DELETE command to the pipeline
    #[instrument(skip(self), level = "debug")]
    pub fn delete<K>(mut self, key: K) -> Self
    where
        K: CacheKey,
    {
        let key_str = key.to_string();
        self.pipeline.cmd("DEL").arg(&key_str);
        self
    }

    /// Builds the pipeline and returns it
    pub fn build(self) -> RedisPipeline {
        self.pipeline
    }
}

/// Pipeline trait for batching Redis operations
pub trait Pipeline {
    /// Execute the pipeline
    async fn execute(self) -> RedisCacheResult<()>;
}

/// Redis pipeline implementation
#[derive(Debug)]
pub struct RedisPipelineImpl {
    /// Connection manager
    connection_manager: Arc<RedisConnectionManager>,
    /// Redis pipeline
    pipeline: RedisPipeline,
    /// Key for tracking which entity this pipeline is operating on
    key: String,
}

impl RedisPipelineImpl {
    /// Create a new Redis pipeline
    pub fn new(connection_manager: Arc<RedisConnectionManager>, key: &str) -> Self {
        Self {
            connection_manager,
            pipeline: pipe(),
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

impl Pipeline for RedisPipelineImpl {
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

impl RedisCommandPipeline for RedisCache {
    #[instrument(skip(self), level = "debug")]
    fn pipeline(&self) -> RedisPipelineBuilder {
        RedisPipelineBuilder::new()
    }

    #[instrument(skip(self), level = "debug")]
    async fn execute_pipeline(
        &self,
        pipeline_builder: RedisPipelineBuilder,
    ) -> RedisCacheResult<()> {
        let start = Instant::now();
        let pipeline = pipeline_builder.build();

        self.execute_sync_command("PIPELINE", |mut conn| {
            pipeline.query(&mut conn)?;
            Ok(())
        })
        .await
        .map_err(|e| {
            let err = RedisCacheError::Other(format!("Pipeline error: {}", e));
            metrics::record_operation_error(metrics::names::PIPELINE_EXECUTE, &err);
            err
        })?;

        metrics::record_operation_duration(metrics::names::PIPELINE_EXECUTE, start.elapsed());
        Ok(())
    }
}
