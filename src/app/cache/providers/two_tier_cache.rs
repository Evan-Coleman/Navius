use crate::core::error::Result;
use crate::core::services::cache_provider::{
    CacheKey, CacheOperations, DynCacheOperations, TypedCache, TypedCacheFactory,
};
use bincode::{Decode, Encode};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, instrument, trace};

/// TwoTierCache implements a fallback strategy with a fast primary cache and a slower but more
/// persistent secondary cache.
///
/// Read operations:
///   1. First try to get from the fast cache
///   2. If not found, try to get from the slow cache
///   3. If found in slow cache, promote to fast cache
///
/// Write operations:
///   1. Write to both caches simultaneously
///
/// This strategy provides the best of both worlds:
///   - Fast reads from the memory cache
///   - Persistence from the Redis/disk cache
///   - Automatic promotion of frequently used items
pub struct TwoTierCache {
    name: String,
    fast_cache: Box<dyn DynCacheOperations>,
    slow_cache: Box<dyn DynCacheOperations>,
}

impl TwoTierCache {
    /// Create a new two-tier cache with the given fast and slow cache implementations
    pub fn new(
        name: String,
        fast_cache: Box<dyn DynCacheOperations>,
        slow_cache: Box<dyn DynCacheOperations>,
    ) -> Self {
        Self {
            name,
            fast_cache,
            slow_cache,
        }
    }

    /// Promote an item from the slow cache to the fast cache
    async fn promote<T: AsRef<[u8]>>(&self, key: &str, value: T) {
        if let Err(err) = self.fast_cache.set(key, value, None).await {
            error!(
                cache = self.name,
                key = key,
                error = ?err,
                "Failed to promote item from slow to fast cache"
            );
        } else {
            trace!(
                cache = self.name,
                key = key,
                "Promoted item from slow to fast cache"
            );
        }
    }
}

impl CacheOperations for TwoTierCache {
    fn name(&self) -> &str {
        &self.name
    }

    #[instrument(skip(self, value), level = "debug")]
    async fn set<T: AsRef<[u8]>>(
        &self,
        key: &str,
        value: T,
        ttl_seconds: Option<u64>,
    ) -> Result<()> {
        // Write to both caches simultaneously
        let value_bytes = value.as_ref().to_vec();

        // We use tokio::join! to run both operations concurrently
        let (fast_result, slow_result) = tokio::join!(
            self.fast_cache.set(key, value_bytes.clone(), ttl_seconds),
            self.slow_cache.set(key, value_bytes, ttl_seconds)
        );

        // Report errors but only fail if both operations fail
        if let Err(err) = &fast_result {
            error!(
                cache = self.name,
                key = key,
                error = ?err,
                "Failed to set item in fast cache"
            );
        }

        if let Err(err) = &slow_result {
            error!(
                cache = self.name,
                key = key,
                error = ?err,
                "Failed to set item in slow cache"
            );
        }

        // Only return error if both failed
        if fast_result.is_err() && slow_result.is_err() {
            Err(slow_result.unwrap_err()) // Return the slow cache error for better context
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Try to get from fast cache first
        match self.fast_cache.get(key).await {
            Ok(Some(value)) => {
                trace!(cache = self.name, key = key, "Cache hit (fast)");
                return Ok(Some(value));
            }
            Ok(None) => {
                trace!(cache = self.name, key = key, "Cache miss (fast)");
                // Fall through to slow cache
            }
            Err(err) => {
                debug!(
                    cache = self.name,
                    key = key,
                    error = ?err,
                    "Error getting item from fast cache, falling back to slow cache"
                );
                // Fall through to slow cache
            }
        }

        // Try to get from slow cache
        match self.slow_cache.get(key).await {
            Ok(Some(value)) => {
                trace!(cache = self.name, key = key, "Cache hit (slow)");

                // Promote to fast cache
                let value_to_promote = value.clone();
                self.promote(key, value_to_promote).await;

                Ok(Some(value))
            }
            Ok(None) => {
                trace!(cache = self.name, key = key, "Cache miss (slow)");
                Ok(None)
            }
            Err(err) => {
                debug!(
                    cache = self.name,
                    key = key,
                    error = ?err,
                    "Error getting item from slow cache"
                );
                Err(err)
            }
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn delete(&self, key: &str) -> Result<bool> {
        // Delete from both caches
        let (fast_result, slow_result) =
            tokio::join!(self.fast_cache.delete(key), self.slow_cache.delete(key));

        // Report errors
        if let Err(err) = &fast_result {
            error!(
                cache = self.name,
                key = key,
                error = ?err,
                "Failed to delete item from fast cache"
            );
        }

        if let Err(err) = &slow_result {
            error!(
                cache = self.name,
                key = key,
                error = ?err,
                "Failed to delete item from slow cache"
            );
        }

        // Return success if either operation succeeded
        if fast_result.is_ok() || slow_result.is_ok() {
            Ok(fast_result.unwrap_or(false) || slow_result.unwrap_or(false))
        } else {
            Err(slow_result.unwrap_err())
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn clear(&self) -> Result<()> {
        // Clear both caches
        let (fast_result, slow_result) =
            tokio::join!(self.fast_cache.clear(), self.slow_cache.clear());

        // Report errors
        if let Err(err) = &fast_result {
            error!(
                cache = self.name,
                error = ?err,
                "Failed to clear fast cache"
            );
        }

        if let Err(err) = &slow_result {
            error!(
                cache = self.name,
                error = ?err,
                "Failed to clear slow cache"
            );
        }

        // Only return error if both failed
        if fast_result.is_err() && slow_result.is_err() {
            Err(slow_result.unwrap_err())
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn get_many(&self, keys: &[&str]) -> Result<HashMap<String, Vec<u8>>> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        // Try to get from fast cache first
        let mut result = match self.fast_cache.get_many(keys).await {
            Ok(fast_results) => fast_results,
            Err(err) => {
                debug!(
                    cache = self.name,
                    error = ?err,
                    "Error getting items from fast cache, falling back to slow cache for all keys"
                );
                HashMap::new()
            }
        };

        // Calculate missing keys
        let missing_keys: Vec<&str> = keys
            .iter()
            .filter(|&key| !result.contains_key(*key))
            .copied()
            .collect();

        if !missing_keys.is_empty() {
            trace!(
                cache = self.name,
                missing_count = missing_keys.len(),
                "Fetching missing keys from slow cache"
            );

            // Get missing keys from slow cache
            match self.slow_cache.get_many(&missing_keys).await {
                Ok(slow_results) => {
                    // Promote all found items to fast cache
                    for (key, value) in &slow_results {
                        let value_to_promote = value.clone();
                        self.promote(key, value_to_promote).await;
                    }

                    // Merge results
                    result.extend(slow_results);
                }
                Err(err) => {
                    debug!(
                        cache = self.name,
                        error = ?err,
                        "Error getting items from slow cache"
                    );
                    // Continue with what we have from fast cache
                }
            }
        }

        Ok(result)
    }
}

impl DynCacheOperations for TwoTierCache {
    fn get_typed_cache<T>(&self) -> Box<dyn TypedCache<T>>
    where
        T: Encode + Decode<()> + Send + Sync + 'static,
    {
        Box::new(TwoTierTypedCache { cache: self })
    }
}

/// TypedCache implementation for the TwoTierCache
pub struct TwoTierTypedCache<'a, T>
where
    T: Encode + Decode<()> + Send + Sync + 'static,
{
    cache: &'a TwoTierCache,
}

impl<'a, T> TypedCache<T> for TwoTierTypedCache<'a, T>
where
    T: Encode + Decode<()> + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        self.cache.name()
    }

    async fn get(&self, key: &str) -> Result<Option<T>> {
        match self.cache.get(key).await? {
            Some(bytes) => {
                let config = bincode::config::standard();
                match bincode::decode_from_slice::<T, ()>(&bytes, config) {
                    Ok((value, _)) => Ok(Some(value)),
                    Err(err) => {
                        error!(
                            cache = self.name(),
                            key = key,
                            error = ?err,
                            "Failed to decode value from cache"
                        );
                        Ok(None)
                    }
                }
            }
            None => Ok(None),
        }
    }

    async fn set(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<()> {
        let config = bincode::config::standard();
        let bytes = bincode::encode_to_vec(value, config)?;
        self.cache.set(key, bytes, ttl_seconds).await
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        self.cache.delete(key).await
    }

    async fn clear(&self) -> Result<()> {
        self.cache.clear().await
    }

    async fn get_many(&self, keys: &[&str]) -> Result<HashMap<String, T>> {
        let bytes_result = self.cache.get_many(keys).await?;

        let mut result = HashMap::with_capacity(bytes_result.len());
        let config = bincode::config::standard();

        for (key, bytes) in bytes_result {
            match bincode::decode_from_slice::<T, ()>(&bytes, config) {
                Ok((value, _)) => {
                    result.insert(key, value);
                }
                Err(err) => {
                    error!(
                        cache = self.name(),
                        key = key,
                        error = ?err,
                        "Failed to decode value from cache"
                    );
                    // Skip this key
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::services::memory_cache::InMemoryCache;
    use std::time::Duration;

    #[derive(Debug, PartialEq, Clone, Encode, Decode)]
    struct TestData {
        id: u32,
        name: String,
    }

    fn create_test_caches() -> (Box<dyn DynCacheOperations>, Box<dyn DynCacheOperations>) {
        let fast_cache = Box::new(InMemoryCache::new("fast_test", Some(100), None));
        let slow_cache = Box::new(InMemoryCache::new("slow_test", Some(1000), None));
        (fast_cache, slow_cache)
    }

    #[tokio::test]
    async fn test_get_set_basic() {
        let (fast_cache, slow_cache) = create_test_caches();
        let two_tier = TwoTierCache::new("test_cache".to_string(), fast_cache, slow_cache);

        // Set a value in the cache
        let key = "test_key";
        let value = b"test_value".to_vec();
        two_tier.set(key, &value, None).await.unwrap();

        // Get the value from the cache
        let result = two_tier.get(key).await.unwrap();
        assert_eq!(result, Some(value));
    }

    #[tokio::test]
    async fn test_typed_cache() {
        let (fast_cache, slow_cache) = create_test_caches();
        let two_tier = TwoTierCache::new("test_typed_cache".to_string(), fast_cache, slow_cache);

        let typed_cache = two_tier.get_typed_cache::<TestData>();

        // Set a value in the cache
        let key = "test_key";
        let value = TestData {
            id: 123,
            name: "Test".to_string(),
        };

        typed_cache.set(key, &value, None).await.unwrap();

        // Get the value from the cache
        let result = typed_cache.get(key).await.unwrap();
        assert_eq!(result, Some(value));
    }

    #[tokio::test]
    async fn test_promotion_between_caches() {
        let (fast_cache, slow_cache) = create_test_caches();
        let two_tier = TwoTierCache::new(
            "test_promotion".to_string(),
            fast_cache.clone(),
            slow_cache.clone(),
        );

        // Set a value directly in the slow cache, bypassing the two-tier interface
        let key = "promotion_key";
        let value = b"promotion_value".to_vec();
        slow_cache.set(key, &value, None).await.unwrap();

        // Verify it's not in the fast cache yet
        let fast_result = fast_cache.get(key).await.unwrap();
        assert_eq!(fast_result, None);

        // Access through the two-tier cache, which should trigger promotion
        let result = two_tier.get(key).await.unwrap();
        assert_eq!(result, Some(value.clone()));

        // Now check it's been promoted to the fast cache
        let fast_result = fast_cache.get(key).await.unwrap();
        assert_eq!(fast_result, Some(value));
    }

    #[tokio::test]
    async fn test_get_many() {
        let (fast_cache, slow_cache) = create_test_caches();
        let two_tier = TwoTierCache::new(
            "test_get_many".to_string(),
            fast_cache.clone(),
            slow_cache.clone(),
        );

        // Set some values in different caches
        let key1 = "key1";
        let value1 = b"value1".to_vec();
        let key2 = "key2";
        let value2 = b"value2".to_vec();

        // key1 in fast cache
        fast_cache.set(key1, &value1, None).await.unwrap();

        // key2 in slow cache
        slow_cache.set(key2, &value2, None).await.unwrap();

        // Get both keys in one operation
        let keys = vec![key1, key2];
        let results = two_tier.get_many(&keys).await.unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results.get(key1), Some(&value1));
        assert_eq!(results.get(key2), Some(&value2));

        // Verify key2 was promoted to fast cache
        let fast_result = fast_cache.get(key2).await.unwrap();
        assert_eq!(fast_result, Some(value2));
    }

    #[tokio::test]
    async fn test_delete() {
        let (fast_cache, slow_cache) = create_test_caches();
        let two_tier = TwoTierCache::new(
            "test_delete".to_string(),
            fast_cache.clone(),
            slow_cache.clone(),
        );

        // Set a value in both caches
        let key = "delete_key";
        let value = b"delete_value".to_vec();
        two_tier.set(key, &value, None).await.unwrap();

        // Verify it's in both caches
        assert!(fast_cache.get(key).await.unwrap().is_some());
        assert!(slow_cache.get(key).await.unwrap().is_some());

        // Delete the key
        let deleted = two_tier.delete(key).await.unwrap();
        assert!(deleted);

        // Verify it's deleted from both caches
        assert!(fast_cache.get(key).await.unwrap().is_none());
        assert!(slow_cache.get(key).await.unwrap().is_none());
    }
}
