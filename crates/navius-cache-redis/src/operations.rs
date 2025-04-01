use redis::{aio::ConnectionManager, AsyncCommands, RedisError};
use std::{fmt::Debug, marker::PhantomData, sync::Arc, time::Duration};
use tokio::time::timeout;
use tracing::{debug, error, instrument};

use crate::{connection::RedisConnectionManager, error::RedisCacheError, metrics};
use navius_cache::{error::CacheError, CacheKey, CacheOperations, CacheOptions};
use serde::{de::DeserializeOwned, Serialize};

pub type CacheResult<T> = Result<T, RedisCacheError>;

pub struct RedisOperations<K, V>
where
    K: CacheKey + 'static + std::fmt::Debug,
    V: Serialize + Send + Sync + 'static + std::fmt::Debug,
{
    pub(crate) connection_manager: Arc<RedisConnectionManager>,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> RedisOperations<K, V>
where
    K: CacheKey + 'static + std::fmt::Debug,
    V: Serialize + Send + Sync + 'static + std::fmt::Debug,
{
    pub fn new(connection_manager: Arc<RedisConnectionManager>) -> Self {
        Self {
            connection_manager,
            _phantom: PhantomData,
        }
    }

    pub fn get_connection_manager(&self) -> &RedisConnectionManager {
        &self.connection_manager
    }

    pub fn into_cache_error(err: RedisCacheError) -> CacheError {
        match err {
            RedisCacheError::ConnectionError(msg) => CacheError::ConnectionError(msg),
            RedisCacheError::OperationError(msg) => CacheError::OperationError(msg),
            RedisCacheError::SerializationError(msg) => CacheError::SerializationError(msg),
            RedisCacheError::DeserializationError(msg) => CacheError::SerializationError(msg),
            RedisCacheError::Timeout => {
                CacheError::TimeoutError("Redis operation timed out".to_string())
            }
            RedisCacheError::Redis(err) => CacheError::BackendError(err.to_string()),
        }
    }

    #[instrument(skip(self, value), level = "debug")]
    pub async fn set<K, V>(&self, key: K, value: V, ttl: Option<Duration>) -> CacheResult<()>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        V: Serialize + Send + Sync + 'static + std::fmt::Debug,
    {
        let mut conn = self.connection_manager.get_connection().await?;
        let key = self.connection_manager.prefix_key(&key.to_string());
        let serialized =
            serde_json::to_string(&value).map_err(|e| RedisCacheError::SerializationError(e))?;

        let result = timeout(
            self.connection_manager.command_timeout,
            conn.set(&key, &serialized),
        )
        .await;

        match result {
            Ok(Ok(_)) => {
                if let Some(ttl) = ttl {
                    let ttl_secs: i64 = ttl.as_secs().try_into().map_err(|_| {
                        RedisCacheError::OperationError(ToString::to_string("TTL value too large"))
                    })?;
                    let result = timeout(
                        self.connection_manager.command_timeout,
                        conn.expire(&key, ttl_secs),
                    )
                    .await;

                    match result {
                        Ok(Ok(_)) => Ok(()),
                        Ok(Err(e)) => Err(RedisCacheError::OperationError(ToString::to_string(&e))),
                        Err(_) => Err(RedisCacheError::Timeout(ToString::to_string(
                            "Operation timed out",
                        ))),
                    }
                } else {
                    Ok(())
                }
            }
            Ok(Err(e)) => Err(RedisCacheError::OperationError(ToString::to_string(&e))),
            Err(_) => Err(RedisCacheError::Timeout(ToString::to_string(
                "Operation timed out",
            ))),
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        V: DeserializeOwned + Send + Sync + 'static + std::fmt::Debug,
    {
        let mut conn = self.connection_manager.get_connection().await?;
        let key = self.connection_manager.prefix_key(&key.to_string());

        let result = timeout(
            self.connection_manager.command_timeout,
            conn.get::<_, Option<String>>(&key),
        )
        .await;

        match result {
            Ok(Ok(Some(value))) => {
                let deserialized = serde_json::from_str(&value)
                    .map_err(|e| RedisCacheError::SerializationError(e))?;
                Ok(Some(deserialized))
            }
            Ok(Ok(None)) => Ok(None),
            Ok(Err(e)) => Err(RedisCacheError::OperationError(ToString::to_string(&e))),
            Err(_) => Err(RedisCacheError::Timeout(ToString::to_string(
                "Operation timed out",
            ))),
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug,
    {
        let mut conn = self.connection_manager.get_connection().await?;
        let key = self.connection_manager.prefix_key(&key.to_string());

        let result = timeout(
            self.connection_manager.command_timeout,
            conn.del::<_, i64>(&key),
        )
        .await;

        match result {
            Ok(Ok(deleted)) => Ok(deleted > 0),
            Ok(Err(e)) => Err(RedisCacheError::OperationError(ToString::to_string(&e))),
            Err(_) => Err(RedisCacheError::Timeout(ToString::to_string(
                "Operation timed out",
            ))),
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug,
    {
        let mut conn = self.connection_manager.get_connection().await?;
        let key = self.connection_manager.prefix_key(&key.to_string());

        let result = timeout(
            self.connection_manager.command_timeout,
            conn.exists::<_, bool>(&key),
        )
        .await;

        match result {
            Ok(Ok(exists)) => Ok(exists),
            Ok(Err(e)) => Err(RedisCacheError::OperationError(ToString::to_string(&e))),
            Err(_) => Err(RedisCacheError::Timeout(ToString::to_string(
                "Operation timed out",
            ))),
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug,
    {
        let mut conn = self.connection_manager.get_connection().await?;
        let key = self.connection_manager.prefix_key(&key.to_string());

        let result = timeout(
            self.connection_manager.command_timeout,
            conn.expire::<_, bool>(&key, ttl.as_secs() as i64),
        )
        .await;

        match result {
            Ok(Ok(set)) => Ok(set),
            Ok(Err(e)) => Err(RedisCacheError::OperationError(ToString::to_string(&e))),
            Err(_) => Err(RedisCacheError::Timeout(ToString::to_string(
                "Operation timed out",
            ))),
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + std::fmt::Debug,
    {
        let mut conn = self.connection_manager.get_connection().await?;
        let key = self.connection_manager.prefix_key(&key.to_string());

        let result = timeout(
            self.connection_manager.command_timeout,
            conn.incr::<_, _, i64>(&key, amount),
        )
        .await;

        match result {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(e)) => Err(RedisCacheError::OperationError(ToString::to_string(&e))),
            Err(_) => Err(RedisCacheError::Timeout(ToString::to_string(
                "Operation timed out",
            ))),
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn flush(&self) -> CacheResult<()> {
        let mut conn = self.connection_manager.get_connection().await?;

        timeout(
            self.connection_manager.get_command_timeout(),
            redis::cmd("FLUSHDB").query_async(&mut conn),
        )
        .await
        .map_err(|_| RedisCacheError::Timeout(ToString::to_string("Flush operation timeout")))?
        .map_err(|e| RedisCacheError::OperationError(ToString::to_string(&e)))?;

        Ok(())
    }

    /// Get multiple values by keys
    #[instrument(skip(self), level = "debug")]
    pub async fn get_many<T>(&self, keys: &[&str]) -> CacheResult<Vec<Option<T>>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let prefixed_keys: Vec<String> = keys
            .iter()
            .map(|&k| self.connection_manager.prefix_key(k))
            .collect();

        let key_refs: Vec<&str> = prefixed_keys.iter().map(|k| k.as_str()).collect();

        let operation = metrics::names::GET_MANY;

        self.connection_manager
            .execute_command(&key_refs.join(","), operation, |conn| async move {
                let results: Vec<Option<Vec<u8>>> = conn.get(key_refs.as_slice()).await?;

                Ok(results)
            })
            .await?
            .into_iter()
            .map(|value| match value {
                Some(bytes) => match serde_json::from_slice(&bytes) {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => Err(RedisCacheError::DeserializationError(e)),
                },
                None => Ok(None),
            })
            .collect()
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn set_many<T>(&self, pairs: &[(&str, &T)], ttl: Option<Duration>) -> CacheResult<()>
    where
        T: Serialize + Send + Sync + 'static + Debug,
    {
        if pairs.is_empty() {
            return Ok(());
        }

        let operation = metrics::names::SET_MANY;
        let mut serialized_pairs = Vec::with_capacity(pairs.len());

        for (key, value) in pairs {
            let prefixed_key = self
                .connection_manager
                .prefix_key(&ToString::to_string(key));
            let serialized = self.connection_manager.serialize(value).await?;
            serialized_pairs.push((prefixed_key, serialized));
        }

        self.connection_manager
            .execute_command("multiple_keys", operation, |conn| async move {
                conn.mset(&serialized_pairs).await?;

                if let Some(ttl) = ttl {
                    for (key, _) in &serialized_pairs {
                        conn.expire(key, ttl.as_secs() as i64).await?;
                    }
                }

                Ok(())
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn delete_many(&self, keys: &[&str]) -> CacheResult<u64> {
        if keys.is_empty() {
            return Ok(0);
        }

        let prefixed_keys: Vec<String> = keys
            .iter()
            .map(|k| self.connection_manager.prefix_key(&ToString::to_string(k)))
            .collect();
        let operation = metrics::names::DELETE_MANY;

        self.connection_manager
            .execute_command(&prefixed_keys.join(","), operation, |conn| async move {
                let result: i64 = conn.del(prefixed_keys.as_slice()).await?;
                Ok(result as u64)
            })
            .await
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn set_with_ttl(&self, key: &str, value: &str, ttl: Duration) -> CacheResult<()> {
        let mut conn = self.connection_manager.get_connection().await?;
        let result = timeout(self.connection_manager.command_timeout, async {
            conn.set(key, value).await?;
            let ttl_secs: i64 = ttl.as_secs().try_into().map_err(|_| {
                RedisCacheError::OperationError(ToString::to_string("TTL value too large"))
            })?;
            conn.expire(key, ttl_secs).await
        })
        .await;

        match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(RedisCacheError::OperationError(ToString::to_string(&e))),
            Err(_) => Err(RedisCacheError::Timeout(ToString::to_string(
                "Operation timed out",
            ))),
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn check_health(&self) -> CacheResult<()> {
        self.connection_manager.check_health().await
    }

    pub async fn zset_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let mut conn = self.connection_manager.get_connection().await?;
        let key = self
            .connection_manager
            .prefix_key(&ToString::to_string(key.to_string()));

        let result: i64 = timeout(
            self.connection_manager.get_command_timeout(),
            conn.zcard(&key),
        )
        .await
        .map_err(|_| {
            RedisCacheError::Timeout(ToString::to_string("Sorted set length operation timeout"))
        })?
        .map_err(|e| RedisCacheError::OperationError(ToString::to_string(&e)))?;

        Ok(result as usize)
    }
}

#[async_trait::async_trait]
impl CacheOperations for RedisOperations<String, Vec<u8>> {
    async fn get<K, V>(&self, key: K) -> navius_cache::error::CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        self.get(key).await.map_err(Self::into_cache_error)
    }

    async fn get_many<K, V>(&self, keys: Vec<K>) -> navius_cache::error::CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        // Create a vector of Strings first
        let key_strings: Vec<String> = keys.iter().map(|k| k.to_string()).collect();

        // Then collect string slices from those strings
        let key_strs: Vec<&str> = key_strings.iter().map(AsRef::as_ref).collect();

        // Call our internal implementation with the string slices
        self.get_many(&key_strs)
            .await
            .map_err(|e| Self::into_cache_error(e))
    }

    async fn set<K, V>(
        &self,
        key: K,
        value: &V,
        options: Option<navius_cache::CacheOptions>,
    ) -> navius_cache::error::CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let ttl = options.as_ref().and_then(|opts| opts.ttl);
        self.set(key, value, ttl)
            .await
            .map_err(Self::into_cache_error)
    }

    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<navius_cache::CacheOptions>,
    ) -> navius_cache::error::CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if entries.is_empty() {
            return Ok(());
        }

        let ttl = options.as_ref().and_then(|opts| opts.ttl);

        // Convert entries to the format expected by set_many
        let pairs: Vec<(&str, &V)> = entries
            .iter()
            .map(|(k, v)| (k.to_string().as_str(), v))
            .collect();

        self.set_many(&pairs, ttl)
            .await
            .map_err(Self::into_cache_error)
    }

    async fn delete<K>(&self, key: K) -> navius_cache::error::CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        self.delete(key).await.map_err(Self::into_cache_error)
    }

    async fn delete_many<K>(&self, keys: Vec<K>) -> navius_cache::error::CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        if keys.is_empty() {
            return Ok(0);
        }

        // Create a vector of Strings first
        let key_strings: Vec<String> = keys.iter().map(|k| k.to_string()).collect();

        // Then collect string slices from those strings
        let key_strs: Vec<&str> = key_strings.iter().map(AsRef::as_ref).collect();

        self.delete_many(&key_strs)
            .await
            .map(|count| count as usize)
            .map_err(|e| Self::into_cache_error(e))
    }

    async fn exists<K>(&self, key: K) -> navius_cache::error::CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        self.exists(key).await.map_err(Self::into_cache_error)
    }

    async fn increment<K>(&self, key: K, amount: i64) -> navius_cache::error::CacheResult<i64>
    where
        K: CacheKey + 'static,
    {
        self.increment(key, amount)
            .await
            .map_err(Self::into_cache_error)
    }

    async fn expire<K>(&self, key: K, ttl: Duration) -> navius_cache::error::CacheResult<bool>
    where
        K: CacheKey + 'static,
    {
        self.expire(key, ttl).await.map_err(Self::into_cache_error)
    }

    async fn clear(&self) -> navius_cache::error::CacheResult<()> {
        self.flush().await.map_err(Self::into_cache_error)
    }

    async fn health_check(&self) -> navius_cache::error::CacheResult<()> {
        self.check_health().await.map_err(Self::into_cache_error)
    }

    // List Operations
    async fn list_push_right<K, V>(
        &self,
        key: K,
        value: &V,
    ) -> navius_cache::error::CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(Self::into_cache_error)?;

        let key = self.connection_manager.prefix_key(&key.to_string());
        let serialized = serde_json::to_string(value)
            .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;

        let result: i64 = redis::cmd("RPUSH")
            .arg(&key)
            .arg(&serialized)
            .query_async(&mut conn)
            .await
            .map_err(|e: RedisError| {
                navius_cache::error::CacheError::OperationError(e.to_string())
            })?;

        Ok(result as usize)
    }

    async fn list_push_right_many<K, V>(
        &self,
        key: K,
        values: &[V],
    ) -> navius_cache::error::CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if values.is_empty() {
            return Ok(0);
        }

        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(Self::into_cache_error)?;

        let key = self.connection_manager.prefix_key(&key.to_string());
        let mut cmd = redis::cmd("RPUSH");
        cmd.arg(&key);

        for value in values {
            let serialized = serde_json::to_string(value)
                .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;
            cmd.arg(&serialized);
        }

        let result: i64 = cmd.query_async(&mut conn).await.map_err(|e: RedisError| {
            navius_cache::error::CacheError::OperationError(e.to_string())
        })?;

        Ok(result as usize)
    }

    async fn list_push_left<K, V>(
        &self,
        key: K,
        value: &V,
    ) -> navius_cache::error::CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(Self::into_cache_error)?;

        let key = self.connection_manager.prefix_key(&key.to_string());
        let serialized = serde_json::to_string(value)
            .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;

        let result: i64 = redis::cmd("LPUSH")
            .arg(&key)
            .arg(&serialized)
            .query_async(&mut conn)
            .await
            .map_err(|e: RedisError| {
                navius_cache::error::CacheError::OperationError(e.to_string())
            })?;

        Ok(result as usize)
    }

    async fn list_push_left_many<K, V>(
        &self,
        key: K,
        values: &[V],
    ) -> navius_cache::error::CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        if values.is_empty() {
            return Ok(0);
        }

        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(Self::into_cache_error)?;

        let key = self.connection_manager.prefix_key(&key.to_string());
        let mut cmd = redis::cmd("LPUSH");
        cmd.arg(&key);

        for value in values {
            let serialized = serde_json::to_string(value)
                .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;
            cmd.arg(&serialized);
        }

        let result: i64 = cmd.query_async(&mut conn).await.map_err(|e: RedisError| {
            navius_cache::error::CacheError::OperationError(e.to_string())
        })?;

        Ok(result as usize)
    }

    async fn list_pop_right<K, V>(&self, key: K) -> navius_cache::error::CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(Self::into_cache_error)?;

        let key = self.connection_manager.prefix_key(&key.to_string());

        let result: Option<String> = redis::cmd("RPOP")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e: RedisError| {
                navius_cache::error::CacheError::OperationError(e.to_string())
            })?;

        match result {
            Some(serialized) => {
                let deserialized = serde_json::from_str(&serialized).map_err(|e| {
                    navius_cache::error::CacheError::SerializationError(e.to_string())
                })?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    async fn list_pop_left<K, V>(&self, key: K) -> navius_cache::error::CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(Self::into_cache_error)?;

        let key = self.connection_manager.prefix_key(&key.to_string());

        let result: Option<String> = redis::cmd("LPOP")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e: RedisError| {
                navius_cache::error::CacheError::OperationError(e.to_string())
            })?;

        match result {
            Some(serialized) => {
                let deserialized = serde_json::from_str(&serialized).map_err(|e| {
                    navius_cache::error::CacheError::SerializationError(e.to_string())
                })?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    async fn list_range<K, V>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> navius_cache::error::CacheResult<Vec<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static,
    {
        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(Self::into_cache_error)?;

        let key = self.connection_manager.prefix_key(&key.to_string());

        let result: Vec<String> = redis::cmd("LRANGE")
            .arg(&key)
            .arg(start)
            .arg(stop)
            .query_async(&mut conn)
            .await
            .map_err(|e: RedisError| {
                navius_cache::error::CacheError::OperationError(e.to_string())
            })?;

        let mut deserialized = Vec::with_capacity(result.len());
        for item in result {
            let value = serde_json::from_str(&item)
                .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;
            deserialized.push(value);
        }

        Ok(deserialized)
    }

    async fn list_length<K>(&self, key: K) -> navius_cache::error::CacheResult<usize>
    where
        K: CacheKey + 'static,
    {
        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(Self::into_cache_error)?;

        let key = self.connection_manager.prefix_key(&key.to_string());

        let result: i64 = redis::cmd("LLEN")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e: RedisError| {
                navius_cache::error::CacheError::OperationError(e.to_string())
            })?;

        Ok(result as usize)
    }

    async fn list_remove<K, V>(
        &self,
        key: K,
        count: isize,
        value: &V,
    ) -> navius_cache::error::CacheResult<usize>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(Self::into_cache_error)?;

        let key = self.connection_manager.prefix_key(&key.to_string());
        let serialized = serde_json::to_string(value)
            .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;

        let result: i64 = redis::cmd("LREM")
            .arg(&key)
            .arg(count)
            .arg(&serialized)
            .query_async(&mut conn)
            .await
            .map_err(|e: RedisError| {
                navius_cache::error::CacheError::OperationError(e.to_string())
            })?;

        Ok(result as usize)
    }

    async fn list_trim<K>(
        &self,
        key: K,
        start: isize,
        stop: isize,
    ) -> navius_cache::error::CacheResult<()>
    where
        K: CacheKey + 'static,
    {
        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(Self::into_cache_error)?;

        let key = self.connection_manager.prefix_key(&key.to_string());

        redis::cmd("LTRIM")
            .arg(&key)
            .arg(start)
            .arg(stop)
            .query_async(&mut conn)
            .await
            .map_err(|e: RedisError| {
                navius_cache::error::CacheError::OperationError(e.to_string())
            })?;

        Ok(())
    }

    async fn list_set<K, V>(
        &self,
        key: K,
        index: isize,
        value: &V,
    ) -> navius_cache::error::CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let mut conn = self
            .connection_manager
            .get_connection()
            .await
            .map_err(Self::into_cache_error)?;

        let key = self.connection_manager.prefix_key(&key.to_string());
        let serialized = serde_json::to_string(value)
            .map_err(|e| navius_cache::error::CacheError::SerializationError(e.to_string()))?;

        redis::cmd("LSET")
            .arg(&key)
            .arg(index)
            .arg(&serialized)
            .query_async(&mut conn)
            .await
            .map_err(|e: RedisError| {
                navius_cache::error::CacheError::OperationError(e.to_string())
            })?;

        Ok(())
    }
}
