use crate::core::api::ApiResource;
use crate::core::cache::{
    CacheStats, get_or_fetch, get_resource_cache, init_cache_registry, last_fetch_from_cache,
    register_resource_cache,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// Test resource type that implements ApiResource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestCacheResource {
    id: String,
    name: String,
    value: i32,
    updated_at: String,
}

impl ApiResource for TestCacheResource {
    type Id = String;

    fn resource_type() -> &'static str {
        "test_cache_resource"
    }

    fn api_name() -> &'static str {
        "TestCacheService"
    }
}

/// Integration tests for the cache system
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    // Helper function to create a test resource
    fn create_test_resource(id: &str, value: i32) -> TestCacheResource {
        TestCacheResource {
            id: id.to_string(),
            name: format!("Resource {}", id),
            value,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[test]
    async fn test_cache_basic_operations() -> Result<()> {
        // Initialize the cache registry with reasonable test values
        let registry = init_cache_registry(true, 100, 10); // 10 second TTL for quick testing

        // Register the test resource type
        register_resource_cache::<TestCacheResource>(&registry, "test_cache_resource")?;

        // Verify the cache was registered
        let cache = get_resource_cache::<TestCacheResource>(&registry, "test_cache_resource");
        assert!(cache.is_some(), "Cache should be registered");

        // Create a test resource
        let resource = create_test_resource("integration-1", 100);

        // Test get_or_fetch - First call should fetch
        let result = get_or_fetch(
            &registry,
            "test_cache_resource",
            "integration-1",
            || async { Ok(resource.clone()) },
        )
        .await?;

        assert_eq!(result, resource, "Fetched resource should match original");
        assert!(
            !last_fetch_from_cache(),
            "First fetch should not be from cache"
        );

        // Second call should hit cache
        let result2 = get_or_fetch(
            &registry,
            "test_cache_resource",
            "integration-1",
            || async {
                // This should not be called if cache hit
                Ok(create_test_resource("integration-1", 999))
            },
        )
        .await?;

        assert_eq!(result2, resource, "Cached resource should match original");
        assert!(last_fetch_from_cache(), "Second fetch should be from cache");

        // Test cache expiration
        sleep(Duration::from_secs(11)).await; // Wait longer than the TTL

        // After expiration, should fetch again
        let new_resource = create_test_resource("integration-1", 200);
        let result3 = get_or_fetch(
            &registry,
            "test_cache_resource",
            "integration-1",
            || async { Ok(new_resource.clone()) },
        )
        .await?;

        assert_eq!(
            result3, new_resource,
            "After expiration, should get new resource"
        );
        assert!(
            !last_fetch_from_cache(),
            "After expiration, should not be from cache"
        );

        Ok(())
    }

    #[test]
    async fn test_cache_stats() -> Result<()> {
        // Initialize the cache registry
        let registry = init_cache_registry(true, 100, 3600);

        // Register the test resource type
        register_resource_cache::<TestCacheResource>(&registry, "test_cache_resource")?;

        // Create and fetch multiple resources to generate stats
        for i in 1..=5 {
            let resource = create_test_resource(&format!("stats-{}", i), i * 10);

            // Initial fetch - miss
            let _ = get_or_fetch(
                &registry,
                "test_cache_resource",
                &format!("stats-{}", i),
                || async { Ok(resource.clone()) },
            )
            .await?;

            // Second fetch - hit
            let _ = get_or_fetch(
                &registry,
                "test_cache_resource",
                &format!("stats-{}", i),
                || async { Ok(resource.clone()) },
            )
            .await?;
        }

        // Get the cache to check stats
        let cache = get_resource_cache::<TestCacheResource>(&registry, "test_cache_resource")
            .expect("Cache should exist");

        // Get stats
        let stats = cache.get_stats();

        // Validate stats - we should have 5 entries, with hits and misses
        assert!(stats.size <= 5, "Cache size should be at most 5");
        assert!(stats.hits > 0, "Should have cache hits");
        assert!(stats.misses > 0, "Should have cache misses");
        assert!(stats.hit_ratio > 0.0, "Hit ratio should be greater than 0");

        Ok(())
    }

    // Note: An actual Redis integration test would require Redis to be running
    // This would typically be set up using testcontainers or similar
    // For now, this is a placeholder showing how it would work
    #[test]
    #[ignore = "Requires Redis to be running"]
    async fn test_redis_integration() -> Result<()> {
        // This test would:
        // 1. Start a Redis container using testcontainers
        // 2. Configure the cache to use that Redis instance
        // 3. Test cache operations that specifically use Redis features
        //    (like distributed caching)
        // 4. Verify that data persists in Redis after cache operations

        // For now, we'll just document the structure

        // TODO: When implementing Redis integration tests:
        // - Use testcontainers to start Redis
        // - Create Redis client to validate entries
        // - Test Redis-specific features like TTL, persistence

        Ok(())
    }
}
