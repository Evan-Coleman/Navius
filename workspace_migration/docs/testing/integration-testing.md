# Integration Testing with Cross-Crate Testing Infrastructure

**Date:** March 29, 2025  
**Status:** Complete  
**Related Component:** Cross-Crate Testing Infrastructure

## Overview

This guide provides instructions and best practices for implementing integration tests using the Navius Cross-Crate Testing Infrastructure. Integration tests verify that components from different crates work correctly together, ensuring that interface boundaries are properly maintained and that cross-crate workflows function as expected.

## Key Components for Integration Testing

The Cross-Crate Testing Infrastructure provides several key components for integration testing:

1. **IntegrationContext**: Provides the environment for tests, including configuration, mocks, and resources
2. **IntegrationRunner**: Manages the test lifecycle including setup, execution, and teardown
3. **TestFixture**: Configures components and their dependencies for testing
4. **MockRegistry**: Manages mock implementations across crate boundaries
5. **TestResult**: Standardized error handling and reporting

## Setting Up Integration Tests

### Basic Integration Test Structure

```rust
use navius_test::{
    error::TestResult,
    integration::{IntegrationRunner, IntegrationTestConfig},
};

#[test]
async fn test_cross_crate_interaction() -> TestResult<()> {
    // Configure the test
    let config = IntegrationTestConfig {
        name: "cross-crate-test".to_string(),
        verify_mocks: true,
        cleanup_resources: true,
        timeout: Some(Duration::from_secs(10)),
        ..Default::default()
    };

    // Create a test runner
    let runner = IntegrationRunner::new(config)?;

    // Execute the test with the context
    runner.run(|context| async move {
        // Test implementation
        
        Ok(())
    }).await
}
```

### Testing Components from Multiple Crates

To test interactions between components from different crates:

```rust
#[test]
async fn test_database_and_cache_integration() -> TestResult<()> {
    // Create a test fixture with components from multiple crates
    let fixture = TestFixture::new()
        .with_database()
        .with_cache()
        .build()?;
    
    // Get the database client
    let db = fixture.get::<dyn DatabaseClient>()?;
    
    // Get the cache client
    let cache = fixture.get::<dyn CacheClient>()?;
    
    // Set up test data
    db.execute("INSERT INTO users (id, name) VALUES (1, 'Test User')")?;
    
    // Test the interaction
    let user_service = UserService::new(db, cache);
    let user = user_service.get_user(1).await?;
    
    // Verify results
    assert_eq(user.name, "Test User", "Should retrieve user from database")?;
    
    // Verify it was cached
    assert_true(cache.has("user:1").await?, "User should be cached")?;
    
    // Verify all mock expectations
    fixture.verify()?;
    
    Ok(())
}
```

## Using Test Fixtures for Integration

Test Fixtures provide a powerful way to set up complex test scenarios involving multiple components:

```rust
#[test]
async fn test_complete_workflow() -> TestResult<()> {
    // Create a comprehensive test environment
    let fixture = TestFixture::new()
        .with_config(|config| {
            config.set("app.environment", "test");
            config.set("db.connection", "mock://test");
            config.set("cache.ttl", "60");
        })
        .with_database(|db| {
            db.expect_query("SELECT * FROM users WHERE id = ?", |args| {
                assert_eq!(args[0], 1);
                Ok(MockQueryResult::new().add_row({
                    let mut row = HashMap::new();
                    row.insert("id".to_string(), MockValue::Integer(1));
                    row.insert("name".to_string(), MockValue::String("Test User".to_string()));
                    row
                }))
            });
        })
        .with_cache(|cache| {
            cache.expect_get("user:1", None);
            cache.expect_set("user:1", json!({"id": 1, "name": "Test User"}), Some(60));
        })
        .build()?;
    
    // Create the service under test with components from the fixture
    let user_service = UserService::new(
        fixture.get::<dyn DatabaseClient>()?,
        fixture.get::<dyn CacheClient>()?,
        fixture.get::<dyn ConfigProvider>()?,
    );
    
    // Execute the workflow
    let result = user_service.process_user(1).await?;
    
    // Verify results
    assert_eq(result.status, "success", "Workflow should succeed")?;
    
    // Verify all mock expectations were met
    fixture.verify()?;
    
    Ok(())
}
```

## Cross-Crate Event Testing

Testing events that cross crate boundaries:

```rust
#[test]
async fn test_event_propagation_across_crates() -> TestResult<()> {
    // Create a test fixture with event system
    let fixture = TestFixture::new()
        .with_event_system()
        .with_user_service()
        .with_notification_service()
        .build()?;
    
    // Get services
    let user_service = fixture.get::<dyn UserService>()?;
    let notification_service = fixture.get::<dyn NotificationService>()?;
    
    // Configure event expectations
    let event_bus = fixture.get::<dyn EventBus>()?;
    event_bus.expect_event("user.created", |event| {
        assert_eq(event.payload.get("user_id"), Some(1));
        true
    });
    
    // Execute the action that should trigger events
    user_service.create_user("test_user", "test@example.com").await?;
    
    // Verify that the notification service received and processed the event
    let notifications = notification_service.get_pending_notifications().await?;
    assert_true(
        notifications.iter().any(|n| n.user_id == 1 && n.type_name == "welcome"),
        "Welcome notification should be created"
    )?;
    
    // Verify all mock expectations
    fixture.verify()?;
    
    Ok(())
}
```

## Error Propagation Testing

Testing how errors propagate across crate boundaries:

```rust
#[test]
async fn test_error_propagation() -> TestResult<()> {
    // Create a test fixture with error injection
    let fixture = TestFixture::new()
        .with_database(|db| {
            // Configure database to return an error
            db.expect_query("SELECT * FROM users WHERE id = ?", |args| {
                assert_eq!(args[0], 1);
                Err(MockDatabaseError::new("Connection refused"))
            });
        })
        .with_cache()
        .build()?;
    
    // Create service with components that will fail
    let user_service = UserService::new(
        fixture.get::<dyn DatabaseClient>()?,
        fixture.get::<dyn CacheClient>()?,
    );
    
    // Execute operation that should result in error
    let result = user_service.get_user(1).await;
    
    // Verify error behavior
    assert_err(result, "Should propagate database error")?;
    
    // Check error details
    let error = result.unwrap_err();
    assert_contains(error.to_string(), "Connection refused", "Error should contain original message")?;
    assert_contains(error.to_string(), "user_service", "Error should mention user service")?;
    
    // Verify all mock expectations
    fixture.verify()?;
    
    Ok(())
}
```

## Test Cleanup and Resource Management

Managing resources during tests:

```rust
#[test]
async fn test_with_resource_cleanup() -> TestResult<()> {
    // Create temporary test resources
    let temp_dir = tempfile::tempdir()?;
    let temp_file = temp_dir.path().join("test.json");
    
    // Create test configuration
    let config = IntegrationTestConfig {
        name: "resource-test".to_string(),
        test_dir: temp_dir.path().to_path_buf(),
        cleanup_resources: true,
        ..Default::default()
    };
    
    // Create runner with resource management
    let runner = IntegrationRunner::new(config)?;
    
    // Run test
    runner.run(|context| async move {
        // Create a file in the test directory
        let fs = context.filesystem()?;
        fs.write_file(&temp_file, r#"{"test": true}"#).await?;
        
        // Use the file in tests
        assert_true(fs.exists(&temp_file).await?, "File should exist")?;
        
        Ok(())
    }).await?;
    
    // Verify cleanup (if configured with cleanup_resources: false)
    // assert!(!temp_file.exists(), "File should be cleaned up");
    
    Ok(())
}
```

## Best Practices for Integration Testing

### 1. Isolate Tests

Ensure each integration test runs in isolation to prevent test interference:

```rust
// Good: Create new fixtures for each test
#[test]
async fn test_one() -> TestResult<()> {
    let fixture = TestFixture::new().with_database().build()?;
    // Test implementation
    Ok(())
}

#[test]
async fn test_two() -> TestResult<()> {
    let fixture = TestFixture::new().with_database().build()?;
    // Test implementation
    Ok(())
}
```

### 2. Use Realistic Data

Where possible, use realistic data to ensure tests reflect actual usage:

```rust
// Configure database with realistic test data
db.expect_query("SELECT * FROM orders WHERE user_id = ?", |args| {
    Ok(MockQueryResult::new()
        .add_row({
            let mut row = HashMap::new();
            row.insert("id".to_string(), MockValue::Integer(1001));
            row.insert("product".to_string(), MockValue::String("Widget Model X".to_string()));
            row.insert("quantity".to_string(), MockValue::Integer(5));
            row.insert("price".to_string(), MockValue::Decimal("49.99".to_string()));
            row
        })
    )
});
```

### 3. Test Error Handling Thoroughly

Integration boundaries are common sources of errors, so test error cases thoroughly:

```rust
// Test all possible error scenarios
#[test]
async fn test_database_connection_failure() -> TestResult<()> {
    // Setup with connection failure
    Ok(())
}

#[test]
async fn test_database_query_error() -> TestResult<()> {
    // Setup with query error
    Ok(())
}

#[test]
async fn test_cache_connection_failure() -> TestResult<()> {
    // Setup with cache connection failure
    Ok(())
}
```

### 4. Use Timeouts for Long-Running Tests

Ensure tests don't hang indefinitely:

```rust
#[test]
async fn test_long_running_operation() -> TestResult<()> {
    let config = IntegrationTestConfig {
        timeout: Some(Duration::from_secs(30)),
        ..Default::default()
    };
    
    let runner = IntegrationRunner::new(config)?;
    
    runner.run_with_timeout(|_| async {
        // Long-running test...
        Ok(())
    }, Duration::from_secs(10)).await
}
```

### 5. Verify All Mock Expectations

Always verify that all mock expectations were met:

```rust
#[test]
async fn test_with_mocks() -> TestResult<()> {
    let fixture = TestFixture::new()
        .with_database()
        .with_cache()
        .build()?;
    
    // Test implementation
    
    // Always verify mock expectations at the end of the test
    fixture.verify()?;
    
    Ok(())
}
```

## Conclusion

The Cross-Crate Testing Infrastructure provides a comprehensive framework for implementing robust integration tests. By following the patterns and practices outlined in this guide, you can ensure that components from different crates work together correctly and that the interfaces between crates are well-tested.

For more information, refer to:
- [Test Migration Guide](./test-migration-guide.md)
- [Cross-Crate Testing Strategies](./cross-crate-testing-strategies.md)
- API documentation in the `navius_test` crate

---

*Updated: March 29, 2025* 