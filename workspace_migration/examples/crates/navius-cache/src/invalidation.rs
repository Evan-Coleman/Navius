use crate::error::{CacheError, CacheResult};
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashSet;
use std::time::Duration;
use tracing::{debug, instrument};

/// Cache invalidation strategy interface
#[async_trait]
pub trait CacheInvalidator: Send + Sync {
    /// Invalidate a specific key
    #[instrument(skip(self), level = "debug")]
    async fn invalidate_key(&self, key: &str) -> CacheResult<bool>;

    /// Invalidate multiple keys
    #[instrument(skip(self), level = "debug")]
    async fn invalidate_keys(&self, keys: &[&str]) -> CacheResult<u64>;

    /// Invalidate cache entries by pattern
    #[instrument(skip(self), level = "debug")]
    async fn invalidate_by_pattern(&self, pattern: &str) -> CacheResult<u64>;

    /// Invalidate all cache entries
    #[instrument(skip(self), level = "debug")]
    async fn invalidate_all(&self) -> CacheResult<bool>;

    /// Associate cache tags with a key
    #[instrument(skip(self), level = "debug")]
    async fn tag_key(&self, key: &str, tags: &[&str]) -> CacheResult<()>;

    /// Get all tags for a key
    #[instrument(skip(self), level = "debug")]
    async fn get_key_tags(&self, key: &str) -> CacheResult<HashSet<String>>;

    /// Invalidate all keys with a specific tag
    #[instrument(skip(self), level = "debug")]
    async fn invalidate_by_tag(&self, tag: &str) -> CacheResult<u64>;

    /// Invalidate all keys with specific tags (any of the tags)
    #[instrument(skip(self), level = "debug")]
    async fn invalidate_by_tags(&self, tags: &[&str]) -> CacheResult<u64>;
}

/// Cache TTL management interface
#[async_trait]
pub trait CacheTtlManager: Send + Sync {
    /// Set TTL for a key
    #[instrument(skip(self), level = "debug")]
    async fn set_ttl(&self, key: &str, ttl: Duration) -> CacheResult<bool>;

    /// Get TTL for a key
    #[instrument(skip(self), level = "debug")]
    async fn get_ttl(&self, key: &str) -> CacheResult<Option<Duration>>;

    /// Remove TTL for a key
    #[instrument(skip(self), level = "debug")]
    async fn remove_ttl(&self, key: &str) -> CacheResult<bool>;
}

/// Event-based cache invalidation strategy
#[async_trait]
pub trait CacheEventInvalidator: Send + Sync {
    /// Subscribe to cache invalidation events
    async fn subscribe_to_invalidation_events(&self) -> CacheResult<()>;

    /// Publish a cache invalidation event
    async fn publish_invalidation_event(&self, event_type: &str, payload: &str) -> CacheResult<()>;

    /// Process a cache invalidation event
    async fn process_invalidation_event(&self, event_type: &str, payload: &str) -> CacheResult<()>;
}

/// Cache entity change tracking
#[async_trait]
pub trait CacheEntityTracker: Send + Sync {
    /// Track a change to an entity
    async fn track_entity_change<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        entity_type: &str,
        entity_id: &str,
        entity: &T,
    ) -> CacheResult<()>;

    /// Get all related cache keys for an entity
    async fn get_entity_cache_keys(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> CacheResult<Vec<String>>;

    /// Invalidate all cache entries related to an entity
    async fn invalidate_entity(&self, entity_type: &str, entity_id: &str) -> CacheResult<u64>;
}

/// Implements a simple multi-strategy invalidator that combines multiple invalidation strategies
pub struct CompositeInvalidator<T: CacheInvalidator> {
    invalidators: Vec<T>,
}

impl<T: CacheInvalidator> CompositeInvalidator<T> {
    /// Create a new composite invalidator with the specified invalidators
    pub fn new(invalidators: Vec<T>) -> Self {
        Self { invalidators }
    }
}

#[async_trait]
impl<T: CacheInvalidator> CacheInvalidator for CompositeInvalidator<T> {
    async fn invalidate_key(&self, key: &str) -> CacheResult<bool> {
        let mut success = true;
        for invalidator in &self.invalidators {
            success = success && invalidator.invalidate_key(key).await?;
        }
        Ok(success)
    }

    async fn invalidate_keys(&self, keys: &[&str]) -> CacheResult<u64> {
        let mut total = 0;
        for invalidator in &self.invalidators {
            total += invalidator.invalidate_keys(keys).await?;
        }
        Ok(total)
    }

    async fn invalidate_by_pattern(&self, pattern: &str) -> CacheResult<u64> {
        let mut total = 0;
        for invalidator in &self.invalidators {
            total += invalidator.invalidate_by_pattern(pattern).await?;
        }
        Ok(total)
    }

    async fn invalidate_all(&self) -> CacheResult<bool> {
        let mut success = true;
        for invalidator in &self.invalidators {
            success = success && invalidator.invalidate_all().await?;
        }
        Ok(success)
    }

    async fn tag_key(&self, key: &str, tags: &[&str]) -> CacheResult<()> {
        for invalidator in &self.invalidators {
            invalidator.tag_key(key, tags).await?;
        }
        Ok(())
    }

    async fn get_key_tags(&self, key: &str) -> CacheResult<HashSet<String>> {
        let mut all_tags = HashSet::new();
        for invalidator in &self.invalidators {
            let tags = invalidator.get_key_tags(key).await?;
            all_tags.extend(tags);
        }
        Ok(all_tags)
    }

    async fn invalidate_by_tag(&self, tag: &str) -> CacheResult<u64> {
        let mut total = 0;
        for invalidator in &self.invalidators {
            total += invalidator.invalidate_by_tag(tag).await?;
        }
        Ok(total)
    }

    async fn invalidate_by_tags(&self, tags: &[&str]) -> CacheResult<u64> {
        let mut total = 0;
        for invalidator in &self.invalidators {
            total += invalidator.invalidate_by_tags(tags).await?;
        }
        Ok(total)
    }
}

/// A mock implementation of the CacheInvalidator for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;

    /// Mock cache invalidator for testing
    pub struct MockInvalidator {
        keys: RwLock<HashMap<String, HashSet<String>>>,
    }

    impl MockInvalidator {
        /// Create a new mock invalidator
        pub fn new() -> Self {
            Self {
                keys: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl CacheInvalidator for MockInvalidator {
        async fn invalidate_key(&self, key: &str) -> CacheResult<bool> {
            let mut keys = self.keys.write().unwrap();
            let removed = keys.remove(key).is_some();
            Ok(removed)
        }

        async fn invalidate_keys(&self, keys: &[&str]) -> CacheResult<u64> {
            let mut count = 0;
            let mut cache_keys = self.keys.write().unwrap();
            for key in keys {
                if cache_keys.remove(*key).is_some() {
                    count += 1;
                }
            }
            Ok(count)
        }

        async fn invalidate_by_pattern(&self, pattern: &str) -> CacheResult<u64> {
            let mut count = 0;
            let mut cache_keys = self.keys.write().unwrap();
            let keys_to_remove: Vec<String> = cache_keys
                .keys()
                .filter(|k| k.contains(pattern))
                .cloned()
                .collect();

            for key in keys_to_remove {
                cache_keys.remove(&key);
                count += 1;
            }

            Ok(count)
        }

        async fn invalidate_all(&self) -> CacheResult<bool> {
            let mut keys = self.keys.write().unwrap();
            keys.clear();
            Ok(true)
        }

        async fn tag_key(&self, key: &str, tags: &[&str]) -> CacheResult<()> {
            let mut keys = self.keys.write().unwrap();
            let entry = keys.entry(key.to_string()).or_insert_with(HashSet::new);
            for tag in tags {
                entry.insert(tag.to_string());
            }
            Ok(())
        }

        async fn get_key_tags(&self, key: &str) -> CacheResult<HashSet<String>> {
            let keys = self.keys.read().unwrap();
            match keys.get(key) {
                Some(tags) => Ok(tags.clone()),
                None => Ok(HashSet::new()),
            }
        }

        async fn invalidate_by_tag(&self, tag: &str) -> CacheResult<u64> {
            let mut count = 0;
            let mut keys = self.keys.write().unwrap();
            let keys_to_remove: Vec<String> = keys
                .iter()
                .filter(|(_, tags)| tags.contains(tag))
                .map(|(k, _)| k.clone())
                .collect();

            for key in keys_to_remove {
                keys.remove(&key);
                count += 1;
            }

            Ok(count)
        }

        async fn invalidate_by_tags(&self, tags: &[&str]) -> CacheResult<u64> {
            let mut count = 0;
            let mut keys = self.keys.write().unwrap();
            let tag_set: HashSet<String> = tags.iter().map(|t| t.to_string()).collect();

            let keys_to_remove: Vec<String> = keys
                .iter()
                .filter(|(_, key_tags)| key_tags.iter().any(|t| tag_set.contains(t)))
                .map(|(k, _)| k.clone())
                .collect();

            for key in keys_to_remove {
                keys.remove(&key);
                count += 1;
            }

            Ok(count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockInvalidator;
    use super::*;

    #[tokio::test]
    async fn test_mock_invalidator_basic() {
        let invalidator = MockInvalidator::new();

        // Add tags to keys
        invalidator
            .tag_key("user:1", &["user", "entity"])
            .await
            .unwrap();
        invalidator
            .tag_key("user:2", &["user", "entity"])
            .await
            .unwrap();
        invalidator
            .tag_key("product:1", &["product", "entity"])
            .await
            .unwrap();

        // Verify tags
        let user_tags = invalidator.get_key_tags("user:1").await.unwrap();
        assert!(user_tags.contains("user"));
        assert!(user_tags.contains("entity"));

        // Invalidate by tag
        let count = invalidator.invalidate_by_tag("user").await.unwrap();
        assert_eq!(count, 2);

        // Verify keys were invalidated
        let remaining_tags = invalidator.get_key_tags("user:1").await.unwrap();
        assert!(remaining_tags.is_empty());

        let product_tags = invalidator.get_key_tags("product:1").await.unwrap();
        assert!(!product_tags.is_empty());
    }

    #[tokio::test]
    async fn test_mock_invalidator_pattern() {
        let invalidator = MockInvalidator::new();

        // Add tags to keys
        invalidator.tag_key("user:1", &["user"]).await.unwrap();
        invalidator.tag_key("user:2", &["user"]).await.unwrap();
        invalidator.tag_key("user:3", &["user"]).await.unwrap();
        invalidator
            .tag_key("product:1", &["product"])
            .await
            .unwrap();

        // Invalidate by pattern
        let count = invalidator.invalidate_by_pattern("user:").await.unwrap();
        assert_eq!(count, 3);

        // Verify product still exists
        let product_tags = invalidator.get_key_tags("product:1").await.unwrap();
        assert!(!product_tags.is_empty());
    }

    #[tokio::test]
    async fn test_composite_invalidator() {
        let invalidator1 = MockInvalidator::new();
        let invalidator2 = MockInvalidator::new();

        let composite = CompositeInvalidator::new(vec![invalidator1, invalidator2]);

        // Add tags to keys in both invalidators
        composite
            .tag_key("user:1", &["user", "entity"])
            .await
            .unwrap();
        composite
            .tag_key("product:1", &["product", "entity"])
            .await
            .unwrap();

        // Invalidate by tag
        let count = composite.invalidate_by_tag("user").await.unwrap();
        assert_eq!(count, 2); // Counts from both invalidators

        // Invalidate all
        let success = composite.invalidate_all().await.unwrap();
        assert!(success);
    }
}
