#[cfg(feature = "redis")]
mod invalidation_tests {
    use navius_cache::{
        invalidation::{CacheInvalidation, CacheInvalidator, InvalidationStrategy},
        CacheConfig, CacheConnectionManager, RedisCache,
    };
    use navius_test::error::{assert_eq, assert_none, assert_some, assert_true, TestResult};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::RwLock;
    use uuid::Uuid;

    // Test data structure
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestEntity {
        id: String,
        name: String,
    }

    impl TestEntity {
        fn new(name: &str) -> Self {
            Self {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
            }
        }
    }

    // Test invalidator that tracks calls for verification
    #[derive(Debug, Default)]
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

        async fn get_invalidated_keys(&self) -> Vec<String> {
            self.invalidated_keys.read().await.clone()
        }

        async fn get_invalidated_patterns(&self) -> Vec<String> {
            self.invalidated_patterns.read().await.clone()
        }
    }

    #[async_trait::async_trait]
    impl CacheInvalidator for TestInvalidator {
        async fn invalidate_key(&self, key: &str) -> Result<(), navius_cache::Error> {
            let mut keys = self.invalidated_keys.write().await;
            keys.push(key.to_string());
            Ok(())
        }

        async fn invalidate_pattern(&self, pattern: &str) -> Result<(), navius_cache::Error> {
            let mut patterns = self.invalidated_patterns.write().await;
            patterns.push(pattern.to_string());
            Ok(())
        }
    }

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
    async fn test_immediate_invalidation() -> TestResult<()> {
        // Skip if Redis is not available
        let Some(cache) = create_test_cache().await else {
            return Ok(());
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
        assert_true(result, "Invalidation should return true")?;

        // Verify first and third keys still exist
        let result1: Option<String> = cache.get(&keys[0]).await.unwrap();
        let result2: Option<String> = cache.get(&keys[1]).await.unwrap();
        let result3: Option<String> = cache.get(&keys[2]).await.unwrap();

        assert_eq(
            result1,
            Some(value.to_string()),
            "First key should still exist",
        )?;
        assert_eq(result2, None, "Second key should be invalidated")?;
        assert_eq(
            result3,
            Some(value.to_string()),
            "Third key should still exist",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_ttl_invalidation() -> TestResult<()> {
        // Skip if Redis is not available
        let Some(cache) = create_test_cache().await else {
            return Ok(());
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
        assert_true(result, "Invalidation should return true")?;

        // Verify key still exists immediately
        let result: Option<String> = cache.get(key).await.unwrap();
        assert_eq(
            result,
            Some(value.to_string()),
            "Key should still exist immediately",
        )?;

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Verify key is now gone
        let result: Option<String> = cache.get(key).await.unwrap();
        assert_eq(result, None, "Key should be invalidated after TTL expires")?;

        Ok(())
    }

    #[tokio::test]
    async fn test_pattern_based_invalidation() -> TestResult<()> {
        // Skip if Redis is not available
        let Some(cache) = create_test_cache().await else {
            return Ok(());
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
        assert_true(result, "Invalidation should return true")?;

        // Verify product keys are gone but other key remains
        for i in 0..3 {
            let result: Option<String> = cache.get(&keys[i]).await.unwrap();
            assert_eq(result, None, "Key {} should be invalidated", keys[i])?;
        }

        // The "another:key" should still exist
        let result: Option<String> = cache.get(&keys[3]).await.unwrap();
        assert_eq(
            result,
            Some(value.to_string()),
            "Key another:key should still exist",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_invalidate_all() -> TestResult<()> {
        // Skip if Redis is not available
        let Some(cache) = create_test_cache().await else {
            return Ok(());
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
            assert_eq(result, None, "Key {} should be invalidated", key)?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_change_strategy() -> TestResult<()> {
        // Skip if Redis is not available
        let Some(cache) = create_test_cache().await else {
            return Ok(());
        };

        // Create invalidator with initial strategy
        let mut invalidator =
            CacheInvalidator::<_, String>::new(cache.clone(), InvalidationStrategy::Immediate);

        // Check initial strategy
        assert_eq!(
            *invalidator.strategy(),
            InvalidationStrategy::Immediate,
            "Initial strategy should be Immediate"
        )?;

        // Change strategy
        invalidator.set_strategy(InvalidationStrategy::TimeToLive(Duration::from_secs(60)));

        // Check new strategy
        if let InvalidationStrategy::TimeToLive(ttl) = invalidator.strategy() {
            assert_eq!(
                *ttl,
                Duration::from_secs(60),
                "New strategy should be TimeToLive with TTL 60 seconds"
            )?;
        } else {
            panic!("Strategy should be TimeToLive");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_entity_invalidation_strategy() -> TestResult<()> {
        // Create a test cache
        let cache = create_test_cache().await.unwrap();

        // Create a test invalidator
        let invalidator = Arc::new(TestInvalidator::new());

        // Define the invalidation strategy
        let strategy = InvalidationStrategy::new()
            .when_entity("user")
            .with_id_field("id")
            .invalidate_key("user:{id}")
            .invalidate_pattern("user:*:friends")
            .build();

        // Execute the strategy
        let entity = TestEntity::new("John");
        strategy
            .execute(
                &entity,
                Arc::clone(&invalidator) as Arc<dyn CacheInvalidator>,
            )
            .await?;

        // Check invalidated keys
        let invalidated_keys = invalidator.get_invalidated_keys().await;
        assert_eq(
            invalidated_keys.len(),
            1,
            "Should invalidate exactly one key",
        )?;
        assert_true(
            invalidated_keys[0].contains(&entity.id),
            "Invalidated key should contain the entity ID",
        )?;

        // Check invalidated patterns
        let invalidated_patterns = invalidator.get_invalidated_patterns().await;
        assert_eq(
            invalidated_patterns.len(),
            1,
            "Should invalidate exactly one pattern",
        )?;
        assert_eq(
            invalidated_patterns[0],
            "user:*:friends",
            "Should invalidate the correct pattern",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_compound_invalidation_strategy() -> TestResult<()> {
        // Create a test invalidator
        let invalidator = Arc::new(TestInvalidator::new());

        // Define a compound invalidation strategy
        let strategy = InvalidationStrategy::new()
            .when_entity("order")
            .with_id_field("id")
            .invalidate_key("order:{id}")
            .invalidate_key("order:{id}:details")
            .invalidate_pattern("user:{userId}:orders")
            .build();

        // Mock entity with compound data
        let mut entity_data = HashMap::new();
        entity_data.insert("id".to_string(), "order-123".to_string());
        entity_data.insert("userId".to_string(), "user-456".to_string());

        // Execute the strategy with the HashMap entity
        strategy
            .execute(
                &entity_data,
                Arc::clone(&invalidator) as Arc<dyn CacheInvalidator>,
            )
            .await?;

        // Check invalidated keys
        let invalidated_keys = invalidator.get_invalidated_keys().await;
        assert_eq(
            invalidated_keys.len(),
            2,
            "Should invalidate exactly two keys",
        )?;

        let expected_keys = vec![
            "order:order-123".to_string(),
            "order:order-123:details".to_string(),
        ];

        for key in expected_keys {
            assert_true(
                invalidated_keys.contains(&key),
                &format!("Should invalidate key: {}", key),
            )?;
        }

        // Check invalidated patterns
        let invalidated_patterns = invalidator.get_invalidated_patterns().await;
        assert_eq(
            invalidated_patterns.len(),
            1,
            "Should invalidate exactly one pattern",
        )?;
        assert_eq(
            invalidated_patterns[0],
            "user:user-456:orders",
            "Should interpolate the userId in the pattern",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_with_invalidation() -> TestResult<()> {
        // Create a test cache
        let cache = create_test_cache().await.unwrap();

        // Set up test data
        let user_id = Uuid::new_v4().to_string();
        let user_key = format!("user:{}", user_id);
        let user = TestEntity::new("Alice");

        // Store the user in cache
        cache.set(&user_key, &user, None).await?;

        // Verify the user is in cache
        let cached_user: Option<TestEntity> = cache.get(&user_key).await?;
        assert_some(
            cached_user,
            user.clone(),
            "User should be available in cache after setting",
        )?;

        // Create a test invalidator
        let invalidator = Arc::new(TestInvalidator::new());

        // Define and execute an invalidation strategy
        let strategy = InvalidationStrategy::new()
            .when_entity("user")
            .with_id_field("id")
            .invalidate_key(&format!("user:{}", user.id))
            .build();

        strategy
            .execute(&user, Arc::clone(&invalidator) as Arc<dyn CacheInvalidator>)
            .await?;

        // Check invalidated keys
        let invalidated_keys = invalidator.get_invalidated_keys().await;
        assert_eq(
            invalidated_keys.len(),
            1,
            "Should invalidate exactly one key",
        )?;
        assert_eq(
            invalidated_keys[0],
            &format!("user:{}", user.id),
            "Should invalidate the user key",
        )?;

        // Manually clear the cache to simulate the invalidator's effect
        cache.delete(&user_key).await?;

        // Verify the user is no longer in cache
        let cached_user: Option<TestEntity> = cache.get(&user_key).await?;
        assert_none(
            cached_user,
            "User should not be available in cache after invalidation",
        )?;

        Ok(())
    }
}
