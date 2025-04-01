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
        let operation = TimedOperation::new(command);
        let prefixed_key = self.prefix_key(key);
        
        let conn = self.connection().await?;
        
        let result = f(conn).await.map_err(|e| e.into());
        
        operation.complete(result.is_ok());
        
        result
    }
} 