use navius_cache::{
    CacheConfig, CacheConnectionManager, CacheInvalidation, CacheOperations, InvalidationStrategy,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashSet;

    // Mock invalidator to test custom invalidation strategies
    struct TestInvalidator {
        invalidated_keys: Arc<RwLock<Vec<String>>>,
        invalidated_patterns: Arc<RwLock<Vec<String>>>,
    }

    impl TestInvalidator {
        fn new() -> Self {
            Self {
                invalidated_keys: Arc::new(RwLock::new(Vec::new())),
                invalidated_patterns: Arc::new(RwLock::new(Vec::new())),
            }
        }

        async fn invalidated_key_count(&self) -> usize {
            self.invalidated_keys.read().await.len()
        }

        async fn invalidated_pattern_count(&self) -> usize {
            self.invalidated_patterns.read().await.len()
        }

        async fn contains_key(&self, key: &str) -> bool {
            self.invalidated_keys
                .read()
                .await
                .contains(&key.to_string())
        }

        async fn contains_pattern(&self, pattern: &str) -> bool {
            self.invalidated_patterns
                .read()
                .await
                .contains(&pattern.to_string())
        }
    }

    #[async_trait]
    trait CacheInvalidator {
        async fn invalidate_key(&self, key: &str) -> Result<(), navius_cache::error::CacheError>;
        async fn invalidate_pattern(
            &self,
            pattern: &str,
        ) -> Result<(), navius_cache::error::CacheError>;
    }

    #[async_trait]
    impl CacheInvalidator for TestInvalidator {
        async fn invalidate_key(&self, key: &str) -> Result<(), navius_cache::error::CacheError> {
            let mut keys = self.invalidated_keys.write().await;
            keys.push(key.to_string());
            Ok(())
        }

        async fn invalidate_pattern(
            &self,
            pattern: &str,
        ) -> Result<(), navius_cache::error::CacheError> {
            let mut patterns = self.invalidated_patterns.write().await;
            patterns.push(pattern.to_string());
            Ok(())
        }
    }

    // Helper function to create a unique test configuration
    fn create_test_config() -> CacheConfig {
        CacheConfig::new(
            "memory://".to_string(),
            format!("navius-test-{}:", Uuid::new_v4()),
            Duration::from_secs(60),
        )
        .with_trace(true)
    }

    // Helper function to create a test cache manager
    fn create_test_cache() -> CacheConnectionManager {
        CacheConnectionManager::new_memory(create_test_config())
    }

    #[tokio::test]
    async fn test_immediate_invalidation() {
        // Set up cache with data
        let cache = create_test_cache();
        cache.clear().await.unwrap();

        // Add test data
        cache.set("test:key1", "value1", None).await.unwrap();
        cache.set("test:key2", "value2", None).await.unwrap();

        // Verify data exists
        assert_eq!(cache.exists("test:key1").await.unwrap(), true);
        assert_eq!(cache.exists("test:key2").await.unwrap(), true);

        // Create and use invalidator for immediate deletion
        let invalidator =
            CacheInvalidation::new(InvalidationStrategy::ImmediateKey("test:key1".to_string()));
        invalidator.invalidate(&cache).await.unwrap();

        // Verify key1 is gone but key2 remains
        assert_eq!(cache.exists("test:key1").await.unwrap(), false);
        assert_eq!(cache.exists("test:key2").await.unwrap(), true);
    }

    #[tokio::test]
    async fn test_entity_invalidation() {
        // Set up cache with data
        let cache = create_test_cache();
        cache.clear().await.unwrap();

        // Add test data
        cache
            .set("user:123:profile", "profile_data", None)
            .await
            .unwrap();
        cache
            .set("user:123:settings", "settings_data", None)
            .await
            .unwrap();
        cache
            .set("user:456:profile", "other_profile", None)
            .await
            .unwrap();

        // Verify data exists
        assert_eq!(cache.exists("user:123:profile").await.unwrap(), true);
        assert_eq!(cache.exists("user:123:settings").await.unwrap(), true);
        assert_eq!(cache.exists("user:456:profile").await.unwrap(), true);

        // Create and use entity invalidator
        let invalidator = CacheInvalidation::new(InvalidationStrategy::EntityBased);
        invalidator
            .invalidate_entity(&cache, "user", "123")
            .await
            .unwrap();

        // Verify user 123 data is gone but 456 remains
        assert_eq!(cache.exists("user:123:profile").await.unwrap(), false);
        assert_eq!(cache.exists("user:123:settings").await.unwrap(), false);
        assert_eq!(cache.exists("user:456:profile").await.unwrap(), true);
    }

    #[tokio::test]
    async fn test_custom_invalidator() {
        // Create test invalidator
        let invalidator = TestInvalidator::new();

        // Invalidate some keys and patterns
        invalidator.invalidate_key("test:key1").await.unwrap();
        invalidator.invalidate_key("test:key2").await.unwrap();
        invalidator.invalidate_pattern("user:*").await.unwrap();

        // Verify invalidation was tracked
        assert_eq!(invalidator.invalidated_key_count().await, 2);
        assert_eq!(invalidator.invalidated_pattern_count().await, 1);
        assert!(invalidator.contains_key("test:key1").await);
        assert!(invalidator.contains_key("test:key2").await);
        assert!(invalidator.contains_pattern("user:*").await);
    }

    #[tokio::test]
    async fn test_ttl_invalidation() {
        // Set up cache with data
        let cache = create_test_cache();
        cache.clear().await.unwrap();

        // Add test data with different TTLs
        let short_ttl =
            navius_cache::operations::CacheOptions::new().ttl(Duration::from_millis(100));
        let long_ttl = navius_cache::operations::CacheOptions::new().ttl(Duration::from_secs(60));

        cache
            .set("short-lived", "value1", Some(short_ttl))
            .await
            .unwrap();
        cache
            .set("long-lived", "value2", Some(long_ttl))
            .await
            .unwrap();

        // Verify data exists
        assert_eq!(cache.exists("short-lived").await.unwrap(), true);
        assert_eq!(cache.exists("long-lived").await.unwrap(), true);

        // Wait for the short TTL to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Verify short-lived is gone but long-lived remains
        assert_eq!(cache.exists("short-lived").await.unwrap(), false);
        assert_eq!(cache.exists("long-lived").await.unwrap(), true);
    }
}
