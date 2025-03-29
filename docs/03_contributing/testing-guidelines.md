---
title: ""
description: "Reference documentation for Navius "
category: "Reference"
tags: ["documentation", "reference"]
last_updated: "April 3, 2025"
version: "1.0"
---


# Testing Guidelines

This document outlines testing guidelines for Navius components, with special focus on testing complex features like the Two-Tier Cache implementation.

## Table of Contents

- [General Testing Principles](#general-testing-principles)
- [Test Types](#test-types)
- [Test Structure](#test-structure)
- [Testing Complex Components](#testing-complex-components)
  - [Testing Cache Implementations](#testing-cache-implementations)
  - [Testing Server Customization](#testing-server-customization)
- [Test Coverage Requirements](#test-coverage-requirements)
- [Mocking and Test Doubles](#mocking-and-test-doubles)
- [CI/CD Integration](#cicd-integration)

## General Testing Principles

1. **Test Isolation**: Each test should be isolated from others and not depend on external state
2. **Coverage**: Aim for high test coverage, but focus on critical paths and edge cases
3. **Test Behavior**: Test the behavior of components, not their implementation details
4. **Reliability**: Tests should be reliable and not produce flaky results
5. **Performance**: Tests should execute quickly to support rapid development
6. **Readability**: Tests should be easy to understand and maintain

## Test Types

### Unit Tests

- Test individual functions and methods in isolation
- Mock external dependencies
- Focus on specific behavior

```rust
#[test]
fn test_cache_key_formatting() {
    let key = format_cache_key("user", "123");
    assert_eq!(key, "user:123");
}
```

### Integration Tests

- Test interactions between components
- Use real implementations or realistic mocks
- Focus on component boundaries

```rust
#[tokio::test]
async fn test_cache_with_redis() {
    let redis = MockRedisClient::new();
    let cache = RedisCache::new(redis);
    
    cache.set("test", b"value", None).await.unwrap();
    let result = cache.get("test").await.unwrap();
    
    assert_eq!(result, b"value");
}
```

### End-to-End Tests

- Test the entire system as a whole
- Use real external dependencies where possible
- Focus on user scenarios

```rust
#[tokio::test]
async fn test_user_service_with_cache() {
    let app = test_app().await;
    
    // Create a user
    let user_id = app.create_user("test@example.com").await.unwrap();
    
    // First request should hit the database
    let start = Instant::now();
    let user1 = app.get_user(user_id).await.unwrap();
    let first_request_time = start.elapsed();
    
    // Second request should hit the cache
    let start = Instant::now();
    let user2 = app.get_user(user_id).await.unwrap();
    let second_request_time = start.elapsed();
    
    // Verify cache is faster
    assert!(second_request_time < first_request_time);
    
    // Verify data is the same
    assert_eq!(user1, user2);
}
```

## Test Structure

Follow the AAA pattern for test structure:

1. **Arrange**: Set up the test conditions
2. **Act**: Execute the code under test
3. **Assert**: Verify the expected outcome

```rust
#[test]
fn test_cache_ttl() {
    // Arrange
    let mock_clock = MockClock::new();
    let cache = InMemoryCache::new_with_clock(100, mock_clock.clone());
    
    // Act - Set a value with TTL
    cache.set("key", b"value", Some(Duration::from_secs(5))).unwrap();
    
    // Assert - Value exists before expiration
    assert_eq!(cache.get("key").unwrap(), b"value");
    
    // Act - Advance time past TTL
    mock_clock.advance(Duration::from_secs(6));
    
    // Assert - Value is gone after expiration
    assert!(cache.get("key").is_err());
}
```

## Testing Complex Components

### Testing Cache Implementations

Testing caching components requires special attention to:

1. **Cache Hit/Miss Scenarios**
   ```rust
   #[tokio::test]
   async fn test_cache_hit_miss() {
       let cache = create_test_cache().await;
       
       // Test cache miss
       let result = cache.get("missing-key").await;
       assert!(result.is_err());
       assert!(matches!(result.unwrap_err(), AppError::NotFound { .. }));
       
       // Set value and test cache hit
       cache.set("test-key", b"value", None).await.unwrap();
       let result = cache.get("test-key").await.unwrap();
       assert_eq!(result, b"value");
   }
   ```

2. **TTL Behavior**
   ```rust
   #[tokio::test]
   async fn test_cache_ttl() {
       let cache = create_test_cache().await;
       
       // Set with short TTL
       cache.set("expires", b"value", Some(Duration::from_millis(100))).await.unwrap();
       
       // Verify exists
       let result = cache.get("expires").await.unwrap();
       assert_eq!(result, b"value");
       
       // Wait for expiration
       tokio::time::sleep(Duration::from_millis(150)).await;
       
       // Verify expired
       let result = cache.get("expires").await;
       assert!(result.is_err());
   }
   ```

3. **Two-Tier Cache Promotion**
   ```rust
   #[tokio::test]
   async fn test_two_tier_promotion() {
       let fast_cache = MockCache::new("fast");
       let slow_cache = MockCache::new("slow");
       
       // Configure mocks
       fast_cache.expect_get().with(eq("key")).return_error(AppError::not_found("key"));
       slow_cache.expect_get().with(eq("key")).return_once(|_| Ok(b"value".to_vec()));
       fast_cache.expect_set().with(eq("key"), eq(b"value".to_vec()), any()).return_once(|_, _, _| Ok(()));
       
       let two_tier = TwoTierCache::new(
           Box::new(fast_cache),
           Box::new(slow_cache),
           true, // promote_on_get
           None,
           None,
       );
       
       // Item should be fetched from slow cache and promoted to fast cache
       let result = two_tier.get("key").await.unwrap();
       assert_eq!(result, b"value");
   }
   ```

4. **Redis Unavailability**
   ```rust
   #[tokio::test]
   async fn test_redis_unavailable() {
       let config = CacheConfig {
           redis_url: "redis://nonexistent:6379",
           // other config...
       };
       
       // Create cache with invalid Redis URL
       let cache = create_memory_only_two_tier_cache(&config, None).await;
       
       // Should still work using just the memory cache
       cache.set("test", b"value", None).await.unwrap();
       let result = cache.get("test").await.unwrap();
       assert_eq!(result, b"value");
   }
   ```

5. **Concurrent Operations**
   ```rust
   #[tokio::test]
   async fn test_concurrent_operations() {
       let cache = create_test_cache().await;
       
       // Spawn multiple tasks writing to the same key
       let mut handles = vec![];
       for i in 0..10 {
           let cache_clone = cache.clone();
           let handle = tokio::spawn(async move {
               let value = format!("value-{}", i).into_bytes();
               cache_clone.set("concurrent-key", value, None).await.unwrap();
           });
           handles.push(handle);
       }
       
       // Wait for all operations to complete
       for handle in handles {
           handle.await.unwrap();
       }
       
       // Verify key exists
       let result = cache.get("concurrent-key").await;
       assert!(result.is_ok());
   }
   ```

### Testing Server Customization

For server customization components, focus on:

1. **Feature Flag Combinations**
2. **Feature Dependency Resolution**
3. **Configuration Validation**
4. **Build System Integration**

## Test Coverage Requirements

Aim for the following coverage levels:

| Component Type | Minimum Coverage |
|----------------|------------------|
| Core Services  | 90%              |
| Cache Implementations | 95%       |
| Utilities      | 80%              |
| API Handlers   | 85%              |
| Configuration  | 90%              |

## Mocking and Test Doubles

1. **Use Mock Implementations for External Dependencies**
   ```rust
   #[derive(Clone)]
   struct MockRedisClient {
       data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
   }
   
   #[async_trait]
   impl RedisClient for MockRedisClient {
       async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, RedisError> {
           let data = self.data.read().await;
           Ok(data.get(key).cloned())
       }
       
       async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<(), RedisError> {
           let mut data = self.data.write().await;
           data.insert(key.to_string(), value);
           Ok(())
       }
       
       // Other methods...
   }
   ```

2. **Inject Test Doubles**
   ```rust
   #[tokio::test]
   async fn test_cache_with_mock_redis() {
       let redis = MockRedisClient::new();
       let cache = RedisCache::new(Arc::new(redis));
       
       // Test cache operations...
   }
   ```

3. **Use Test Fixtures for Common Setup**
   ```rust
   async fn create_test_cache() -> Arc<Box<dyn DynCacheOperations>> {
       let config = CacheConfig {
           redis_url: "redis://localhost:6379".to_string(),
           // other test config...
       };
       
       // Use in-memory implementation for tests
       create_memory_only_two_tier_cache(&config, None).await
   }
   ```

## CI/CD Integration

1. **Run Tests on Every PR**
   ```yaml
   # In .gitlab-ci.yml
   test:
     stage: test
     script:
       - cargo test
   ```

2. **Track Code Coverage**
   ```yaml
   coverage:
     stage: test
     script:
       - cargo install cargo-tarpaulin
       - cargo tarpaulin --out Xml
       - upload-coverage coverage.xml
   ```

3. **Enforce Coverage Thresholds**
   ```yaml
   coverage:
     stage: test
     script:
       - cargo tarpaulin --out Xml --fail-under 85
   ```

## Frequently Asked Questions

### How to test async code?

Use the `tokio::test` attribute for async tests:

```rust
#[tokio::test]
async fn test_async_cache_operations() {
    let cache = create_test_cache().await;
    // Test async operations...
}
```

### How to test error handling?

Test both success and error cases:

```rust
#[tokio::test]
async fn test_cache_error_handling() {
    let cache = create_test_cache().await;
    
    // Test missing key
    let result = cache.get("nonexistent").await;
    assert!(result.is_err());
    
    // Test invalid serialization
    let typed_cache = cache.get_typed_cache::<User>();
    let result = typed_cache.get("invalid-json").await;
    assert!(result.is_err());
}
```

### How to test with real Redis?

For integration tests, use a real Redis instance:

```rust
#[tokio::test]
async fn test_with_real_redis() {
    // Skip if Redis is not available
    if !is_redis_available("redis://localhost:6379").await {
        println!("Skipping test: Redis not available");
        return;
    }
    
    let config = CacheConfig {
        redis_url: "redis://localhost:6379".to_string(),
        // other config...
    };
    
    let cache = create_two_tier_cache(&config, None).await.unwrap();
    
    // Test with real Redis...
}
```
