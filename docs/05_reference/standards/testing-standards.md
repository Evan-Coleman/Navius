---
title: ""
description: "Reference documentation for Navius "
category: "Reference"
tags: ["documentation", "reference"]
last_updated: "April 3, 2025"
version: "1.0"
---


# Testing Standards

This document outlines the testing standards and patterns used throughout the Navius framework, providing a reference for consistent testing implementation.

## Testing Philosophy

Navius follows these core testing principles:

1. **Test-Driven Development**: Tests should guide implementation
2. **Comprehensive Coverage**: Target 80%+ code coverage for core functionality
3. **Isolation**: Tests should be independent and not affect each other
4. **Fast Feedback**: Tests should run quickly to support development
5. **Representative**: Tests should reflect real-world usage
6. **Maintainable**: Tests should be easy to understand and maintain

## Test Types

### Unit Tests

Unit tests verify individual components in isolation:

```rust
// In src/utils/validation.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_email() {
        // Valid emails
        assert!(validate_email("user@example.com"));
        assert!(validate_email("user+tag@example.co.uk"));
        
        // Invalid emails
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("@missing-user.com"));
        assert!(!validate_email(""));
    }
}
```

### Integration Tests

Integration tests verify interactions between components:

```rust
// In tests/api/user_tests.rs
#[tokio::test]
async fn test_get_user_endpoint() {
    // Setup
    let app = test_app().await;
    let user_id = app.create_test_user().await;
    
    // Execute
    let response = app
        .get(&format!("/users/{}", user_id))
        .send()
        .await;
        
    // Verify
    assert_eq!(response.status(), 200);
    let user = response.json::<User>().await;
    assert_eq!(user.id, user_id);
}
```

### End-to-End Tests

End-to-end tests verify the entire application stack:

```rust
// In tests/e2e/user_flow_test.rs
#[tokio::test]
async fn test_user_registration_flow() {
    // Start server with test database
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    
    // Register user
    let register_response = client
        .post(&format!("{}/register", server.address()))
        .json(&RegisterRequest {
            username: "testuser",
            email: "test@example.com",
            password: "password123",
        })
        .send()
        .await
        .unwrap();
        
    assert_eq!(register_response.status(), 201);
    
    // Login
    let login_response = client
        .post(&format!("{}/login", server.address()))
        .json(&LoginRequest {
            email: "test@example.com",
            password: "password123",
        })
        .send()
        .await
        .unwrap();
        
    assert_eq!(login_response.status(), 200);
    let token = login_response.json::<TokenResponse>().await.unwrap().token;
    
    // Use token to access protected resource
    let protected_response = client
        .get(&format!("{}/profile", server.address()))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
        
    assert_eq!(protected_response.status(), 200);
}
```

### Property-Based Tests

Property-based tests verify invariants across many inputs:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_id_generation_properties(
        name in "[a-zA-Z0-9]{1,10}",
        timestamp in 0..100000u64,
    ) {
        let id1 = generate_id(&name, timestamp);
        let id2 = generate_id(&name, timestamp);
        
        // IDs generated with same inputs should be identical
        prop_assert_eq!(id1, id2);
        
        // IDs should have expected length
        prop_assert_eq!(id1.len(), 36);
        
        // IDs with different timestamps should be different
        if timestamp > 0 {
            let id3 = generate_id(&name, timestamp - 1);
            prop_assert_ne!(id1, id3);
        }
    }
}
```

### Doc Tests

Doc tests verify code examples in documentation:

```rust
/// Validates an email address against RFC 5322 standard.
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

### Directory Structure

```
project/
├── src/
│   └── module.rs       # Unit tests live alongside implementation
└── tests/
    ├── common/
    │   ├── mod.rs      # Common test utilities
    │   └── fixtures.rs # Test fixtures
    ├── api/
    │   ├── user_tests.rs    # API integration tests
    │   └── health_tests.rs
    ├── repository/
    │   └── user_repository_tests.rs
    └── e2e/
        └── user_flow_tests.rs  # End-to-end tests
```

### Naming Conventions

- Test files: `*_tests.rs` or `*_test.rs`
- Test functions: `test_<functionality_being_tested>`
- Test utilities: `setup_*`, `teardown_*`, `create_test_*`
- Test modules: `mod tests { ... }` within source files

## Test Fixtures

### Test Application

```rust
// In tests/common/mod.rs
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub api_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        // Create unique test database
        let db_name = format!("test_db_{}", Uuid::new_v4().simple());
        let db_pool = setup_test_database(&db_name).await;
        
        // Configure app with test database
        let app = application::build(db_pool.clone()).await;
        let server = axum::Server::from_tcp(listener)
            .serve(app.into_make_service());
        
        // Start server on random port
        let address = format!("http://127.0.0.1:{}", port);
        let _ = tokio::spawn(server);
        
        Self {
            address,
            db_pool,
            api_client: reqwest::Client::new(),
        }
    }
    
    pub async fn create_test_user(&self) -> String {
        // Create test user and return ID
    }
    
    pub async fn get<U: AsRef<str>>(&self, uri: U) -> reqwest::RequestBuilder {
        self.api_client.get(format!("{}{}", self.address, uri.as_ref()))
    }
    
    pub async fn post<U: AsRef<str>>(&self, uri: U) -> reqwest::RequestBuilder {
        self.api_client.post(format!("{}{}", self.address, uri.as_ref()))
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        // Clean up any resources
    }
}
```

### Test Database

```rust
// In tests/common/database.rs
pub async fn setup_test_database(db_name: &str) -> PgPool {
    // Connect to postgres to create test database
    let root_pool = PgPool::connect("postgres://postgres:password@localhost:5432/postgres")
        .await
        .expect("Failed to connect to postgres");
        
    // Create test database
    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&root_pool)
        .await
        .expect("Failed to create test database");
        
    // Connect to test database
    let db_url = format!("postgres://postgres:password@localhost:5432/{}", db_name);
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

pub async fn teardown_test_database(db_name: &str) {
    // Connect to postgres to drop test database
    let root_pool = PgPool::connect("postgres://postgres:password@localhost:5432/postgres")
        .await
        .expect("Failed to connect to postgres");
        
    // Terminate existing connections
    sqlx::query(&format!(
        "SELECT pg_terminate_backend(pg_stat_activity.pid) 
         FROM pg_stat_activity 
         WHERE pg_stat_activity.datname = '{}'",
        db_name
    ))
    .execute(&root_pool)
    .await
    .expect("Failed to terminate connections");
    
    // Drop test database
    sqlx::query(&format!("DROP DATABASE {}", db_name))
        .execute(&root_pool)
        .await
        .expect("Failed to drop test database");
}
```

## Mocking

### Mock Implementation

```rust
use mockall::predicate::*;
use mockall::mock;

mock! {
    pub UserRepository {
        fn get_user(&self, id: &str) -> Result<User, RepositoryError>;
        fn create_user(&self, user: User) -> Result<User, RepositoryError>;
        fn update_user(&self, user: User) -> Result<User, RepositoryError>;
        fn delete_user(&self, id: &str) -> Result<(), RepositoryError>;
    }
}

#[test]
fn test_user_service_get_user() {
    // Setup
    let mut mock_repo = MockUserRepository::new();
    mock_repo.expect_get_user()
        .with(eq("user-123"))
        .times(1)
        .returning(|_| Ok(User {
            id: "user-123".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }));
        
    let service = UserService::new(Arc::new(mock_repo));
    
    // Execute
    let result = service.get_user("user-123").unwrap();
    
    // Verify
    assert_eq!(result.id, "user-123");
    assert_eq!(result.name, "Test User");
}
```

### Mock HTTP Server

```rust
use wiremock::{Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_external_service_integration() {
    // Start mock server
    let mock_server = wiremock::MockServer::start().await;
    
    // Setup response
    Mock::given(method("GET"))
        .and(path("/api/external/data"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "data": "test-data"
            })))
        .mount(&mock_server)
        .await;
        
    // Create client with mock URL
    let client = ExternalServiceClient::new(&mock_server.uri());
    
    // Execute
    let result = client.get_data().await.unwrap();
    
    // Verify
    assert_eq!(result.data, "test-data");
}
```

## Test Coverage

### Measuring Coverage

```bash
# Run coverage analysis
cargo tarpaulin --out Html --output-dir coverage

# Run with specific options
cargo tarpaulin --skip-clean --packages navius --exclude-files "tests/*" --out Html
```

### Coverage Requirements

- Core business logic: 90%+ coverage
- API handlers: 80%+ coverage
- Utility functions: 70%+ coverage
- Infrastructure code: 60%+ coverage

## Test Assertions

### Standard Assertions

```rust
// Basic assertions
assert!(condition);
assert_eq!(actual, expected);
assert_ne!(actual, unexpected);

// With custom messages
assert!(condition, "Custom error message");
assert_eq!(actual, expected, "Values should be equal");

// Approximate equality for floating point
assert!((actual - expected).abs() < 0.001);

// Collection assertions
assert_eq!(vec.len(), 3);
assert!(vec.contains(&item));
```

### Error Assertions

```rust
// Check error type
let err = result.unwrap_err();
assert!(matches!(err, AppError::NotFound(_)));

// More specific error checking
if let AppError::NotFound(msg) = err {
    assert!(msg.contains("User"));
} else {
    panic!("Expected NotFound error, got {:?}", err);
}
```

## Async Testing

### Tokio Runtime

```rust
#[tokio::test]
async fn test_async_function() {
    // Test async code
}

// With runtime configuration
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_operations() {
    // Test code that benefits from multiple threads
}
```

### Timeouts

```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_with_timeout() {
    let result = timeout(
        Duration::from_millis(100),
        async_operation()
    ).await;
    
    assert!(result.is_ok());
}
```

## Test Utilities

### Helper Functions

```rust
// In tests/common/utils.rs
pub fn generate_test_user() -> User {
    User {
        id: format!("test-{}", Uuid::new_v4()),
        name: "Test User".to_string(),
        email: format!("test-{}@example.com", Uuid::new_v4().simple()),
    }
}

pub async fn create_test_data(pool: &PgPool) -> Vec<String> {
    // Create multiple test entities and return their IDs
}
```

### Custom Assertions

```rust
// In tests/common/assertions.rs
pub fn assert_user_equals(actual: &User, expected: &User) {
    assert_eq!(actual.id, expected.id);
    assert_eq!(actual.name, expected.name);
    assert_eq!(actual.email, expected.email);
    // Other fields...
}

pub fn assert_error_response(response: &ErrorResponse, expected_status: u16, expected_message: &str) {
    assert_eq!(response.status, expected_status);
    assert!(response.message.contains(expected_message));
}
```

## Test Configuration

### Test Config File

```yaml
# tests/config/test.yaml
database:
  host: "localhost"
  port: 5432
  user: "test_user"
  password: "test_password"
  
test:
  isolation: true
  parallelism: 4
  log_level: "debug"
```

### Loading Test Config

```rust
// In tests/common/config.rs
pub fn load_test_config() -> TestConfig {
    let config_path = std::path::Path::new("tests/config/test.yaml");
    let config_str = std::fs::read_to_string(config_path)
        .expect("Failed to read test config");
        
    serde_yaml::from_str(&config_str)
        .expect("Failed to parse test config")
}
```

## CI Integration

### Test Workflow

```yaml
# .gitlab-ci.yml
test:
  stage: test
  image: rust:latest
  script:
    - cargo test --all-features
    - cargo tarpaulin --out Xml --output-dir coverage
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: coverage/cobertura.xml
```

## Test Best Practices

1. **Write Tests First**: Follow test-driven development
2. **One Assertion Per Test**: Focus each test on a single behavior
3. **Descriptive Test Names**: Test names should describe what is being tested
4. **Clean Test Setup**: Use fixtures and helpers to keep tests clean
5. **Independence**: Tests should not depend on each other
6. **Realistic Data**: Use realistic test data to catch edge cases
7. **Test Failure Scenarios**: Test error paths, not just happy paths
8. **Avoid Test Logic**: Tests should be simple; avoid complex logic in tests
9. **Fast Tests**: Keep tests fast to maintain development velocity
10. **Regular Maintenance**: Update tests when requirements change

## Specific Testing Patterns

### Repository Testing

```rust
#[tokio::test]
async fn test_user_repository_crud() {
    // Setup
    let pool = setup_test_database("test_user_repository").await;
    let repo = UserRepository::new(pool.clone());
    
    // Create
    let user = User {
        id: "".to_string(), // Will be generated
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
    };
    
    let created = repo.create_user(user).await.unwrap();
    assert!(!created.id.is_empty());
    
    // Read
    let retrieved = repo.get_user(&created.id).await.unwrap();
    assert_eq!(retrieved.name, "Test User");
    
    // Update
    let mut updated = retrieved.clone();
    updated.name = "Updated Name".to_string();
    let updated = repo.update_user(updated).await.unwrap();
    assert_eq!(updated.name, "Updated Name");
    
    // Delete
    repo.delete_user(&created.id).await.unwrap();
    let result = repo.get_user(&created.id).await;
    assert!(matches!(result, Err(RepositoryError::NotFound(_))));
    
    // Cleanup
    teardown_test_database("test_user_repository").await;
}
```

### Service Testing

```rust
#[test]
fn test_user_service_business_logic() {
    // Setup mocks
    let mut mock_repo = MockUserRepository::new();
    let mut mock_auth = MockAuthService::new();
    
    // Mock expectations
    mock_repo.expect_get_user()
        .with(eq("admin-user"))
        .returning(|_| Ok(User {
            id: "admin-user".to_string(),
            name: "Admin".to_string(),
            role: "admin".to_string(),
        }));
        
    mock_auth.expect_check_permission()
        .with(eq("admin-user"), eq("delete_user"))
        .returning(|_, _| Ok(true));
        
    // Create service with mocks
    let service = UserService::new(
        Arc::new(mock_repo), 
        Arc::new(mock_auth)
    );
    
    // Test business logic
    let result = service.can_delete_user("admin-user", "other-user");
    assert!(result.unwrap());
}
```

### API Testing

```rust
#[tokio::test]
async fn test_user_api_validation() {
    // Setup
    let app = test_app().await;
    
    // Test invalid input
    let response = app
        .post("/users")
        .json(&json!({
            "name": "x", // Too short
            "email": "invalid-email", // Invalid format
        }))
        .send()
        .await;
        
    // Verify error response
    assert_eq!(response.status(), 400);
    let error = response.json::<ErrorResponse>().await.unwrap();
    
    // Validate error details
    assert!(error.message.contains("validation"));
    assert!(error.details.contains_key("name"));
    assert!(error.details.contains_key("email"));
}
``` 
