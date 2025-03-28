---
title: Navius Testing Guide
description: Comprehensive guide to testing Navius applications
category: guides
tags:
  - testing
  - unit-tests
  - integration-tests
  - test-coverage
related:
  - development-workflow.md
  - ../../reference/standards/naming-conventions.md
  - ../../contributing/test-implementation-template.md
last_updated: March 23, 2025
version: 1.0
---

# Navius Testing Guide

## Overview
This guide outlines the testing methodology, tools, and best practices used in Navius applications. It covers different testing approaches, from unit testing to integration testing, as well as utilities and patterns that make testing easier and more effective.

## Prerequisites
Before using this testing guide, ensure you have:

- Basic understanding of Rust testing fundamentals
- Navius development environment set up
- Familiarity with common testing concepts (unit tests, integration tests)

## Testing Methodology

### Testing Pyramid

Navius follows a testing pyramid approach with:

1. **Unit Tests**: Testing individual components in isolation
2. **Integration Tests**: Testing interactions between components
3. **End-to-End Tests**: Testing the entire application flow

This approach ensures comprehensive test coverage while maintaining fast test execution.

## Step-by-step Testing

### 1. Running Tests

Navius provides several commands for running different types of tests:

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run a specific test or test module
cargo test test_name

# Run tests with output shown
RUST_LOG=debug cargo test -- --nocapture
```

### 2. Writing Unit Tests

Unit tests are located alongside the code they test, in the same file, inside a `tests` module:

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
```

For async tests, use the `#[tokio::test]` attribute:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_function() {
        let result = fetch_data().await;
        assert!(result.is_ok());
    }
}
```

### 3. Writing Integration Tests

Integration tests are located in the `tests/` directory at the root of the project:

```
tests/
  ├── api/
  │    ├── auth_tests.rs
  │    └── user_tests.rs
  ├── database/
  │    └── migrations_tests.rs
  └── common/
       └── test_utils.rs
```

Example integration test for an API endpoint:

```rust
use navius::test::TestApp;

#[tokio::test]
async fn test_create_user() {
    // Create a test application with an in-memory database
    let app = TestApp::new().await;
    
    // Send a request to the API
    let response = app
        .post("/api/users")
        .json(&json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), 201);
    
    let user: User = response.json().await;
    assert_eq!(user.email, "test@example.com");
}
```

### 4. Using Test Utilities

Navius provides several testing utilities to simplify writing tests:

#### TestApp

The `TestApp` struct helps create a test instance of the application:

```rust
use navius::test::TestApp;

#[tokio::test]
async fn test_get_user() {
    // Create a test application
    let app = TestApp::new().await;
    
    // Create a test user
    let user_id = app.create_test_user("test@example.com", "password123").await;
    
    // Make a request to the API
    let response = app
        .get(&format!("/api/users/{}", user_id))
        .with_auth()
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
    
    let user: User = response.json().await;
    assert_eq!(user.email, "test@example.com");
}
```

#### Mocking

Navius uses `mockall` for creating mock objects:

```rust
use mockall::predicate::*;
use mockall::mock;

// Define a mock for UserRepository
mock! {
    UserRepository {}
    
    impl UserRepository for UserRepository {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DatabaseError>;
        async fn create(&self, user: NewUser) -> Result<User, DatabaseError>;
    }
}

#[tokio::test]
async fn test_user_service() {
    // Create a mock repository
    let mut mock_repo = MockUserRepository::new();
    
    // Set expectations
    mock_repo.expect_find_by_id()
        .with(eq(user_id))
        .times(1)
        .returning(|_| Ok(Some(User {
            id: user_id,
            email: "test@example.com".to_string(),
            // ...other fields
        })));
    
    // Create the service with the mock
    let service = UserService::new(mock_repo);
    
    // Test the service
    let user = service.get_user(user_id).await.unwrap();
    assert_eq!(user.email, "test@example.com");
}
```

#### Database Testing

For database tests, Navius provides utilities for creating test databases:

```rust
use navius::test::TestDb;

#[tokio::test]
async fn test_database_operations() {
    // Create a test database
    let db = TestDb::new().await;
    
    // Run migrations
    db.migrate().await;
    
    // Get a connection
    let conn = db.get_connection().await;
    
    // Perform database operations
    let user = sqlx::query_as::<_, User>("INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *")
        .bind("test@example.com")
        .bind("hashed_password")
        .fetch_one(&conn)
        .await
        .unwrap();
    
    assert_eq!(user.email, "test@example.com");
}
```

### 5. Property-Based Testing

Navius supports property-based testing with `proptest`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_add_commutative(a in 0..100i32, b in 0..100i32) {
        assert_eq!(add(a, b), add(b, a));
    }
}
```

### 6. Test Coverage

To generate test coverage reports:

```bash
# Install tarpaulin if not already installed
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html

# Generate JSON coverage report
cargo tarpaulin -o Json --output-file target/@navius-coverage.json
```

Coverage targets:
- Overall project: 80%+ code coverage
- Critical modules: 90%+ coverage
- Helper/utility functions: 70%+ coverage

## Best Practices

### General Testing Practices

1. **Write tests first**: Follow Test-Driven Development when possible
2. **Focus on behavior**: Test what the code does, not how it's implemented
3. **Keep tests isolated**: Each test should not depend on others
4. **Use descriptive names**: Test names should describe what is being tested
5. **One assertion per test**: When possible, test one thing per test function
6. **Clean up after tests**: Ensure tests don't leave behind test data
7. **Test edge cases**: Test boundary conditions and error cases

### Async Testing

1. Use `#[tokio::test]` for async tests
2. Be aware of race conditions in async tests
3. Use timeouts for tests that might hang

### Mock Testing

1. Mock at the interface boundary, not implementation details
2. Verify mock expectations when needed
3. Use `#[cfg_attr(test, mockall::automock)]` for trait mocking

## Troubleshooting

### Common Issues

1. **Failing async tests**: Ensure you're using `#[tokio::test]` for async tests
2. **Database errors**: Check that migrations are being run in your test setup
3. **Race conditions**: Use synchronization primitives for concurrent tests
4. **Slow tests**: Use `cargo test -- --nocapture` to see where tests are slow

### Debug Output

To enable debug logging during tests:

```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

## Advanced Testing Techniques

### Snapshot Testing

Navius supports snapshot testing for stable API responses:

```rust
use navius::test::{TestApp, snapshot};

#[tokio::test]
async fn test_api_response_format() {
    let app = TestApp::new().await;
    
    // Create test data
    let user_id = app.create_test_user("test@example.com", "password123").await;
    
    // Get the API response
    let response = app
        .get(&format!("/api/users/{}", user_id))
        .send()
        .await;
    
    let body = response.json::<serde_json::Value>().await;
    
    // Compare with stored snapshot
    snapshot::assert_json_snapshot!("user_response", body);
}
```

### Performance Testing

For basic performance testing:

```rust
use navius::test::{TestApp, performance};

#[tokio::test]
async fn test_endpoint_performance() {
    let app = TestApp::new().await;
    
    // Run performance test
    let results = performance::benchmark(|| async {
        app.get("/api/users").send().await
    }, 100).await;
    
    assert!(results.avg_duration < std::time::Duration::from_millis(100));
    assert!(results.p95_duration < std::time::Duration::from_millis(200));
}
```

## Related Documents

- [Development Workflow](development-workflow.md) - Development process overview
- [Test Implementation Template](../../contributing/test-implementation-template.md) - Template for writing tests
- [API Integration Guide](../features/api-integration.md) - Testing API integrations
- [Naming Conventions](../../reference/standards/naming-conventions.md) - Conventions for naming tests 