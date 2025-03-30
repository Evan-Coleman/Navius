# Migrating Tests to the Cross-Crate Testing Infrastructure

## Overview

This guide provides instructions for migrating existing tests to use our new Cross-Crate Testing Infrastructure. The infrastructure provides robust tools for testing components across crate boundaries, ensuring that interfaces remain compatible and robust throughout the Navius workspace migration process.

## Core Components

Our new testing infrastructure consists of the following core components:

1. **TestFixture**: Manages test dependencies and resources
2. **MockRegistry**: Handles registration and retrieval of mock implementations
3. **TestHarness**: Integrates TestFixture and MockRegistry for running tests
4. **Error Testing Framework**: Provides error injection and verification
5. **Mock Interface Registry**: Contains mock implementations for common interfaces
6. **Integration Test Utilities**: Supports testing across crate boundaries

## Migration Steps

Follow these steps to migrate your existing tests:

### 1. Update Imports

Add imports for the navius-test crate components:

```rust
use navius_test::{
    error::{TestResult, assert_ok, assert_err, assert_true, assert_false},
    fixture::TestFixture,
    harness::{TestHarness, TestOptions},
    mock::MockRegistry,
    mocks::{
        // Import specific mocks based on your needs
        database::{DatabaseClient, MockDatabaseClient, MockQueryResult, MockValue},
        filesystem::{FileSystem, MockFileSystem},
        // Other mocks as needed
        MockFixture,
    },
};
```

### 2. Change Return Type

Update test functions to return `TestResult<()>` instead of `()`:

```rust
// Before
#[test]
async fn test_example() {
    // Test code
}

// After
#[test]
async fn test_example() -> TestResult<()> {
    // Test code
    Ok(())
}
```

### 3. Create Test Fixture

Replace mock creation with TestFixture:

```rust
// Before
let mock_db = MockDatabase::new();
mock_db.expect_query("SELECT * FROM table", Ok(result));

// After
let fixture = MockFixture::new();
let db = fixture.database();
db.expect_query(
    "SELECT * FROM table", 
    Ok(MockQueryResult::new().add_row({
        let mut row = HashMap::new();
        row.insert("id".to_string(), MockValue::Integer(1));
        row.insert("name".to_string(), MockValue::String("Test".to_string()));
        row
    })),
);
```

### 4. Use Test Harness (Optional)

For complex tests, use the TestHarness to manage test lifecycle:

```rust
let options = TestOptions {
    verify_mocks: true,
    cleanup_resources: true,
    timeout: Some(Duration::from_secs(5)),
};

let harness = TestHarness::new()
    .with_options(options)
    .with_subject(system_under_test)?;

harness.run(|subject| async move {
    // Test code using subject
    let result = subject.some_method().await;
    assert_ok(result, "Method should succeed")?;
    
    Ok(())
}).await
```

### 5. Replace Assertions

Replace standard assertions with test framework assertions:

```rust
// Before
assert!(result.is_ok());
assert_eq!(value, expected);

// After
assert_ok(result, "Operation should succeed")?;
assert_eq(value, expected, "Values should match")?;
```

### 6. Verify Mock Expectations

Add verification of mock expectations at the end of your tests:

```rust
// Verify that all mock expectations were met
fixture.verify()?;
```

## Example: Before and After

### Before

```rust
#[test]
async fn test_database_query() {
    // Create mock database
    let mock_db = MockDatabase::new();
    mock_db.expect_query("SELECT * FROM users", Ok(vec![user1, user2]));
    
    // Create service with mock
    let service = UserService::new(mock_db);
    
    // Execute test
    let users = service.get_all_users().await;
    
    // Verify result
    assert!(users.is_ok());
    assert_eq!(users.unwrap().len(), 2);
}
```

### After

```rust
#[test]
async fn test_database_query() -> TestResult<()> {
    // Create test fixture
    let fixture = MockFixture::new();
    
    // Configure mock database
    let db = fixture.database();
    db.expect_query(
        "SELECT * FROM users",
        Ok(MockQueryResult::new()
            .add_row({
                let mut row = HashMap::new();
                row.insert("id".to_string(), MockValue::Integer(1));
                row.insert("name".to_string(), MockValue::String("User 1".to_string()));
                row
            })
            .add_row({
                let mut row = HashMap::new();
                row.insert("id".to_string(), MockValue::Integer(2));
                row.insert("name".to_string(), MockValue::String("User 2".to_string()));
                row
            })),
    );
    
    // Create service with mock
    let service = UserService::new(db);
    
    // Execute test
    let users = service.get_all_users().await;
    
    // Verify result
    assert_ok(&users, "Getting users should succeed")?;
    assert_eq(users.unwrap().len(), 2, "Should return 2 users")?;
    
    // Verify all mock expectations were met
    fixture.verify()?;
    
    Ok(())
}
```

## Common Patterns

### Testing Error Scenarios

Use the error injection framework to test error handling:

```rust
// Configure mock to return an error
db.expect_query(
    "SELECT * FROM users",
    Err(MockDatabaseError::new("Database connection failed")),
);

// Execute test and verify error handling
let result = service.get_all_users().await;
assert_err(result, "Service should handle database errors")?;
```

### Testing Multiple Components

Test interactions between components using the same fixture:

```rust
let fixture = MockFixture::new();
let db = fixture.database();
let fs = fixture.filesystem();

// Configure both mocks
db.expect_query(...);
fs.expect_read_file(...);

// Test service that uses both components
let service = Service::new(db, fs);
let result = service.process_data().await;

// Verify results and mock expectations
assert_ok(result, "Processing should succeed")?;
fixture.verify()?;
```

### Integration Testing Across Crates

Use the integration test utilities for testing across crate boundaries:

```rust
// Create integration context
let config = IntegrationTestConfig {
    name: "cross-crate-test".to_string(),
    verify_mocks: true,
    cleanup_resources: true,
    timeout: Some(Duration::from_secs(10)),
    ..Default::default()
};

let runner = IntegrationRunner::new(config)?;

// Run test with context
runner.run(|context| async move {
    // Test code using context
    let db = context.registry().get::<dyn DatabaseClient, MockDatabaseClient>()?;
    // Configure mock and execute test
    
    Ok(())
}).await
```

## Troubleshooting

### Missing Mocks

If you need a mock that isn't provided, implement it and register it with the MockRegistry:

```rust
let registry = MockRegistry::new();
let custom_mock = MyCustomMock::new();
custom_mock.register(&registry)?;
```

### Type Errors with Mocks

Ensure you're using the correct interfaces and types:

```rust
// Use the trait type when retrieving from registry
let registry = fixture.registry();
let db = registry.get::<dyn DatabaseClient, MockDatabaseClient>()?;

// Or use the mock fixture helpers
let db = fixture.database();
```

### TestResult Propagation

Remember that functions returning `TestResult<T>` require `?` operator for error propagation:

```rust
// This works
let value = function_returning_result()?;

// This fails to propagate errors
let value = function_returning_result(); // Error: doesn't handle the Result
```

## Next Steps

After migrating your tests:

1. Run your test suite to ensure all tests pass with the new infrastructure
2. Consider adding more comprehensive tests using the new capabilities
3. Use the error injection framework to test error handling more thoroughly
4. Add integration tests that verify cross-crate interactions

For more information, refer to:
- API documentation in `navius_test` crate
- Example tests in `navius-test/examples/`
- Integration testing guide in `docs/testing/integration-testing.md`

## Support

If you encounter any issues during migration, please contact the Testing Infrastructure team or file an issue in the issue tracker.

---
*Updated: March 29, 2025* 