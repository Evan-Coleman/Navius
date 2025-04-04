use async_trait::async_trait;
/// A mock Redis implementation for unit testing
///
/// This module provides a simple in-memory mock of Redis for testing without
/// requiring an actual Redis server.
use navius_cache::{Cache, CacheKey, CacheOperations, CacheOptions, CacheResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Structure representing a cached value with metadata
struct CachedValue {
    value: Vec<u8>,
    expires_at: Option<Instant>,
}

/// A simple in-memory mock of Redis cache for testing
pub struct MockRedis {
    data: Arc<Mutex<HashMap<String, CachedValue>>>,
    prefix: String,
    default_ttl: Duration,
}

impl MockRedis {
    /// Create a new MockRedis instance
    pub fn new(prefix: String, default_ttl: Duration) -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            prefix,
            default_ttl,
        }
    }

    /// Helper to format a key with the prefix
    fn format_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }

    /// Clear all data in the mock Redis
    pub async fn clear(&self) {
        let mut data = self.data.lock().await;
        data.clear();
    }

    /// Simulate a delay (useful for testing timeouts)
    pub async fn simulate_delay(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }

    /// Get the number of stored keys
    pub async fn len(&self) -> usize {
        let data = self.data.lock().await;
        data.len()
    }

    /// Check if the mock Redis is empty
    pub async fn is_empty(&self) -> bool {
        let data = self.data.lock().await;
        data.is_empty()
    }

    /// Get the raw data (for testing purposes)
    pub async fn get_raw_data(&self) -> HashMap<String, Vec<u8>> {
        let data = self.data.lock().await;
        data.iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    /// Clean expired values (this is automatically called on operations)
    async fn clean_expired(&self) {
        let mut data = self.data.lock().await;
        let now = Instant::now();

        data.retain(|_, value| match value.expires_at {
            Some(expires) => expires > now,
            None => true,
        });
    }
}

#[async_trait]
impl Cache for MockRedis {
    async fn health_check(&self) -> CacheResult<()> {
        // Always healthy in mock
        Ok(())
    }
}

#[async_trait]
impl CacheOperations for MockRedis {
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        self.clean_expired().await;

        let formatted_key = self.format_key(key.as_ref());
        let data = self.data.lock().await;

        match data.get(&formatted_key) {
            Some(cached) => {
                // Check if expired
                if let Some(expires) = cached.expires_at {
                    if expires <= Instant::now() {
                        return Ok(None);
                    }
                }

                // Deserialize
                match serde_json::from_slice(&cached.value) {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => Err(navius_cache::CacheError::serialization_error(format!(
                        "Failed to deserialize value: {}",
                        e
                    ))),
                }
            }
            None => Ok(None),
        }
    }

    async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>
    where
        K: CacheKey,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.clean_expired().await;

        let formatted_key = self.format_key(key.as_ref());

        // Serialize the value
        let serialized = serde_json::to_vec(value).map_err(|e| {
            navius_cache::CacheError::serialization_error(format!(
                "Failed to serialize value: {}",
                e
            ))
        })?;

        // Calculate expiry time
        let expires_at = options
            .and_then(|opt| opt.ttl)
            .map(|ttl| Instant::now() + ttl)
            .or_else(|| Some(Instant::now() + self.default_ttl));

        // Store the value
        let cached = CachedValue {
            value: serialized,
            expires_at,
        };

        let mut data = self.data.lock().await;
        data.insert(formatted_key, cached);

        Ok(())
    }

    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey,
    {
        self.clean_expired().await;

        let formatted_key = self.format_key(key.as_ref());
        let mut data = self.data.lock().await;

        Ok(data.remove(&formatted_key).is_some())
    }

    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey,
    {
        self.clean_expired().await;

        let formatted_key = self.format_key(key.as_ref());
        let data = self.data.lock().await;

        match data.get(&formatted_key) {
            Some(cached) => {
                // Check if expired
                if let Some(expires) = cached.expires_at {
                    if expires <= Instant::now() {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey,
    {
        self.clean_expired().await;

        let formatted_key = self.format_key(key.as_ref());
        let mut data = self.data.lock().await;

        match data.get_mut(&formatted_key) {
            Some(cached) => {
                cached.expires_at = Some(Instant::now() + ttl);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn ttl<K>(&self, key: K) -> CacheResult<Option<i64>>
    where
        K: CacheKey,
    {
        self.clean_expired().await;

        let formatted_key = self.format_key(key.as_ref());
        let data = self.data.lock().await;

        match data.get(&formatted_key) {
            Some(cached) => {
                match cached.expires_at {
                    Some(expires) => {
                        let now = Instant::now();
                        if expires <= now {
                            return Ok(Some(0));
                        }

                        let remaining = expires.duration_since(now);
                        Ok(Some(remaining.as_secs() as i64))
                    }
                    None => Ok(None), // No expiration
                }
            }
            None => Ok(None),
        }
    }

    async fn increment<K>(&self, key: K, value: i64) -> CacheResult<i64>
    where
        K: CacheKey,
    {
        self.clean_expired().await;

        let formatted_key = self.format_key(key.as_ref());
        let mut data = self.data.lock().await;

        let current: i64 = match data.get(&formatted_key) {
            Some(cached) => {
                // Check if expired
                if let Some(expires) = cached.expires_at {
                    if expires <= Instant::now() {
                        0
                    } else {
                        // Deserialize
                        match serde_json::from_slice(&cached.value) {
                            Ok(value) => value,
                            Err(_) => 0,
                        }
                    }
                } else {
                    // No expiration
                    match serde_json::from_slice(&cached.value) {
                        Ok(value) => value,
                        Err(_) => 0,
                    }
                }
            }
            None => 0,
        };

        let new_value = current + value;

        // Serialize the new value
        let serialized = serde_json::to_vec(&new_value).map_err(|e| {
            navius_cache::CacheError::serialization_error(format!(
                "Failed to serialize value: {}",
                e
            ))
        })?;

        // Get expiration time (keep the existing one if possible)
        let expires_at = data
            .get(&formatted_key)
            .and_then(|cached| cached.expires_at)
            .or_else(|| Some(Instant::now() + self.default_ttl));

        // Store the new value
        let cached = CachedValue {
            value: serialized,
            expires_at,
        };

        data.insert(formatted_key, cached);

        Ok(new_value)
    }

    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        self.clean_expired().await;

        let mut results = Vec::with_capacity(keys.len());

        for key in keys {
            let result = self.get(key).await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<CacheOptions>,
    ) -> CacheResult<()>
    where
        K: CacheKey,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.clean_expired().await;

        for (key, value) in entries {
            self.set(key, &value, options.clone()).await?;
        }

        Ok(())
    }

    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey,
    {
        self.clean_expired().await;

        let mut count = 0;

        for key in keys {
            if self.delete(key).await? {
                count += 1;
            }
        }

        Ok(count)
    }

    // Implement some basic list operations
    async fn list_len<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey,
    {
        self.clean_expired().await;

        let formatted_key = self.format_key(key.as_ref());
        let data = self.data.lock().await;

        match data.get(&formatted_key) {
            Some(cached) => {
                // Check if expired
                if let Some(expires) = cached.expires_at {
                    if expires <= Instant::now() {
                        return Ok(0);
                    }
                }

                // Deserialize as Vec<serde_json::Value>
                match serde_json::from_slice::<Vec<serde_json::Value>>(&cached.value) {
                    Ok(values) => Ok(values.len()),
                    Err(_) => Ok(0), // Not a list
                }
            }
            None => Ok(0),
        }
    }

    async fn list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>
    where
        K: CacheKey,
        V: serde::Serialize + Send + Sync + 'static,
    {
        self.clean_expired().await;

        let formatted_key = self.format_key(key.as_ref());
        let mut data = self.data.lock().await;

        // Serialize new value
        let value_json = serde_json::to_value(value).map_err(|e| {
            navius_cache::CacheError::serialization_error(format!(
                "Failed to serialize value: {}",
                e
            ))
        })?;

        // Get current list
        let mut list = match data.get(&formatted_key) {
            Some(cached) => {
                // Check if expired
                if let Some(expires) = cached.expires_at {
                    if expires <= Instant::now() {
                        vec![]
                    } else {
                        // Deserialize as Vec<serde_json::Value>
                        match serde_json::from_slice::<Vec<serde_json::Value>>(&cached.value) {
                            Ok(values) => values,
                            Err(_) => vec![], // Not a list, start fresh
                        }
                    }
                } else {
                    // No expiration
                    match serde_json::from_slice::<Vec<serde_json::Value>>(&cached.value) {
                        Ok(values) => values,
                        Err(_) => vec![], // Not a list, start fresh
                    }
                }
            }
            None => vec![],
        };

        // Add new value
        list.push(value_json);

        // Get expiration time (keep the existing one if possible)
        let expires_at = data
            .get(&formatted_key)
            .and_then(|cached| cached.expires_at)
            .or_else(|| Some(Instant::now() + self.default_ttl));

        // Serialize the list
        let serialized = serde_json::to_vec(&list).map_err(|e| {
            navius_cache::CacheError::serialization_error(format!(
                "Failed to serialize list: {}",
                e
            ))
        })?;

        // Store the updated list
        let cached = CachedValue {
            value: serialized,
            expires_at,
        };

        data.insert(formatted_key, cached);

        Ok(list.len())
    }

    async fn list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        self.clean_expired().await;

        let formatted_key = self.format_key(key.as_ref());
        let mut data = self.data.lock().await;

        match data.get(&formatted_key) {
            Some(cached) => {
                // Check if expired
                if let Some(expires) = cached.expires_at {
                    if expires <= Instant::now() {
                        return Ok(None);
                    }
                }

                // Deserialize as Vec<serde_json::Value>
                match serde_json::from_slice::<Vec<serde_json::Value>>(&cached.value) {
                    Ok(mut values) => {
                        if values.is_empty() {
                            return Ok(None);
                        }

                        // Pop the first element
                        let popped = values.remove(0);

                        // Deserialize the popped value
                        let result = serde_json::from_value(popped).map_err(|e| {
                            navius_cache::CacheError::serialization_error(format!(
                                "Failed to deserialize popped value: {}",
                                e
                            ))
                        })?;

                        // Get expiration time (keep the existing one)
                        let expires_at = cached.expires_at;

                        // Serialize the updated list
                        let serialized = serde_json::to_vec(&values).map_err(|e| {
                            navius_cache::CacheError::serialization_error(format!(
                                "Failed to serialize updated list: {}",
                                e
                            ))
                        })?;

                        // Store the updated list
                        let new_cached = CachedValue {
                            value: serialized,
                            expires_at,
                        };

                        data.insert(formatted_key, new_cached);

                        Ok(Some(result))
                    }
                    Err(_) => Ok(None), // Not a list
                }
            }
            None => Ok(None),
        }
    }

    async fn list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>
    where
        K: CacheKey,
        V: serde::de::DeserializeOwned + Send + 'static,
    {
        self.clean_expired().await;

        let formatted_key = self.format_key(key.as_ref());
        let data = self.data.lock().await;

        match data.get(&formatted_key) {
            Some(cached) => {
                // Check if expired
                if let Some(expires) = cached.expires_at {
                    if expires <= Instant::now() {
                        return Ok(vec![]);
                    }
                }

                // Deserialize as Vec<serde_json::Value>
                match serde_json::from_slice::<Vec<serde_json::Value>>(&cached.value) {
                    Ok(values) => {
                        let len = values.len() as isize;

                        // Calculate start and stop indices
                        let start_idx = if start < 0 {
                            (len + start).max(0) as usize
                        } else {
                            start as usize
                        };

                        let stop_idx = if stop < 0 {
                            (len + stop + 1).max(0) as usize
                        } else {
                            (stop + 1).min(len) as usize
                        };

                        if start_idx >= values.len() || start_idx >= stop_idx {
                            return Ok(vec![]);
                        }

                        // Get the range
                        let range = &values[start_idx..stop_idx];

                        // Deserialize each value in the range
                        let mut results = Vec::with_capacity(range.len());
                        for value in range {
                            let result = serde_json::from_value(value.clone()).map_err(|e| {
                                navius_cache::CacheError::serialization_error(format!(
                                    "Failed to deserialize list value: {}",
                                    e
                                ))
                            })?;
                            results.push(result);
                        }

                        Ok(results)
                    }
                    Err(_) => Ok(vec![]), // Not a list
                }
            }
            None => Ok(vec![]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_redis_basic_operations() {
        let mock = MockRedis::new("test:".to_string(), Duration::from_secs(60));

        // Test basic set/get
        mock.set("key1", &"value1", None).await.unwrap();
        let value: Option<String> = mock.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Test exists
        let exists = mock.exists("key1").await.unwrap();
        assert!(exists);

        // Test non-existent key
        let value: Option<String> = mock.get("nonexistent").await.unwrap();
        assert_eq!(value, None);

        // Test delete
        let deleted = mock.delete("key1").await.unwrap();
        assert!(deleted);

        // Test after delete
        let exists = mock.exists("key1").await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_mock_redis_ttl() {
        let mock = MockRedis::new("test:".to_string(), Duration::from_secs(60));

        // Set with TTL
        let options = CacheOptions {
            ttl: Some(Duration::from_secs(30)),
        };
        mock.set("key_with_ttl", &"value", Some(options))
            .await
            .unwrap();

        // Check TTL
        let ttl = mock.ttl("key_with_ttl").await.unwrap();
        assert!(ttl.is_some());

        let ttl_value = ttl.unwrap();
        assert!(ttl_value > 0 && ttl_value <= 30);

        // Set custom TTL
        mock.expire("key_with_ttl", Duration::from_secs(10))
            .await
            .unwrap();

        // Check updated TTL
        let ttl = mock.ttl("key_with_ttl").await.unwrap();
        assert!(ttl.is_some());

        let ttl_value = ttl.unwrap();
        assert!(ttl_value > 0 && ttl_value <= 10);
    }

    #[tokio::test]
    async fn test_mock_redis_expiration() {
        let mock = MockRedis::new("test:".to_string(), Duration::from_secs(60));

        // Set with short TTL
        let options = CacheOptions {
            ttl: Some(Duration::from_millis(100)), // 100ms
        };
        mock.set("short_lived", &"value", Some(options))
            .await
            .unwrap();

        // Verify it exists
        let exists = mock.exists("short_lived").await.unwrap();
        assert!(exists);

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Check that it's gone
        let exists = mock.exists("short_lived").await.unwrap();
        assert!(!exists);

        let value: Option<String> = mock.get("short_lived").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_mock_redis_increment() {
        let mock = MockRedis::new("test:".to_string(), Duration::from_secs(60));

        // Increment a new key
        let count = mock.increment("counter", 1).await.unwrap();
        assert_eq!(count, 1);

        // Increment again
        let count = mock.increment("counter", 5).await.unwrap();
        assert_eq!(count, 6);

        // Get the value
        let value: Option<i64> = mock.get("counter").await.unwrap();
        assert_eq!(value, Some(6));
    }

    #[tokio::test]
    async fn test_mock_redis_list_operations() {
        let mock = MockRedis::new("test:".to_string(), Duration::from_secs(60));

        // Push values to list
        mock.list_push_right("mylist", &"value1").await.unwrap();
        mock.list_push_right("mylist", &"value2").await.unwrap();
        mock.list_push_right("mylist", &"value3").await.unwrap();

        // Check list length
        let len = mock.list_len("mylist").await.unwrap();
        assert_eq!(len, 3);

        // Get range
        let values: Vec<String> = mock.list_range("mylist", 0, -1).await.unwrap();
        assert_eq!(
            values,
            vec![
                "value1".to_string(),
                "value2".to_string(),
                "value3".to_string()
            ]
        );

        // Get partial range
        let values: Vec<String> = mock.list_range("mylist", 1, 2).await.unwrap();
        assert_eq!(values, vec!["value2".to_string()]);

        // Pop left
        let value: Option<String> = mock.list_pop_left("mylist").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Check updated list
        let values: Vec<String> = mock.list_range("mylist", 0, -1).await.unwrap();
        assert_eq!(values, vec!["value2".to_string(), "value3".to_string()]);
    }
}
