// Redis cache integration tests

use navius_cache::{CacheOperations, CacheOptions};
use navius_cache_redis::{RedisCache, RedisCacheConfig};
use navius_test::error::{TestResult, assert_eq, assert_none, assert_some, assert_true};
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
fn create_test_config() -> RedisCacheConfig {
    RedisCacheConfig::new("redis://127.0.0.1:6379".to_string())
        .with_prefix(format!("navius-test-{}:", Uuid::new_v4()))
        .with_default_ttl(Duration::from_secs(60))
        .with_connect_timeout(Duration::from_secs(1))
        .with_max_connections(5)
        .with_trace(true)
}

// Helper function to create a test cache for each test
async fn create_test_cache() -> RedisCache {
    // Skip the test if Redis is not available
    match RedisCache::new(create_test_config()).await {
        Ok(cache) => cache,
        Err(_) => {
            println!("Skipping test: Redis is not available");
            std::process::exit(0);
        }
    }
}

#[tokio::test]
async fn test_cache_set_get() -> TestResult<()> {
    // Skip if Redis is not available
    let cache = match RedisCache::new(create_test_config()).await {
        Ok(cache) => cache,
        Err(_) => return Ok(()),
    };

    let key = "test-key";
    let value = "test-value";

    // Set a value
    cache.set(key, &value, None).await?;

    // Get the value
    let result: Option<String> = cache.get(key).await?;

    assert_some(
        result,
        value.to_string(),
        "Cache should return the value that was set",
    )?;

    Ok(())
}

#[tokio::test]
async fn test_cache_expiry() -> TestResult<()> {
    // Skip if Redis is not available
    let cache = match RedisCache::new(create_test_config()).await {
        Ok(cache) => cache,
        Err(_) => return Ok(()),
    };

    let key = "expiry-key";
    let value = "expiry-value";

    // Set a value with a short TTL
    let options = CacheOptions::new().ttl(Duration::from_millis(100));
    cache.set(key, &value, Some(options)).await?;

    // Verify it's there
    let result: Option<String> = cache.get(key).await?;
    assert_some(
        result,
        value.to_string(),
        "Cache should return the value before expiry",
    )?;

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Verify it's gone
    let result: Option<String> = cache.get(key).await?;
    assert_none(result, "Cache value should be gone after expiry")?;

    Ok(())
}

#[tokio::test]
async fn test_cache_delete() -> TestResult<()> {
    // Skip if Redis is not available
    let cache = match RedisCache::new(create_test_config()).await {
        Ok(cache) => cache,
        Err(_) => return Ok(()),
    };

    let key = "delete-key";
    let value = "delete-value";

    // Set a value
    cache.set(key, &value, None).await?;

    // Verify it's there
    let result: Option<String> = cache.get(key).await?;
    assert_some(
        result,
        value.to_string(),
        "Cache should return the value that was set",
    )?;

    // Delete it
    let deleted = cache.delete(key).await?;
    assert_true(
        deleted,
        "delete() should return true when deleting an existing key",
    )?;

    // Verify it's gone
    let result: Option<String> = cache.get(key).await?;
    assert_none(result, "Cache value should be gone after deletion")?;

    Ok(())
}

#[tokio::test]
async fn test_cache_complex_type() -> TestResult<()> {
    // Skip if Redis is not available
    let cache = match RedisCache::new(create_test_config()).await {
        Ok(cache) => cache,
        Err(_) => return Ok(()),
    };

    let key = "user-key";
    let user = TestUser::new("John Doe", "john@example.com");

    // Set a complex value
    cache.set(key, &user, None).await?;

    // Get the value
    let result: Option<TestUser> = cache.get(key).await?;

    assert_some(
        result,
        user.clone(),
        "Cache should return the complex value that was set",
    )?;

    Ok(())
}

#[tokio::test]
async fn test_cache_get_many() -> TestResult<()> {
    // Skip if Redis is not available
    let cache = match RedisCache::new(create_test_config()).await {
        Ok(cache) => cache,
        Err(_) => return Ok(()),
    };

    let keys = vec!["key1", "key2", "key3"];
    let values = vec!["value1", "value2", "value3"];

    // Set multiple values
    for (key, value) in keys.iter().zip(values.iter()) {
        cache.set(*key, value, None).await?;
    }

    // Get multiple values
    let results: Vec<Option<String>> = cache.get_many(keys.clone()).await?;

    // Verify results
    for (i, result) in results.iter().enumerate() {
        assert_some(
            result.clone(),
            values[i].to_string(),
            &format!("Cache should return the correct value for key {}", keys[i]),
        )?;
    }

    Ok(())
}

#[tokio::test]
async fn test_cache_increment() -> TestResult<()> {
    // Skip if Redis is not available
    let cache = match RedisCache::new(create_test_config()).await {
        Ok(cache) => cache,
        Err(_) => return Ok(()),
    };

    let key = "counter-key";

    // Initialize counter
    let value = cache.increment(key, 1).await?;
    assert_eq(value, 1, "Initial increment should set counter to 1")?;

    // Increment by 5
    let value = cache.increment(key, 5).await?;
    assert_eq(value, 6, "Incrementing by 5 should result in 6")?;

    // Decrement by 2
    let value = cache.increment(key, -2).await?;
    assert_eq(value, 4, "Decrementing by 2 should result in 4")?;

    Ok(())
}

#[tokio::test]
async fn test_cache_health_check() -> TestResult<()> {
    // Skip if Redis is not available
    let cache = match RedisCache::new(create_test_config()).await {
        Ok(cache) => cache,
        Err(_) => return Ok(()),
    };

    // Perform health check
    let result = cache.health_check().await;
    assert_true(
        result.is_ok(),
        "Health check should succeed for connected Redis instance",
    )?;

    Ok(())
}
