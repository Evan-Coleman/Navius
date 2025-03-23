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
   - [ ] Define service trait interfaces for all major components
   - [ ] Create AppState struct with proper type parameters
   - [ ] Implement service accessor methods
   - [ ] Add service lifecycle management
   - [ ] Create builder pattern for AppState construction
   - [ ] Add configuration validation
   
   *Updated at: Not started*

2. **Service Registration System**
   - [ ] Create service registry trait
   - [ ] Implement service provider pattern
   - [ ] Add service dependency resolution
   - [ ] Create service configuration system
   - [ ] Add service initialization ordering
   - [ ] Implement service validation
   
   *Updated at: Not started*

3. **Error Handling**
   - [ ] Define service initialization error types
   - [ ] Implement error context for debugging
   - [ ] Add service dependency validation
   - [ ] Create helpful error messages
   - [ ] Add error recovery mechanisms
   - [ ] Implement graceful shutdown
   
   *Updated at: Not started*

### Phase 2: Testing Support
1. **Mock Service Framework**
   - [ ] Create mock service trait
   - [ ] Implement mock service provider
   - [ ] Add mock configuration system
   - [ ] Create mock service factory
   - [ ] Add verification capabilities
   - [ ] Implement spy functionality
   
   *Updated at: Not started*

2. **Test Utilities**
   - [ ] Create test AppState builder
   - [ ] Add test configuration helpers
   - [ ] Implement service replacement utilities
   - [ ] Create test data factories
   - [ ] Add assertion helpers
   - [ ] Implement cleanup utilities
   
   *Updated at: Not started*

3. **Integration Test Support**
   - [ ] Create test-specific service configurations
   - [ ] Add service isolation utilities
   - [ ] Implement test transaction support
   - [ ] Create integration test helpers
   - [ ] Add performance test support
   - [ ] Implement chaos testing utilities
   
   *Updated at: Not started*

### Phase 3: Advanced Features
1. **Service Lifecycle Management**
   - [ ] Implement proper startup ordering
   - [ ] Add graceful shutdown support
   - [ ] Create health check system
   - [ ] Add dependency validation
   - [ ] Implement recovery mechanisms
   - [ ] Create monitoring hooks
   
   *Updated at: Not started*

2. **Configuration Management**
   - [ ] Create type-safe configuration
   - [ ] Add environment overrides
   - [ ] Implement secrets handling
   - [ ] Add configuration validation
   - [ ] Create hot reload support
   - [ ] Implement audit logging
   
   *Updated at: Not started*

3. **Performance Optimization**
   - [ ] Implement lazy initialization
   - [ ] Add connection pooling
   - [ ] Create resource limits
   - [ ] Implement caching layer
   - [ ] Add performance monitoring
   - [ ] Create scaling hooks
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 22, 2025
- **Next Milestone**: Core AppState Structure

## Success Criteria
- Services are initialized in a clear, declarative manner
- Handlers can access dependencies with minimal boilerplate
- Testing with mock services is straightforward
- Error handling is robust and informative
- Application startup and shutdown are clean and deterministic
- Configuration is type-safe and validated
- Performance overhead is minimal

## Implementation Notes

### Core Service Pattern
```rust
// Service traits with clear contracts
pub trait DatabaseService: Send + Sync {
    async fn get_user(&self, id: UserId) -> Result<User, DbError>;
    async fn create_user(&self, user: NewUser) -> Result<User, DbError>;
    async fn transaction<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&Transaction) -> Result<R, DbError> + Send;
}

pub trait CacheService: Send + Sync {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError>;
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
}

pub trait AuthService: Send + Sync {
    async fn authenticate(&self, credentials: Credentials) -> Result<AuthToken, AuthError>;
    async fn validate_token(&self, token: &str) -> Result<Claims, AuthError>;
    async fn refresh_token(&self, token: &str) -> Result<AuthToken, AuthError>;
}

// AppState with proper type parameters
pub struct AppState<DB, Cache, Auth>
where
    DB: DatabaseService,
    Cache: CacheService,
    Auth: AuthService,
{
    db: Arc<DB>,
    cache: Arc<Cache>,
    auth: Arc<Auth>,
    config: Arc<AppConfig>,
}

// Builder pattern for clean initialization
impl<DB, Cache, Auth> AppState<DB, Cache, Auth>
where
    DB: DatabaseService,
    Cache: CacheService,
    Auth: AuthService,
{
    pub fn builder() -> AppStateBuilder<DB, Cache, Auth> {
        AppStateBuilder::new()
    }
    
    pub fn db(&self) -> &Arc<DB> {
        &self.db
    }
    
    pub fn cache(&self) -> &Arc<Cache> {
        &self.cache
    }
    
    pub fn auth(&self) -> &Arc<Auth> {
        &self.auth
    }
    
    pub fn config(&self) -> &Arc<AppConfig> {
        &self.config
    }
}

// Service provider for registration
pub trait ServiceProvider {
    type Service;
    type Config;
    type Error;
    
    async fn create_service(
        config: Self::Config,
        registry: &ServiceRegistry,
    ) -> Result<Self::Service, Self::Error>;
}

// Mock implementations for testing
#[cfg(test)]
pub struct MockDatabaseService {
    users: Arc<Mutex<HashMap<UserId, User>>>,
    error_mode: Arc<AtomicBool>,
}

#[cfg(test)]
impl DatabaseService for MockDatabaseService {
    async fn get_user(&self, id: UserId) -> Result<User, DbError> {
        if self.error_mode.load(Ordering::SeqCst) {
            return Err(DbError::ConnectionError);
        }
        
        let users = self.users.lock().await;
        users.get(&id)
            .cloned()
            .ok_or(DbError::NotFound)
    }
    
    // Other implementations...
}

// Example usage in handlers
pub async fn create_user<DB: DatabaseService>(
    State(state): State<Arc<AppState<DB, impl CacheService, impl AuthService>>>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, AppError> {
    let user = state.db().create_user(new_user).await?;
    Ok(Json(user))
}
```

### Testing Support
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestContext {
        app: AppState<MockDatabaseService, MockCacheService, MockAuthService>,
    }
    
    impl TestContext {
        async fn setup() -> Self {
            let app = AppState::builder()
                .with_database(MockDatabaseService::new())
                .with_cache(MockCacheService::new())
                .with_auth(MockAuthService::new())
                .with_config(TestConfig::default())
                .build()
                .await
                .expect("Failed to build test context");
                
            Self { app }
        }
        
        async fn create_test_user(&self) -> User {
            let new_user = NewUser {
                name: "Test User".to_string(),
                email: "test@example.com".to_string(),
            };
            
            self.app.db().create_user(new_user).await
                .expect("Failed to create test user")
        }
    }
    
    #[tokio::test]
    async fn test_user_creation() {
        let ctx = TestContext::setup().await;
        let new_user = NewUser {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let result = create_user(
            State(Arc::new(ctx.app)),
            Json(new_user),
        ).await;
        
        assert!(result.is_ok());
    }
}
```

## References
- [Rust Design Patterns: Dependency Injection](https://rust-unofficial.github.io/patterns/patterns/creational/di.html)
- [Axum State Management](https://docs.rs/axum/latest/axum/extract/struct.State.html)
- [Arc Documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [Builder Pattern in Rust](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
- [Testing in Rust](https://doc.rust-lang.org/book/ch11-00-testing.html) 