#[cfg(feature = "redis")]
mod redis_tests {
    use navius_cache::{CacheConfig, CacheConnectionManager, CacheOptions, RedisCache};
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    use uuid::Uuid;

    // Test data structure
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestUser {
        id: String,
        name: String,
        email: String,
    }

    impl TestUser {
        fn new(name: &str, email: &str) -> Self {
            Self {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                email: email.to_string(),
            }
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

    // Helper function to create a test cache for each test
    async fn create_test_cache() -> CacheConnectionManager<RedisCache> {
        let config = create_test_config();

        // Skip the test if Redis is not available
        match CacheConnectionManager::new_redis(config).await {
            Ok(cache) => cache,
            Err(_) => {
                println!("Skipping test: Redis is not available");
                std::process::exit(0);
            }
        }
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        // Skip if Redis is not available
        let cache = match CacheConnectionManager::new_redis(create_test_config()).await {
            Ok(cache) => cache,
            Err(_) => return,
        };

        let key = "test-key";
        let value = "test-value";

        // Set a value
        cache.set(key, &value, None).await.unwrap();

        // Get the value
        let result: Option<String> = cache.get(key).await.unwrap();

        assert_eq!(result, Some(value.to_string()));
    }

    #[tokio::test]
    async fn test_cache_expiry() {
        // Skip if Redis is not available
        let cache = match CacheConnectionManager::new_redis(create_test_config()).await {
            Ok(cache) => cache,
            Err(_) => return,
        };

        let key = "expiry-key";
        let value = "expiry-value";

        // Set a value with a short TTL
        let options = CacheOptions::new().ttl(Duration::from_millis(100));
        cache.set(key, &value, Some(options)).await.unwrap();

        // Verify it's there
        let result: Option<String> = cache.get(key).await.unwrap();
        assert_eq!(result, Some(value.to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Verify it's gone
        let result: Option<String> = cache.get(key).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_delete() {
        // Skip if Redis is not available
        let cache = match CacheConnectionManager::new_redis(create_test_config()).await {
            Ok(cache) => cache,
            Err(_) => return,
        };

        let key = "delete-key";
        let value = "delete-value";

        // Set a value
        cache.set(key, &value, None).await.unwrap();

        // Verify it's there
        let result: Option<String> = cache.get(key).await.unwrap();
        assert_eq!(result, Some(value.to_string()));

        // Delete it
        let deleted = cache.delete(key).await.unwrap();
        assert!(deleted);

        // Verify it's gone
        let result: Option<String> = cache.get(key).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_complex_type() {
        // Skip if Redis is not available
        let cache = match CacheConnectionManager::new_redis(create_test_config()).await {
            Ok(cache) => cache,
            Err(_) => return,
        };

        let key = "user-key";
        let user = TestUser::new("John Doe", "john@example.com");

        // Set a complex value
        cache.set(key, &user, None).await.unwrap();

        // Get the value
        let result: Option<TestUser> = cache.get(key).await.unwrap();

        assert_eq!(result, Some(user));
    }

    #[tokio::test]
    async fn test_cache_get_many() {
        // Skip if Redis is not available
        let cache = match CacheConnectionManager::new_redis(create_test_config()).await {
            Ok(cache) => cache,
            Err(_) => return,
        };

        let keys = vec!["key1", "key2", "key3"];
        let values = vec!["value1", "value2", "value3"];

        // Set multiple values
        for (key, value) in keys.iter().zip(values.iter()) {
            cache.set(*key, value, None).await.unwrap();
        }

        // Get multiple values
        let results: Vec<Option<String>> = cache.get_many(keys.clone()).await.unwrap();

        // Verify results
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result, &Some(values[i].to_string()));
        }
    }

    #[tokio::test]
    async fn test_cache_increment() {
        // Skip if Redis is not available
        let cache = match CacheConnectionManager::new_redis(create_test_config()).await {
            Ok(cache) => cache,
            Err(_) => return,
        };

        let key = "counter-key";

        // Initialize counter
        let value = cache.increment(key, 1).await.unwrap();
        assert_eq!(value, 1);

        // Increment by 5
        let value = cache.increment(key, 5).await.unwrap();
        assert_eq!(value, 6);

        // Decrement by 2
        let value = cache.increment(key, -2).await.unwrap();
        assert_eq!(value, 4);
    }

    #[tokio::test]
    async fn test_cache_health_check() {
        // Skip if Redis is not available
        let cache = match CacheConnectionManager::new_redis(create_test_config()).await {
            Ok(cache) => cache,
            Err(_) => return,
        };

        // Health check should succeed
        let result = cache.health_check().await;
        assert!(result.is_ok());
    }
}
