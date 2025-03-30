#[cfg(feature = "redis")]
mod invalidation_tests {
    use navius_cache::{
        CacheConfig, CacheConnectionManager, RedisCache,
        invalidation::{CacheInvalidation, CacheInvalidator, InvalidationStrategy},
    };
    use std::time::Duration;
    use uuid::Uuid;

    // Helper function to create a unique test configuration
    fn create_test_config() -> CacheConfig {
        CacheConfig::new(
            "redis://127.0.0.1:6379".to_string(),
            format!("navius-test-{}:", Uuid::new_v4()),
            Duration::from_secs(60),
        )
        .with_connect_timeout(Duration::from_secs(1))
        .with_max_connections(5)
        .with_trace(true)
    }

    // Helper function to create a test cache manager
    async fn create_test_cache() -> Option<CacheConnectionManager<RedisCache>> {
        match CacheConnectionManager::new_redis(create_test_config()).await {
            Ok(cache) => Some(cache),
            Err(_) => None,
        }
    }

    #[tokio::test]
    async fn test_immediate_invalidation() {
        // Skip if Redis is not available
        let Some(cache) = create_test_cache().await else {
            return;
        };

        // Set up some test data
        let prefix = "user:";
        let keys = vec![
            format!("{}123", prefix),
            format!("{}456", prefix),
            format!("{}789", prefix),
        ];
        let value = "test-value";

        // Set values
        for key in &keys {
            cache.set(key, &value, None).await.unwrap();
        }

        // Create invalidator with immediate strategy
        let invalidator =
            CacheInvalidator::<_, String>::new(cache.clone(), InvalidationStrategy::Immediate);

        // Invalidate one key
        let result = invalidator.invalidate(&keys[1]).await.unwrap();
        assert!(result);

        // Verify first and third keys still exist
        let result1: Option<String> = cache.get(&keys[0]).await.unwrap();
        let result2: Option<String> = cache.get(&keys[1]).await.unwrap();
        let result3: Option<String> = cache.get(&keys[2]).await.unwrap();

        assert_eq!(result1, Some(value.to_string()));
        assert_eq!(result2, None); // This one should be gone
        assert_eq!(result3, Some(value.to_string()));
    }

    #[tokio::test]
    async fn test_ttl_invalidation() {
        // Skip if Redis is not available
        let Some(cache) = create_test_cache().await else {
            return;
        };

        // Set up test data
        let key = "ttl-test";
        let value = "test-value";

        // Set value
        cache.set(key, &value, None).await.unwrap();

        // Create invalidator with TTL strategy (100ms)
        let invalidator = CacheInvalidator::<_, String>::new(
            cache.clone(),
            InvalidationStrategy::TimeToLive(Duration::from_millis(100)),
        );

        // Apply TTL invalidation
        let result = invalidator.invalidate(key).await.unwrap();
        assert!(result);

        // Verify key still exists immediately
        let result: Option<String> = cache.get(key).await.unwrap();
        assert_eq!(result, Some(value.to_string()));

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Verify key is now gone
        let result: Option<String> = cache.get(key).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_pattern_based_invalidation() {
        // Skip if Redis is not available
        let Some(cache) = create_test_cache().await else {
            return;
        };

        // Set up test data
        let prefix = "product:";
        let keys = vec![
            format!("{}123", prefix),
            format!("{}456", prefix),
            format!("{}789", prefix),
            "another:key".to_string(),
        ];
        let value = "test-value";

        // Set values
        for key in &keys {
            cache.set(key, &value, None).await.unwrap();
        }

        // Create invalidator with pattern-based strategy
        let pattern = "{}*".to_string(); // Will be replaced with the key + *
        let invalidator = CacheInvalidator::<_, String>::new(
            cache.clone(),
            InvalidationStrategy::PatternBased(pattern),
        );

        // Invalidate by pattern (product:*)
        let result = invalidator.invalidate(prefix).await.unwrap();
        assert!(result);

        // Verify product keys are gone but other key remains
        for i in 0..3 {
            let result: Option<String> = cache.get(&keys[i]).await.unwrap();
            assert_eq!(result, None, "Key {} should be invalidated", keys[i]);
        }

        // The "another:key" should still exist
        let result: Option<String> = cache.get(&keys[3]).await.unwrap();
        assert_eq!(result, Some(value.to_string()));
    }

    #[tokio::test]
    async fn test_invalidate_all() {
        // Skip if Redis is not available
        let Some(cache) = create_test_cache().await else {
            return;
        };

        // Set up test data
        let keys = vec!["key1", "key2", "key3"];
        let value = "test-value";

        // Set values
        for key in &keys {
            cache.set(*key, &value, None).await.unwrap();
        }

        // Create invalidator
        let invalidator =
            CacheInvalidator::<_, String>::new(cache.clone(), InvalidationStrategy::Immediate);

        // Invalidate all
        invalidator.invalidate_all().await.unwrap();

        // Verify all keys are gone
        for key in &keys {
            let result: Option<String> = cache.get(*key).await.unwrap();
            assert_eq!(result, None, "Key {} should be invalidated", key);
        }
    }

    #[tokio::test]
    async fn test_change_strategy() {
        // Skip if Redis is not available
        let Some(cache) = create_test_cache().await else {
            return;
        };

        // Create invalidator with initial strategy
        let mut invalidator =
            CacheInvalidator::<_, String>::new(cache.clone(), InvalidationStrategy::Immediate);

        // Check initial strategy
        assert_eq!(*invalidator.strategy(), InvalidationStrategy::Immediate);

        // Change strategy
        invalidator.set_strategy(InvalidationStrategy::TimeToLive(Duration::from_secs(60)));

        // Check new strategy
        if let InvalidationStrategy::TimeToLive(ttl) = invalidator.strategy() {
            assert_eq!(*ttl, Duration::from_secs(60));
        } else {
            panic!("Strategy should be TimeToLive");
        }
    }
}
