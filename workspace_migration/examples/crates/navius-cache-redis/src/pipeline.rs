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
