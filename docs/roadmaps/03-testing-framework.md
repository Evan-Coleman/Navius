# Testing Framework Roadmap

## Overview
A focused, security-oriented testing framework for Navius that prioritizes essential test coverage with minimal dependencies. Rather than recreating Spring Boot's extensive testing ecosystem, we'll implement a pragmatic set of testing utilities that leverage Rust's built-in testing capabilities while providing the necessary tools for thorough API testing.

## Current State
The application needs a structured testing approach that ensures security, correctness, and reliability without excessive complexity.

## Target State
A lightweight but comprehensive testing framework that:
- Prioritizes security-critical component testing
- Makes writing tests straightforward for developers
- Leverages Rust's built-in testing capabilities
- Provides specialized utilities for Axum handlers and routes
- Enables reliable integration testing with minimal setup

## Implementation Progress Tracking

### Phase 1: Security and Unit Testing Essentials
1. **Security Testing Infrastructure**
   - [ ] Create authentication/authorization test helpers
   - [ ] Implement security assertion utilities
   - [ ] Add request validation test tools
   
   *Updated at: Not started*

2. **Mocking Fundamentals**
   - [ ] Implement lightweight trait-based mocks for critical services
   - [ ] Create configurable mock responses for security components
   - [ ] Add simple verification capabilities for test assertions
   
   *Updated at: Not started*

3. **Handler Unit Testing**
   - [ ] Build Axum-specific handler test utilities
   - [ ] Create reusable test fixtures for common scenarios
   - [ ] Implement response validation helpers
   
   *Updated at: Not started*

### Phase 2: Integration Testing Essentials
1. **API Testing Framework**
   - [ ] Create test application builder with minimal configuration
   - [ ] Implement type-safe route testing
   - [ ] Add JSON response validation utilities
   
   *Updated at: Not started*

2. **Database Testing**
   - [ ] Implement transaction-based test isolation
   - [ ] Create simple test data factories
   - [ ] Add database state assertions
   
   *Updated at: Not started*

3. **Error Scenario Testing**
   - [ ] Build tools for testing error responses
   - [ ] Implement failure injection for resilience testing
   - [ ] Add security failure scenario testing
   
   *Updated at: Not started*

### Phase 3: CI and Developer Experience
1. **CI Integration**
   - [ ] Create optimized test suite organization
   - [ ] Implement security-focused test tagging
   - [ ] Add code coverage reporting
   
   *Updated at: Not started*

2. **Developer Utilities**
   - [ ] Build test helpers for common scenarios
   - [ ] Create concise test builders with sensible defaults
   - [ ] Implement debug utilities for test failures
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 20, 2024
- **Next Milestone**: Security Testing Infrastructure

## Success Criteria
- Security-critical paths have thorough test coverage
- Writing new tests requires minimal boilerplate
- Tests run quickly enough for developer feedback loops
- Integration tests reliably test API behavior
- Database tests don't require complex setup

## Implementation Notes
This approach focuses on practical testing capabilities that provide the most value for ensuring security and correctness. We'll leverage Rust's built-in testing capabilities and Axum's design to create a lightweight yet effective testing framework.

### Example Implementation

```rust
// Example of a handler unit test
#[tokio::test]
async fn test_create_user_handler_validates_input() {
    // Arrange
    let mock_db = MockDbService::new();
    mock_db.expect_create_user().times(0); // Expect no calls since validation should fail
    
    let app = test_app()
        .with_service(mock_db)
        .build();
    
    // Act
    let response = app
        .call(
            Request::builder()
                .method("POST")
                .uri("/api/users")
                .header("Content-Type", "application/json")
                .header("Authorization", "Bearer test-token")
                .body(json!({"name": "", "email": "not-an-email"}).to_string())
                .unwrap()
        )
        .await;
    
    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let error: ValidationErrorResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(error.fields.contains_key("name"));
    assert!(error.fields.contains_key("email"));
}

// Example of an integration test with security focus
#[tokio::test]
async fn test_protected_endpoint_requires_authentication() {
    // Arrange
    let app = test_app().build();
    
    // Act - Call without auth token
    let response = app
        .call(
            Request::builder()
                .method("GET")
                .uri("/api/protected-resource")
                .body(Body::empty())
                .unwrap()
        )
        .await;
    
    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// Example of a database integration test
#[tokio::test]
async fn test_user_creation_persists_to_database() {
    // Arrange
    let db_pool = test_db_pool().await;
    
    let app = test_app()
        .with_db_pool(db_pool.clone())
        .build();
    
    // Use transaction to ensure test isolation
    let test_tx = db_pool.begin().await.unwrap();
    
    // Act
    let response = app
        .call(
            Request::builder()
                .method("POST")
                .uri("/api/users")
                .header("Content-Type", "application/json")
                .header("Authorization", "Bearer valid-test-token")
                .body(json!({"name": "Test User", "email": "test@example.com"}).to_string())
                .unwrap()
        )
        .await;
    
    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Verify database state
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind("test@example.com")
        .fetch_one(&test_tx)
        .await
        .unwrap();
    
    assert_eq!(user.name, "Test User");
    
    // Rollback transaction to clean up
    test_tx.rollback().await.unwrap();
}
```

## References
- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Axum Testing](https://docs.rs/axum/latest/axum/middleware/index.html)
- [SQLx Testing](https://github.com/launchbadge/sqlx/blob/main/FAQ.md#how-do-i-mock-sqlx-in-my-tests)
- [mockall](https://docs.rs/mockall/latest/mockall/) 