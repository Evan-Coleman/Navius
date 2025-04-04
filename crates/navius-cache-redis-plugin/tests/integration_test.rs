/// Redis Cache Plugin Integration Tests
///
/// These tests demonstrate how to use the testing utilities to test Redis functionality.
///
/// Note: Some tests require a running Redis server. They will be skipped
/// if Redis is not available.
// Import test utilities
mod utils;

use navius_cache::{Cache, CacheKey, CacheOperations, CacheOptions};
use navius_cache_redis_plugin::{RedisCache, RedisCacheConfig};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use utils::{MockRedis, RedisTestContext, TestRedisServer};

/// Run integration tests (requires running Redis server)
#[tokio::test]
async fn test_redis_cache_integration() {
    // Skip test if Redis is not available
    let redis_available = TestRedisServer::is_available().await;
    if !redis_available {
        println!("Redis server not available, skipping integration test");
        return;
    }

    // Create a test context that will clean up after itself
    let context = RedisTestContext::new()
        .await
        .expect("Failed to create Redis test context");
    let cache = context.cache();

    // Test basic operations
    test_basic_operations(cache).await;

    // Test complex types
    test_complex_types(cache).await;

    // Test list operations
    test_list_operations(cache).await;

    // Test batch operations
    test_batch_operations(cache).await;

    // The context will automatically clean up Redis keys when dropped
    println!("Integration tests completed successfully");
}

/// Test with mock Redis for unit testing without Redis server
#[tokio::test]
async fn test_mock_redis() {
    // Create a mock Redis instance (in-memory, no actual Redis server needed)
    let mock = MockRedis::new("test:".to_string(), Duration::from_secs(60));

    // Run the same tests with the mock
    test_basic_operations(&mock).await;
    test_complex_types(&mock).await;
    test_list_operations(&mock).await;
    test_batch_operations(&mock).await;

    println!("Mock Redis tests completed successfully");
}

/// Helper test function for basic cache operations
async fn test_basic_operations<T>(cache: &T)
where
    T: Cache + CacheOperations,
{
    // Set and get a simple value
    cache
        .set("test_key", &"test_value", None)
        .await
        .expect("Failed to set value");

    let result: Option<String> = cache.get("test_key").await.expect("Failed to get value");
    assert_eq!(result, Some("test_value".to_string()));

    // Check if key exists
    let exists = cache
        .exists("test_key")
        .await
        .expect("Failed to check existence");
    assert!(exists);

    // Set TTL
    cache
        .expire("test_key", Duration::from_secs(30))
        .await
        .expect("Failed to set TTL");

    // Get TTL
    let ttl = cache.ttl("test_key").await.expect("Failed to get TTL");
    assert!(ttl.is_some());
    let ttl_value = ttl.unwrap();
    assert!(ttl_value > 0 && ttl_value <= 30);

    // Delete a key
    let deleted = cache
        .delete("test_key")
        .await
        .expect("Failed to delete key");
    assert!(deleted);

    // Key should no longer exist
    let exists = cache
        .exists("test_key")
        .await
        .expect("Failed to check existence");
    assert!(!exists);

    // Increment a counter
    let count1 = cache
        .increment("counter", 1)
        .await
        .expect("Failed to increment");
    assert_eq!(count1, 1);

    let count2 = cache
        .increment("counter", 5)
        .await
        .expect("Failed to increment");
    assert_eq!(count2, 6);
}

/// Test with complex data types
async fn test_complex_types<T>(cache: &T)
where
    T: Cache + CacheOperations,
{
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        id: u32,
        name: String,
        email: String,
        active: bool,
    }

    let user = User {
        id: 123,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        active: true,
    };

    // Set a complex object
    cache
        .set("user:123", &user, None)
        .await
        .expect("Failed to set user");

    // Get the complex object back
    let retrieved: Option<User> = cache.get("user:123").await.expect("Failed to get user");
    assert_eq!(retrieved, Some(user));
}

/// Test list operations
async fn test_list_operations<T>(cache: &T)
where
    T: Cache + CacheOperations,
{
    // Push items to a list
    cache
        .list_push_right("test_list", &"item1")
        .await
        .expect("Failed to push item");
    cache
        .list_push_right("test_list", &"item2")
        .await
        .expect("Failed to push item");
    cache
        .list_push_right("test_list", &"item3")
        .await
        .expect("Failed to push item");

    // Get list length
    let len = cache
        .list_len("test_list")
        .await
        .expect("Failed to get list length");
    assert_eq!(len, 3);

    // Get range
    let items: Vec<String> = cache
        .list_range("test_list", 0, -1)
        .await
        .expect("Failed to get list range");
    assert_eq!(
        items,
        vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string()
        ]
    );

    // Get partial range
    let partial: Vec<String> = cache
        .list_range("test_list", 1, 2)
        .await
        .expect("Failed to get partial range");
    assert_eq!(partial, vec!["item2".to_string()]);

    // Pop an item
    let popped: Option<String> = cache
        .list_pop_left("test_list")
        .await
        .expect("Failed to pop item");
    assert_eq!(popped, Some("item1".to_string()));

    // Check updated list
    let updated: Vec<String> = cache
        .list_range("test_list", 0, -1)
        .await
        .expect("Failed to get updated list");
    assert_eq!(updated, vec!["item2".to_string(), "item3".to_string()]);
}

/// Test batch operations
async fn test_batch_operations<T>(cache: &T)
where
    T: Cache + CacheOperations,
{
    // Test set_many
    let entries = vec![
        ("batch:1", "value1"),
        ("batch:2", "value2"),
        ("batch:3", "value3"),
    ];

    cache
        .set_many(entries, None)
        .await
        .expect("Failed to set many");

    // Test get_many
    let keys = vec!["batch:1", "batch:2", "batch:3", "batch:nonexistent"];
    let results: Vec<Option<String>> = cache.get_many(keys).await.expect("Failed to get many");

    assert_eq!(results.len(), 4);
    assert_eq!(results[0], Some("value1".to_string()));
    assert_eq!(results[1], Some("value2".to_string()));
    assert_eq!(results[2], Some("value3".to_string()));
    assert_eq!(results[3], None);

    // Test delete_many
    let delete_keys = vec!["batch:1", "batch:2", "batch:nonexistent"];
    let deleted = cache
        .delete_many(delete_keys)
        .await
        .expect("Failed to delete many");
    assert_eq!(deleted, 2); // Only 2 keys existed

    // Check the remaining key
    let exists = cache
        .exists("batch:3")
        .await
        .expect("Failed to check existence");
    assert!(exists);

    let does_not_exist = cache
        .exists("batch:1")
        .await
        .expect("Failed to check existence");
    assert!(!does_not_exist);
}
