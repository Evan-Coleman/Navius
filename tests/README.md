# Tests

This directory contains all tests for the Navius application, organized by test type.

## Directory Structure

- `common/` - Shared test utilities and fixtures used across different test types
- `integration/` - Integration tests that test multiple components working together
- `unit/` - Unit tests that test individual components in isolation
- `resources/` - Test resources such as configuration files, test data, and Docker configurations
- `utils/` - Utility functions and helpers specific to tests

## Test Types

### Integration Tests

Integration tests verify that different parts of the application work together correctly. These tests typically involve multiple components, such as API endpoints, services, and repositories.

Files:
- `integration/e2e_tests.rs` - End-to-end tests for the entire application
- `integration/api_client_integration_tests.rs` - Tests for the API client
- `integration/router_integration_test.rs` - Tests for the router

### Unit Tests

Unit tests verify that individual components work correctly in isolation. Most unit tests are located alongside the code they test in the source directory, but some cross-component unit tests may be placed here.

## Running Tests

All tests can be run with:

```
cargo test
```

To run specific test types:

```
# Run integration tests only
cargo test --test e2e_tests

# Run a specific test
cargo test test_actuator_endpoint
```

## Test Utilities

Common test utilities are located in the `common` directory. These include:

- Mock implementations for testing
- Test helper functions
- Shared fixtures and setup code

## Adding New Tests

1. Place integration tests in the `integration/` directory
2. Place unit tests alongside the code they test in the source directory
3. If a unit test doesn't fit well in the source directory, place it in the `unit/` directory
4. Place test resources in the `resources/` directory
5. Follow the naming conventions established in existing tests 

## PropTest Regression Files

The `proptest-regressions` directory contains saved failure cases that the PropTest library has discovered during property-based testing. These files:

- Are automatically generated when PropTest finds a failing test case
- Store the "seeds" for reproducing those failures 
- Allow PropTest to re-run previously failed cases before generating new ones
- Should be committed to source control

For more information on property testing in this project, see the property-based tests in various modules such as `src/cache/providers/property_tests.rs`. 