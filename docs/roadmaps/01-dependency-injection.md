# Dependency Injection Roadmap

## Overview
Dependency injection is a design pattern that helps make code more modular, maintainable, and testable by separating the creation of objects from their use. This roadmap outlines a lightweight, Rust-idiomatic approach to dependency management that avoids unnecessary complexity while maintaining the benefits of modularity and testability.

## Current State
Currently, our application passes `Arc<AppState>` manually to handlers and services, which is a good foundation but could benefit from a more structured approach to service creation and testing support.

## Target State
A lightweight dependency management approach that:
- Maintains Rust's compile-time safety guarantees
- Makes testing straightforward with minimal boilerplate
- Avoids complex abstractions and runtime overhead
- Provides convenient access to services throughout the application
- Supports clear initialization and configuration patterns

## Implementation Progress Tracking

### Phase 1: Structured App State Management
1. **Core AppState Structure**
   - [ ] Define a clean AppState struct that holds all services
   - [ ] Implement proper initialization and shutdown for stateful services
   - [ ] Create a builder pattern for flexible AppState construction
   
   *Updated at: Not started*

2. **Service Trait Definitions**
   - [ ] Define core traits for major service categories
   - [ ] Implement real service implementations of these traits
   - [ ] Add configuration options for service instantiation
   
   *Updated at: Not started*

3. **Error Handling**
   - [ ] Implement proper error propagation for service initialization failures
   - [ ] Add graceful shutdown mechanisms for all services
   - [ ] Create helpful error messages for debugging service issues
   
   *Updated at: Not started*

### Phase 2: Testing Support
1. **Mock Implementations**
   - [ ] Create mock versions of core services
   - [ ] Implement helper functions for configuring mock behavior
   - [ ] Add verification capabilities for testing service interactions
   
   *Updated at: Not started*

2. **Test Utilities**
   - [ ] Develop test fixture helpers for common testing patterns
   - [ ] Implement test-specific AppState constructor
   - [ ] Create utilities for setting up test environments
   
   *Updated at: Not started*

3. **Integration Test Support**
   - [ ] Add support for replacing specific services in integration tests
   - [ ] Create test-specific service configurations
   - [ ] Implement service spy functionality for verifying behaviors
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 22, 2025
- **Next Milestone**: Service registration system

## Success Criteria
- Services are initialized in a clear, declarative manner
- Handlers can access dependencies with minimal boilerplate
- Testing with mock services is straightforward
- Error handling is robust and informative
- Application startup and shutdown are clean and deterministic

## Implementation Notes
This approach intentionally avoids complex DI patterns common in languages like Java with Spring, focusing instead on Rust idioms that leverage the type system and ownership model. The goal is clarity and simplicity rather than magic or complex abstractions.

### Example Implementation

```rust
// Service traits
trait DatabaseService: Send + Sync {
    async fn get_user(&self, id: UserId) -> Result<User, DbError>;
    // Other database methods...
}

trait AuthService: Send + Sync {
    async fn authenticate(&self, credentials: Credentials) -> Result<AuthToken, AuthError>;
    // Other auth methods...
}

// AppState
struct AppState {
    db: Arc<dyn DatabaseService>,
    auth: Arc<dyn AuthService>,
    cache: Arc<CacheService>,
    config: AppConfig,
}

impl AppState {
    // Builder pattern
    fn builder() -> AppStateBuilder {
        AppStateBuilder::new()
    }
    
    // For tests
    #[cfg(test)]
    fn for_testing() -> Self {
        Self {
            db: Arc::new(MockDatabaseService::new()),
            auth: Arc::new(MockAuthService::new()),
            cache: Arc::new(MockCacheService::new()),
            config: AppConfig::default(),
        }
    }
}

// Then in handlers
async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<UserId>,
) -> impl IntoResponse {
    match state.db.get_user(user_id).await {
        Ok(user) => Json(user).into_response(),
        Err(e) => {
            tracing::error!("Failed to get user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

## References
- [Rust Design Patterns: Dependency Injection](https://rust-unofficial.github.io/patterns/patterns/creational/di.html)
- [Axum State Management](https://docs.rs/axum/latest/axum/extract/struct.State.html)
- [Arc Documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [Builder Pattern in Rust](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html) 