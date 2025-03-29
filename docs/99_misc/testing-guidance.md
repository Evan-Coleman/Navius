---
title: Navius Testing Guidelines
description: Comprehensive guide for testing practices in the Navius codebase
category: Development
tags: [testing, quality, guidelines]
last_updated: March 27, 2025
version: 1.0.0
---

# Navius Testing Guidelines

Version: 1.0.0
Last Updated: March 26, 2025

## Introduction

This document provides comprehensive guidance for writing and maintaining tests in the Navius codebase. Following these guidelines ensures consistency, reliability, and maintainability across our test suite.

## General Testing Philosophy

- **Focused Tests**: Each test should verify one clear behavior or aspect of functionality
- **AAA Pattern**: Structure tests with clear Arrange, Act, and Assert sections
- **Isolation**: Tests should be isolated and not depend on the order of execution
- **Coverage Goals**: Aim for 80% coverage minimum, with critical security and core logic paths at 100%
- **Realistic Testing**: Tests should reflect real-world usage patterns and edge cases
- **Maintainability**: Tests should be easy to understand and maintain

## Test Organization

### Unit Tests

Place unit tests in the same file as the implementation using a `#[cfg(test)]` module:

```rust
// Implementation
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
```

### Integration Tests

Place integration tests in the `tests` directory with a clear module structure:

```
tests/
├── integration_tests/
│   ├── cli/
│   │   ├── interactive_tests.rs
│   │   └── command_tests.rs
│   ├── api/
│   │   ├── resources_tests.rs
│   │   └── auth_tests.rs
│   └── mod.rs
├── common/
│   ├── fixtures.rs
│   ├── mocks.rs
│   └── mod.rs
└── lib.rs
```

## Test Types

### Unit Tests

Test individual functions and methods in isolation:

```rust
#[test]
fn test_feature_registry_add() {
    let mut registry = FeatureRegistry::new();
    let feature = Feature::new("test-feature", "1.0.0");
    
    registry.add(feature.clone());
    
    assert!(registry.contains(&feature.id));
    assert_eq!(registry.get(&feature.id).unwrap(), &feature);
}
```

### Integration Tests

Test interactions between modules:

```rust
#[tokio::test]
async fn test_feature_cli_load_and_activate() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    
    // Write test configuration
    write_test_config(&config_path).await.unwrap();
    
    // Run CLI command
    let result = Command::cargo_bin("feature-cli")
        .arg("activate")
        .arg("test-feature")
        .arg("--config")
        .arg(&config_path)
        .assert()
        .success();
        
    // Verify feature was activated in config
    let config = read_config(&config_path).await.unwrap();
    assert!(config.is_feature_active("test-feature"));
}
```

### Doc Tests

Include executable examples in documentation:

```rust
/// Returns the sum of two integers
///
/// # Examples
///
/// ```
/// use navius::math::add;
/// assert_eq!(add(2, 3), 5);
/// assert_eq!(add(-1, 1), 0);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Property Tests

Use property-based testing for functions with many input variations:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_feature_serialization_roundtrip(
        name in "[a-z][a-z0-9-]{1,63}",
        version in "[0-9]+(\\.[0-9]+){0,2}",
        description in ".*",
    ) {
        let feature = Feature::new(name.clone(), version.clone())
            .with_description(description.clone());
            
        let serialized = serde_json::to_string(&feature).unwrap();
        let deserialized: Feature = serde_json::from_str(&serialized).unwrap();
        
        prop_assert_eq!(feature, deserialized);
    }
}
```

## Test Coverage

Our current coverage is 31.97%, with a target of 70% overall. Critical components should aim for:
- Core service modules: 80%
- Command-line tools: 75%
- Documentation system: 90%
- Feature system: 85%

### Checking Coverage

Use our coverage tool to track test coverage:

```bash
# Check overall coverage
.devtools/scripts/check_coverage.sh

# Check coverage for a specific module
.devtools/scripts/check_coverage.sh --module feature_system
```

## Test Utilities

### Common Test Fixtures

Use fixtures for common test setup:

```rust
// In tests/common/fixtures.rs
pub fn setup_test_environment() -> TestEnvironment {
    TestEnvironment {
        temp_dir: TempDir::new().unwrap(),
        config: default_test_config(),
        feature_registry: test_feature_registry(),
    }
}

// In your test
use crate::common::fixtures::setup_test_environment;

#[test]
fn test_with_environment() {
    let env = setup_test_environment();
    // Use env in your test
}
```

### Mocking

Use trait-based mocking for dependencies:

```rust
// Define a mock implementation
pub struct MockFeatureLoader {
    pub features_to_return: Vec<Feature>,
}

impl FeatureLoader for MockFeatureLoader {
    fn load_features(&self) -> Result<Vec<Feature>, Error> {
        Ok(self.features_to_return.clone())
    }
}

// Use in tests
#[test]
fn test_with_mock() {
    let mock_loader = MockFeatureLoader {
        features_to_return: vec![
            Feature::new("feature-1", "1.0.0"),
            Feature::new("feature-2", "1.0.0"),
        ],
    };
    
    let service = FeatureService::new(mock_loader);
    let features = service.list_features().unwrap();
    
    assert_eq!(features.len(), 2);
}
```

## Async Testing

Use `#[tokio::test]` for async tests:

```rust
#[tokio::test]
async fn test_async_feature_loading() {
    let loader = AsyncFeatureLoader::new();
    
    let features = loader.load_features().await.unwrap();
    
    assert!(!features.is_empty());
}
```

For testing timeouts:

```rust
#[tokio::test(flavor = "multi_thread")]
#[timeout(1000)]
async fn test_operation_completes_within_timeout() {
    let result = tokio::time::timeout(
        Duration::from_millis(500),
        some_async_operation()
    ).await;
    
    assert!(result.is_ok());
}
```

## Error Handling Tests

Test both happy paths and error cases:

```rust
#[test]
fn test_feature_loading_error_handling() {
    // Test invalid path
    let result = FeatureLoader::from_path("/nonexistent/path");
    assert!(result.is_err());
    match result {
        Err(FeatureError::IoError(_)) => (), // Expected
        other => panic!("Unexpected result: {:?}", other),
    }
    
    // Test invalid format
    let temp_dir = TempDir::new().unwrap();
    let invalid_file = temp_dir.path().join("invalid.json");
    fs::write(&invalid_file, "not json").unwrap();
    
    let result = FeatureLoader::from_path(invalid_file);
    assert!(result.is_err());
    match result {
        Err(FeatureError::ParseError(_)) => (), // Expected
        other => panic!("Unexpected result: {:?}", other),
    }
}
```

## CLI Testing

Test command-line interfaces with clear arguments and assertions:

```rust
#[test]
fn test_cli_help_command() {
    let output = Command::cargo_bin("feature-cli")
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
        
    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("USAGE:"));
    assert!(stdout.contains("feature-cli"));
}
```

## Test Naming Conventions

Use descriptive names that explain the scenario and expected outcome:

- Format: `test_[unit]_[scenario]_[expected]`
- Examples:
  - `test_feature_validation_rejects_invalid_name`
  - `test_config_loading_from_nonexistent_file_returns_error`
  - `test_authentication_with_valid_token_succeeds`

## Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test --package navius-core --lib features

# Run a specific test
cargo test test_feature_registry_add

# Run tests with output
cargo test -- --nocapture

# Run tests with coverage
cargo tarpaulin
```

## CI Integration

All tests are automatically run in the CI pipeline:

1. Tests must pass before a PR can be merged
2. Coverage reports are generated and stored as artifacts
3. Coverage decreases will trigger a warning in the PR

## Advanced Testing Scenarios

### Concurrency Testing

Test concurrent operations:

```rust
#[tokio::test]
async fn test_concurrent_feature_access() {
    let registry = Arc::new(FeatureRegistry::new());
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let registry = Arc::clone(&registry);
            tokio::spawn(async move {
                let feature = Feature::new(format!("feature-{}", i), "1.0.0");
                registry.add(feature);
            })
        })
        .collect();
        
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    assert_eq!(registry.count(), 10);
}
```

### Performance Testing

Use benchmarks for performance testing:

```rust
#[bench]
fn bench_feature_loading(b: &mut test::Bencher) {
    let temp_dir = TempDir::new().unwrap();
    create_test_features(&temp_dir, 100);
    
    b.iter(|| {
        let loader = FeatureLoader::from_path(temp_dir.path()).unwrap();
        loader.load_features().unwrap()
    });
}
```

## Best Practices

1. **Test Behavior, Not Implementation**: Focus on testing the behavior of your code, not its internal implementation.
2. **Test Edge Cases**: Include tests for boundary conditions and edge cases.
3. **Test Regression**: Add tests for any bugs fixed to prevent them from recurring.
4. **Keep Tests Fast**: Optimize test execution time to encourage frequent running.
5. **Readable Tests**: Write clear, readable tests that serve as documentation.
6. **Don't Test External Libraries**: Focus on testing your code, not third-party libraries.
7. **Avoid Test Interdependence**: Each test should be runnable in isolation.
8. **Clean Up After Tests**: Ensure tests clean up any resources they allocate.
9. **Minimize Mocking**: Use mocks sparingly to avoid brittle tests.
10. **Review Test Code**: Review test code with the same care as production code.

## Common Testing Patterns

### Table-Driven Tests

Use table-driven tests for similar test cases:

```rust
#[test]
fn test_feature_validation() {
    let test_cases = vec![
        // (name, version, expected_valid)
        ("valid-feature", "1.0.0", true),
        ("", "1.0.0", false),  // Empty name
        ("invalid~name", "1.0.0", false),  // Invalid character
        ("valid-feature", "", false),  // Empty version
        ("valid-feature", "1.a.0", false),  // Invalid version
    ];
    
    for (name, version, expected_valid) in test_cases {
        let result = Feature::validate(name, version);
        assert_eq!(result.is_ok(), expected_valid, 
            "Failed for name: {}, version: {}", name, version);
    }
}
```

### Snapshot Testing

Use snapshot testing for complex output:

```rust
#[test]
fn test_feature_json_output() {
    let feature = Feature::new("test-feature", "1.0.0")
        .with_description("Test feature")
        .with_tag("test");
        
    let json = serde_json::to_string_pretty(&feature).unwrap();
    
    // Compare with stored snapshot
    insta::assert_snapshot!(json);
}
```

## Security Testing

Ensure proper security testing for:

1. **Authentication**: Test that unauthenticated requests are rejected
2. **Authorization**: Test that users can only access resources they have permission for
3. **Input Validation**: Test that malicious inputs are rejected
4. **Data Protection**: Test that sensitive data is properly protected

```rust
#[test]
fn test_unauthorized_access_is_rejected() {
    let service = FeatureService::new(MockFeatureLoader::default());
    
    // No authentication token
    let result = service.get_protected_feature("sensitive-feature");
    
    assert_eq!(result.unwrap_err(), FeatureError::Unauthorized);
}
```

## Conclusion

Following these testing guidelines will help maintain a high-quality codebase with reliable functionality. Remember that tests are a critical part of our development process, not an afterthought.

For questions or suggestions about these guidelines, please contact the core team. 