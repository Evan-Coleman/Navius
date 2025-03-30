# Navius Test Framework

The Navius Test Framework provides utilities, fixtures, and patterns for testing interactions between different crates in the Navius workspace. This is a critical component for ensuring that the interfaces between crates function as expected and that integration scenarios are thoroughly tested.

## Features

- **Test Fixture Framework** - Standardized test setup and teardown
- **Mock Implementation Registry** - Registry for mock implementations of core interfaces
- **Integration Test Utilities** - Utilities designed specifically for integration testing
- **Error Testing Framework** - Specialized utilities for testing error scenarios

## Getting Started

### Basic Usage

```rust
use navius_test::prelude::*;

#[test]
fn test_component_interaction() {
    // Create a test fixture
    let fixture = TestFixture::new()
        .with_component(mock_database())
        .with_component(mock_cache())
        .build();
    
    // Get components from the fixture
    let db = fixture.get::<MockDatabase>().unwrap();
    let cache = fixture.get::<MockCache>().unwrap();
    
    // Test interaction between components
    // ...
}
```

### Using the Test Harness

The test harness provides a more comprehensive testing environment:

```rust
use navius_test::prelude::*;

#[test]
fn test_with_harness() -> TestResult<()> {
    // Create a test harness
    let mut harness = TestHarnessBuilder::new()
        .with_runtime()
        .build()?;
    
    // Run a test with the harness
    harness.run(|fixture, mock_registry| {
        // Register mocks
        let mock_db = MockDatabase::new();
        mock_registry.register::<dyn DatabaseProvider, MockDatabase>(mock_db.clone())?;
        
        // Register components
        let repository = UserRepository::new(mock_db);
        fixture.register(repository)?;
        
        // Test component behavior
        let repo = fixture.get::<UserRepository<MockDatabase>>()?;
        let user = repo.get_user("1")?;
        
        assert_eq!(user.name, "Test User");
        
        Ok(())
    })?;
    
    Ok(())
}
```

### Async Testing

The framework also supports async tests:

```rust
#[tokio::test]
async fn test_async_component() -> TestResult<()> {
    let mut harness = TestHarnessBuilder::new()
        .with_runtime()
        .build()?;
    
    harness.run_async(|fixture, mock_registry| {
        Box::pin(async move {
            // Test async behavior
            // ...
            Ok(())
        })
    })?;
    
    Ok(())
}
```

## Core Components

### TestFixture

The `TestFixture` manages test resources and components:

- Register components for testing
- Retrieve components by type
- Manage test-specific resources like temporary directories
- Automatic cleanup on test completion

### MockRegistry

The `MockRegistry` manages mock implementations of interfaces:

- Register mock implementations for interfaces
- Retrieve mock implementations by interface type
- Clear mocks between tests

### TestHarness

The `TestHarness` provides a complete test environment:

- Combines fixture and mock registry
- Supports both sync and async tests
- Manages Tokio runtime for async tests
- Handles test setup and teardown

## Examples

See the `examples` directory for more comprehensive examples of using the test framework:

- `basic_test.rs` - Basic usage of the test framework
- `async_test.rs` - Testing async components
- `error_propagation_test.rs` - Testing error handling across components
- `multi_crate_test.rs` - Testing interactions between components in different crates

## Best Practices

1. **Isolate Tests** - Each test should create its own fixture or harness
2. **Clean Up Resources** - Ensure tear_down() is called when manually managing fixtures
3. **Use Type-Safe Access** - Access components by their concrete types
4. **Test Error Paths** - Test both success and error paths
5. **Mock External Services** - Use mocks for external dependencies

## License

MIT OR Apache-2.0 