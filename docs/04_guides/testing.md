# Testing

This guide explains the testing approach in Navius, covering different types of tests, tools, and best practices for ensuring application quality.

## Testing Philosophy

Navius follows these testing principles:

1. **Test-driven development**: Tests should guide implementation
2. **Comprehensive coverage**: Test all critical code paths
3. **Isolation**: Tests should be independent and not affect each other
4. **Representative**: Tests should reflect real-world scenarios
5. **Fast feedback**: Tests should run quickly to support development cycles

## Test Types

### Unit Tests

Unit tests verify individual components in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com"));
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email(""));
    }
}
```

### Integration Tests

Integration tests verify interactions between components:

```rust
// In tests/api_tests.rs
use navius::app::build_app;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_get_user_api() {
    // Setup
    let app = build_app().await;
    
    // Execute
    let response = app
        .oneshot(
            Request::builder()
                .uri("/users/123")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Verify
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(user["id"], "123");
    assert_eq!(user["name"], "Test User");
}
```

### End-to-End Tests

End-to-end tests verify the entire application stack:

```rust
// In tests/e2e/user_flow_test.rs
use navius::app::start_test_server;
use reqwest::Client;

#[tokio::test]
async fn test_user_creation_flow() {
    // Start the server with a test database
    let server = start_test_server().await;
    let client = Client::new();
    
    // Create a user
    let user_data = json!({
        "name": "New User",
        "email": "new@example.com",
        "password": "password123"
    });
    
    let create_response = client
        .post(&format!("http://{}/users", server.address()))
        .json(&user_data)
        .send()
        .await
        .unwrap();
        
    assert_eq!(create_response.status(), 201);
    
    let created_user: serde_json::Value = create_response.json().await.unwrap();
    let user_id = created_user["id"].as_str().unwrap();
    
    // Get the user
    let get_response = client
        .get(&format!("http://{}/users/{}", server.address(), user_id))
        .send()
        .await
        .unwrap();
        
    assert_eq!(get_response.status(), 200);
    
    let retrieved_user: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(retrieved_user["name"], "New User");
    assert_eq!(retrieved_user["email"], "new@example.com");
    
    // Clean up
    server.shutdown().await;
}
```

### Property-Based Tests

Property-based tests verify invariants across many generated inputs:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_user_validation_properties(
        name in "[a-zA-Z0-9]{1,50}",
        email in "[a-zA-Z0-9_.]+@[a-zA-Z0-9_.]+\\.[a-zA-Z0-9_.]+",
    ) {
        let user = User {
            name: name.clone(),
            email: email.clone(),
        };
        
        let validation_result = user.validate();
        prop_assert!(validation_result.is_ok());
        
        let serialized = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&serialized).unwrap();
        
        prop_assert_eq!(deserialized.name, name);
        prop_assert_eq!(deserialized.email, email);
    }
}
```

### Doc Tests

Doc tests verify that code examples in documentation work correctly:

```rust
/// Validates an email address.
///
/// # Examples
///
/// ```
/// use navius::utils::validate_email;
///
/// assert!(validate_email("user@example.com"));
/// assert!(!validate_email("invalid-email"));
/// ```
pub fn validate_email(email: &str) -> bool {
    // Implementation
}
```

## Test Organization

### Unit Tests

Place unit tests in the same file as the implementation:

```rust
// src/utils/validation.rs
pub fn validate_email(email: &str) -> bool {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_email() {
        // Tests
    }
}
```

### Integration Tests

Place integration tests in the `tests/` directory:

```
tests/
├── api/
│   ├── user_api_test.rs
│   └── health_api_test.rs
├── services/
│   ├── user_service_test.rs
│   └── auth_service_test.rs
└── common/
    ├── mod.rs
    └── test_utils.rs
```

## Test Fixtures

Use fixtures to set up test environments:

```rust
// tests/common/mod.rs
use navius::app::TestApp;
use std::sync::Once;

static INIT: Once = Once::new();

pub async fn setup_test_app() -> TestApp {
    // Initialize logging only once
    INIT.call_once(|| {
        env_logger::init();
    });
    
    // Create a test database
    let db_name = format!("test_db_{}", uuid::Uuid::new_v4());
    let db_url = format!("postgres://localhost/{}", db_name);
    
    // Run migrations
    setup_database(&db_url).await;
    
    // Build the app
    let app = TestApp::new(db_url).await;
    
    app
}

pub async fn teardown_test_app(app: TestApp) {
    // Shut down the app
    app.shutdown().await;
    
    // Clean up the database
    drop_database(&app.db_name()).await;
}

// Example usage in tests
#[tokio::test]
async fn test_create_user() {
    let app = setup_test_app().await;
    
    // Test code
    
    teardown_test_app(app).await;
}
```

## Mocking

Use mocks to isolate components during testing:

```rust
use mockall::predicate::*;
use mockall::mock;

mock! {
    pub DatabaseService {
        fn get_user(&self, id: &str) -> Result<User, DatabaseError>;
        fn create_user(&self, user: User) -> Result<User, DatabaseError>;
    }
}

#[test]
fn test_user_service_with_mock() {
    let mut mock_db = MockDatabaseService::new();
    
    // Setup expectations
    mock_db.expect_get_user()
        .with(eq("123"))
        .times(1)
        .returning(|_| Ok(User {
            id: "123".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }));
    
    // Create service with mock
    let user_service = UserService::new(Arc::new(mock_db));
    
    // Run test
    let user = user_service.get_user("123").unwrap();
    
    // Verify
    assert_eq!(user.id, "123");
    assert_eq!(user.name, "Test User");
}
```

## Test Database

For database tests, use a dedicated test database:

```rust
// tests/database_tests.rs
use sqlx::{PgPool, Postgres, Pool};
use uuid::Uuid;

async fn setup_test_db() -> Pool<Postgres> {
    // Generate unique database name
    let db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace("-", ""));
    
    // Connect to postgres to create test database
    let root_pool = PgPool::connect("postgres://localhost/postgres")
        .await
        .expect("Failed to connect to postgres");
        
    // Create test database
    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&root_pool)
        .await
        .expect("Failed to create test database");
        
    // Connect to test database
    let db_url = format!("postgres://localhost/{}", db_name);
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to test database");
        
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
        
    pool
}

async fn teardown_test_db(pool: Pool<Postgres>, db_name: &str) {
    // Close all connections
    pool.close().await;
    
    // Connect to postgres to drop test database
    let root_pool = PgPool::connect("postgres://localhost/postgres")
        .await
        .expect("Failed to connect to postgres");
        
    // Drop test database
    sqlx::query(&format!("DROP DATABASE {}", db_name))
        .execute(&root_pool)
        .await
        .expect("Failed to drop test database");
}

#[tokio::test]
async fn test_user_repository() {
    let pool = setup_test_db().await;
    let db_name = pool.connect_options().get_database()
        .unwrap_or("unknown")
        .to_string();
        
    // Create repository
    let repo = UserRepository::new(pool.clone());
    
    // Test code
    
    // Clean up
    teardown_test_db(pool, &db_name).await;
}
```

## Test Coverage

For measuring test coverage:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html --output-dir coverage

# Run with specific targets
cargo tarpaulin --packages navius-core navius-web --out Html --output-dir coverage
```

## Continuous Integration

Configure CI to run tests automatically:

```yaml
# .gitlab-ci.yml
test:
  stage: test
  script:
    - cargo test --all-features
    - cargo tarpaulin --out Xml --output-dir coverage
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: coverage/cobertura.xml
```

## Best Practices

1. **Write Tests First**: Follow test-driven development where appropriate
2. **Test Behavior, Not Implementation**: Test what code does, not how it does it
3. **Isolate Tests**: Each test should run independently
4. **Use Descriptive Test Names**: Test names should describe what is being tested
5. **Test Edge Cases**: Test boundary conditions and error paths
6. **Keep Tests Fast**: Optimize test speed for developer productivity
7. **Use Test Helpers**: Extract common test code into helper functions
8. **Avoid Test Interdependence**: Don't rely on test execution order
9. **Mock External Dependencies**: Use mocks for external services
10. **Test at the Appropriate Level**: Use the right test type for each scenario

## Troubleshooting Tests

### Deadlocks in Async Tests

```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_with_potential_deadlock() {
    // Use multi-threaded runtime for deadlock prevention
}
```

### Flaky Tests

```rust
#[test]
#[ignore = "Flaky test, needs investigation"]
fn flaky_test() {
    // Problematic test
}
```

### Testing Timeouts

```rust
#[tokio::test(flavor = "multi_thread")]
#[timeout(1000)]  // 1 second timeout
async fn test_with_timeout() {
    // Long-running test
}
```

## Related Guides

- [Error Handling](error-handling.md) for testing error scenarios
- [Dependency Injection](dependency-injection.md) for mocking dependencies
- [Configuration](configuration.md) for configuring test environments 