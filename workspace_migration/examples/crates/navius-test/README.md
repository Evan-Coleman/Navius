# Navius Test Framework

A comprehensive testing framework for the Navius ecosystem, providing tools for cross-crate testing, mocking, and error testing.

## Components

The framework consists of the following core components:

### Test Fixture

The `TestFixture` provides a way to set up and tear down test resources and dependencies. It manages the lifecycle of test components and ensures proper cleanup.

```rust
let fixture = TestFixture::new();
fixture.register_component(my_component);
fixture.register_resource(my_resource);
```

### Mock Registry

The `MockRegistry` allows registering and retrieving mock implementations of interfaces, setting expectations, and verifying calls.

```rust
let registry = MockRegistry::new();
registry.register::<dyn MyInterface, MockImplementation>(mock_impl);
registry.expect::<dyn MyInterface>("method_name").times(1);
```

### Test Harness

The `TestHarness` combines fixture and mock functionality to run tests with dependencies and mocks properly set up.

```rust
let harness = TestHarness::new();
harness.run_test(|| {
    // Test code here
    Ok(())
});
```

### Error Testing Framework

The `ErrorTesting` module provides tools for testing error handling, propagation, and recovery in your code.

```rust
// Create an error injection point
let mut injection = ErrorInjection::new("database_query")
    .inject()
    .with_message("Connection timeout");

// Use the injection to conditionally inject errors
let result = injection.check(|| database.query("SELECT * FROM users"));

// Track error propagation
let mut tracker = ErrorPropagationTracker::new();
tracker.add_component("UserService");

// Verify error handling
let verifier = ErrorVerifier::new()
    .expect_message("not found")
    .expect_component("UserService");
```

## Features

- **Component registration and retrieval**: Register test components and retrieve them by type
- **Resource management**: Register resources that need cleanup after tests
- **Mock expectations**: Set expectations on mock method calls and verify them
- **Synchronous and asynchronous tests**: Run both sync and async tests with the test harness
- **Error injection**: Simulate errors at specific points in your code
- **Error propagation tracking**: Track how errors propagate through your system
- **Error verification**: Verify that errors are handled correctly

## Usage

```rust
use navius_test::{TestHarness, assert_ok};

#[test]
fn test_my_feature() {
    let harness = TestHarness::new();
    
    harness.run_test(|| {
        // Set up test fixture and mocks
        let result = my_function();
        assert_ok!(result);
        
        Ok(())
    });
}
```

See the `examples` directory for more detailed usage examples:

- `basic_usage.rs`: Basic usage of the test fixture and harness
- `mock_example.rs`: Example of using the mock registry
- `error_testing.rs`: Example of using the error testing framework

## Error Testing Macros

The framework provides several macros for error testing:

- `assert_ok!(expr)`: Assert that a result is Ok and return the unwrapped value
- `assert_err!(expr)`: Assert that a result is Err and return the unwrapped error
- `assert_err_variant!(expr, pattern)`: Assert that a result is Err and matches a specific pattern
- `assert_injected_error!(expr, message)`: Assert that an error was injected with a specific message
- `verify_error_path!(tracker, components...)`: Verify that an error passed through specific components
- `verify_error_context!(tracker, key, value)`: Verify that an error has specific context information

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
navius-test = { path = "../navius-test" }
```

## License

This project is licensed under the same terms as the Navius project. 