# Navius Testing Guide

This guide outlines the testing methodology and tools used in the Navius project. We prioritize automated testing to ensure code quality and reliability.

## Testing Pyramid

Navius follows a testing pyramid approach with:

1. **Unit Tests**: Testing individual components in isolation
2. **Integration Tests**: Testing interactions between components
3. **End-to-End Tests**: Testing the entire application flow

## Running Tests

### All Tests

To run all tests:

```bash
cargo test
```

### Unit Tests Only

To run only unit tests:

```bash
cargo test --lib
```

### Integration Tests Only

To run only integration tests:

```bash
cargo test --test '*'
```

### Individual Tests

To run a specific test or test module:

```bash
cargo test test_name
```

### With Logging

To see test output, use:

```bash
RUST_LOG=debug cargo test -- --nocapture
```

## Test Structure

### Unit Tests

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

### Integration Tests

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

## Testing Utilities

Navius provides several testing utilities to simplify writing tests:

### TestApp

The `TestApp` struct helps you create a test instance of the application:

```rust
use navius::test::TestApp;

#[tokio::test]
async fn test_get_user() {
    // Create a test application with an in-memory database
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

### Mocking

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

### Database Testing

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

### API Testing

For testing API endpoints:

```rust
use navius::test::TestApp;

#[tokio::test]
async fn test_create_user() {
    let app = TestApp::new().await;
    
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

## Test Fixtures

Navius provides fixtures for common test data:

```rust
use navius::test::fixtures;

#[tokio::test]
async fn test_with_fixtures() {
    let app = TestApp::new().await;
    
    // Use a user fixture
    let user = fixtures::users::create_test_user(&app).await;
    
    // Use a post fixture with the user
    let post = fixtures::posts::create_test_post(&app, user.id).await;
    
    // Test with the fixtures
    let response = app
        .get(&format!("/api/posts/{}", post.id))
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
}
```

## Property-Based Testing

Navius uses `proptest` for property-based testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_add_commutative(a in 0..100i32, b in 0..100i32) {
        assert_eq!(add(a, b), add(b, a));
    }
}
```

## Coverage

To generate test coverage reports:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

This will generate an HTML report in the `tarpaulin-report.html` file.

## Best Practices

1. **Write tests first**: Follow TDD when possible
2. **Focus on behavior**: Test what the code does, not how it's implemented
3. **Keep tests isolated**: Each test should not depend on others
4. **Use descriptive names**: Test names should describe what is being tested
5. **One assertion per test**: When possible, test one thing per test function
6. **Clean up after tests**: Ensure tests don't leave behind test data
7. **Test edge cases**: Test boundary conditions and error cases

## Continuous Integration

Navius runs tests automatically in CI on:
- Every pull request
- Every merge to main
- Nightly for regression testing

See `.github/workflows/ci.yml` for CI configuration details.

## Troubleshooting Tests

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

## Creating Mocks

Navius uses `mockall` for creating mocks. For structs that use generics or lifetimes, use:

```rust
#[cfg_attr(test, mockall::automock)]
pub trait UserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DatabaseError>;
    async fn create(&self, user: NewUser) -> Result<User, DatabaseError>;
}
```

Then in tests:

```rust
use crate::mocks::MockUserRepository;

#[tokio::test]
async fn test_with_mock() {
    let mut mock_repo = MockUserRepository::new();
    
    // Configure mock
    mock_repo.expect_find_by_id()
        .returning(|_| Ok(Some(user)));
    
    // Use the mock
    let service = UserService::new(mock_repo);
    let result = service.get_user(uuid::Uuid::new_v4()).await;
    
    assert!(result.is_ok());
}
```

## Integration with External Services

For integration tests with external services:

```rust
use navius::test::external::{MockSmtpServer, MockPaymentGateway};

#[tokio::test]
async fn test_payment_processing() {
    // Start a mock payment gateway
    let mock_gateway = MockPaymentGateway::start().await;
    
    // Configure the mock
    mock_gateway.expect_payment()
        .with_amount(100)
        .respond_with(PaymentResponse::success());
    
    // Create app with the mock URL
    let app = TestApp::new()
        .with_payment_url(mock_gateway.url())
        .await;
    
    // Test payment flow
    let response = app
        .post("/api/payments")
        .json(&json!({
            "amount": 100,
            "card_token": "tok_visa"
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
    
    // Verify mock was called correctly
    mock_gateway.verify().await;
}
```

## Snapshot Testing

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

## Performance Testing

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

## Conclusion

Testing is a core part of the Navius development process. By following these guidelines, we maintain high code quality and ensure the framework is reliable and robust. For any questions about testing, please reach out to the development team. 