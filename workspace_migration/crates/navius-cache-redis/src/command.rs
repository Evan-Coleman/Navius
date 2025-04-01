impl RedisCache {
    pub(crate) async fn execute_command<K, F, T, E>(
        &self,
        key: &K,
        command: &str,
        f: F,
    ) -> Result<T, RedisCacheError>
    where
        K: CacheKey + std::fmt::Debug,
        F: FnOnce(redis::aio::Connection) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>> + Send,
        E: Into<RedisCacheError>,
        T: Send + 'static,
    {
        // ... existing code ...
    }
}

// For the pipeline execution function too, ensure all futures are properly awaited
impl RedisPipeline {
    pub async fn execute(self) -> Result<(), RedisCacheError> {
        let cache = self.cache.clone();
        
        // Get a connection from the pool
        let mut conn = cache.connection().await?;
        
        // Convert the stored commands to a redis pipeline
        let mut pipe = redis::pipe();
        
        for cmd in self.commands {
            match cmd {
                PipelineCommand::Set(key, value, expiry) => {
                    // Apply expiry if provided
                    if let Some(duration) = expiry {
                        pipe.cmd("SETEX")
                            .arg(key)
                            .arg(duration.as_secs())
                            .arg(value);
                    } else {
                        pipe.cmd("SET").arg(key).arg(value);
                    }
                },
                // ... other commands ...
            }
        }
        
        // Execute the pipeline
        pipe.query_async(&mut conn).await.map_err(|e| {
            RedisCacheError::CommandError(format!("Failed to execute pipeline: {}", e))
        })?;
        
        Ok(())
    }
}

// Ensure any other async function implementations properly await their futures 