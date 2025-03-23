# Dependency Injection Roadmap

## Overview
A lightweight, Rust-idiomatic approach to dependency injection that leverages the type system to provide compile-time safety and excellent developer experience. Our focus is on creating a flexible yet simple system that maintains Rust's performance characteristics while providing the benefits of dependency injection: modularity, testability, and maintainability.

## Current State
- Basic `Arc<AppState>` pattern established
- Manual dependency passing to handlers
- Initial service trait definitions
- Basic test mocking support
- Prototype service registration
- Initial error handling patterns
- Basic configuration management

## Target State
A comprehensive dependency management system that:
- Leverages Rust's type system for compile-time safety
- Provides ergonomic service access patterns
- Enables straightforward testing with minimal boilerplate
- Supports clear initialization and configuration
- Maintains excellent performance characteristics
- Includes comprehensive documentation
- Supports both development and production environments
- Enables easy service mocking and testing

## Implementation Progress Tracking

### Phase 1: Structured App State Management
1. **Core AppState Structure**
   - [x] Define base service trait interfaces
   - [x] Create generic AppState structure
   - [x] Implement service accessor methods
   - [ ] Add service lifecycle hooks
     - [ ] Startup hooks
     - [ ] Shutdown hooks
     - [ ] Health check hooks
   - [ ] Create AppState builder pattern
     - [ ] Type-safe builder methods
     - [ ] Validation steps
     - [ ] Default configurations
   - [ ] Implement dependency validation
     - [ ] Circular dependency detection
     - [ ] Optional dependency support
     - [ ] Conditional dependencies
   
   *Updated at: March 24, 2025 - Core structure implemented, working on lifecycle management*

2. **Service Registration System**
   - [x] Create ServiceProvider trait
   - [x] Implement basic service registry
   - [ ] Add dependency resolution
     - [ ] Topological sorting
     - [ ] Lazy initialization
     - [ ] Async initialization
   - [ ] Create configuration system
     - [ ] Type-safe configs
     - [ ] Environment overrides
     - [ ] Secrets management
   - [ ] Add initialization ordering
     - [ ] Dependency graph
     - [ ] Parallel initialization
     - [ ] Failure handling
   
   *Updated at: March 24, 2025 - Basic registration working, implementing advanced features*

3. **Error Handling**
   - [x] Define error types
   - [x] Add error context
   - [ ] Implement dependency validation
     - [ ] Missing dependency checks
     - [ ] Version compatibility
     - [ ] Resource availability
   - [ ] Create recovery mechanisms
     - [ ] Retry policies
     - [ ] Circuit breakers
     - [ ] Fallback services
   - [ ] Add shutdown coordination
     - [ ] Graceful shutdown
     - [ ] Resource cleanup
     - [ ] State persistence
   
   *Updated at: March 24, 2025 - Core error handling in place*

### Phase 2: Testing Support
1. **Mock Service Framework**
   - [x] Create MockService trait
   - [x] Implement basic mocking
   - [ ] Add expectation system
     - [ ] Call counting
     - [ ] Argument matching
     - [ ] Return value sequences
   - [ ] Create spy functionality
     - [ ] Call recording
     - [ ] Argument capture
     - [ ] Timing tracking
   - [ ] Add scenario support
     - [ ] State machines
     - [ ] Conditional responses
     - [ ] Error injection
   
   *Updated at: March 24, 2025 - Basic mocking implemented*

2. **Test Utilities**
   - [x] Create TestContext
   - [x] Add basic test helpers
   - [ ] Implement service substitution
     - [ ] Hot swapping
     - [ ] State preservation
     - [ ] Isolation guarantees
   - [ ] Create test data factories
     - [ ] Randomized data
     - [ ] Realistic scenarios
     - [ ] Custom generators
   
   *Updated at: March 24, 2025 - Core utilities available*

3. **Integration Test Support**
   - [ ] Create test configurations
     - [ ] Environment isolation
     - [ ] Resource limits
     - [ ] Logging controls
   - [ ] Add transaction support
     - [ ] Automatic rollback
     - [ ] Savepoints
     - [ ] Cleanup hooks
   - [ ] Implement test containers
     - [ ] Database containers
     - [ ] Cache containers
     - [ ] Service mocks
   
   *Updated at: Not started*

### Phase 3: Advanced Features
1. **Service Lifecycle Management**
   - [ ] Implement startup ordering
     - [ ] Dependency-based ordering
     - [ ] Parallel startup
     - [ ] Timeout handling
   - [ ] Add health monitoring
     - [ ] Health checks
     - [ ] Dependency status
     - [ ] Resource usage
   - [ ] Create recovery system
     - [ ] Automatic restart
     - [ ] Failover support
     - [ ] State recovery
   
   *Updated at: Not started*

2. **Configuration Management**
   - [ ] Create config validation
     - [ ] Schema validation
     - [ ] Type checking
     - [ ] Default values
   - [ ] Add dynamic config
     - [ ] Hot reload
     - [ ] Feature flags
     - [ ] A/B testing
   - [ ] Implement secrets
     - [ ] Encryption
     - [ ] Rotation
     - [ ] Access control
   
   *Updated at: Not started*

3. **Performance Optimization**
   - [ ] Add lazy loading
     - [ ] On-demand initialization
     - [ ] Resource pooling
     - [ ] Cache warming
   - [ ] Implement metrics
     - [ ] Usage tracking
     - [ ] Performance monitoring
     - [ ] Resource utilization
   - [ ] Create scaling support
     - [ ] Load balancing
     - [ ] Sharding
     - [ ] Replication
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 25% complete
- **Last Updated**: March 24, 2025
- **Next Milestone**: Complete Service Lifecycle Management
- **Current Focus**: AppState builder pattern and service initialization

## Success Criteria
- Compile-time dependency validation
- Zero-cost abstractions where possible
- Clear and informative error messages
- Comprehensive test coverage (95%+)
- Minimal runtime overhead
- Excellent developer experience
- Thorough documentation with examples
- Seamless integration with Axum handlers

## Implementation Notes

### Core Service Pattern
```rust
use async_trait::async_trait;
use std::sync::Arc;

// Base service marker trait
pub trait Service: Send + Sync + 'static {}

// Service provider trait for initialization
#[async_trait]
pub trait ServiceProvider: Sized {
    type Service: Service;
    type Config: Clone + Send + Sync;
    type Error: std::error::Error + Send + Sync;
    
    async fn create(
        config: Self::Config,
        registry: &ServiceRegistry,
    ) -> Result<Self::Service, Self::Error>;
    
    async fn health_check(&self) -> Result<(), Self::Error>;
}

// Example database service
#[async_trait]
pub trait DatabaseService: Service {
    async fn transaction<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&Transaction) -> Result<R, DbError> + Send + 'static;
        
    async fn health(&self) -> Result<DbHealth, DbError>;
}

// Implementation with connection pooling
pub struct PostgresDatabase {
    pool: Pool<Postgres>,
    metrics: Arc<DbMetrics>,
    config: DbConfig,
}

#[async_trait]
impl ServiceProvider for PostgresDatabase {
    type Service = Self;
    type Config = DbConfig;
    type Error = DbError;
    
    async fn create(config: Self::Config, _: &ServiceRegistry) -> Result<Self, DbError> {
        let pool = Pool::connect(&config.url).await?;
        let metrics = Arc::new(DbMetrics::new());
        
        Ok(Self { pool, metrics, config })
    }
    
    async fn health_check(&self) -> Result<(), DbError> {
        self.pool.acquire().await?;
        Ok(())
    }
}

// Type-safe AppState with proper constraints
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
    metrics: Arc<Metrics>,
}

// Builder pattern with validation
impl<DB, Cache, Auth> AppState<DB, Cache, Auth>
where
    DB: DatabaseService,
    Cache: CacheService,
    Auth: AuthService,
{
    pub fn builder() -> AppStateBuilder<DB, Cache, Auth> {
        AppStateBuilder::new()
    }
    
    pub async fn initialize(config: AppConfig) -> Result<Self, InitError> {
        let registry = ServiceRegistry::new();
        
        // Parallel initialization where possible
        let (db, cache, auth) = tokio::try_join!(
            DB::create(config.db, &registry),
            Cache::create(config.cache, &registry),
            Auth::create(config.auth, &registry)
        )?;
        
        Ok(Self {
            db: Arc::new(db),
            cache: Arc::new(cache),
            auth: Arc::new(auth),
            config: Arc::new(config),
            metrics: Arc::new(Metrics::new()),
        })
    }
}
```

### Testing Support
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use navius_test::*;

    // Enhanced test context with automatic cleanup
    #[derive(TestContext)]
    struct TestContext {
        app: AppState<MockDb, MockCache, MockAuth>,
        #[cleanup]
        temp_data: TempData,
    }
    
    impl TestContext {
        async fn setup() -> Self {
            let app = AppState::builder()
                .with_database(MockDb::new().with_users([TEST_USER]))
                .with_cache(MockCache::new())
                .with_auth(MockAuth::new().expect_valid_token(TEST_TOKEN))
                .build()
                .await
                .expect("Failed to build test context");
                
            Self {
                app,
                temp_data: TempData::new(),
            }
        }
    }
    
    #[tokio::test]
    async fn test_user_creation() -> TestResult {
        // Arrange
        let ctx = TestContext::setup().await;
        let new_user = fake::user();
        
        // Act
        let result = ctx.app.db()
            .create_user(new_user.clone())
            .await;
            
        // Assert
        assert_ok!(result);
        let user = result.unwrap();
        assert_eq!(user.email, new_user.email);
        
        // Verify cache was updated
        let cached = ctx.app.cache()
            .get::<User>(&user.id.to_string())
            .await?;
        assert_some!(cached);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_service_failure() -> TestResult {
        // Arrange
        let ctx = TestContext::setup().await;
        ctx.app.db().simulate_error(DbError::ConnectionLost);
        
        // Act
        let result = ctx.app.db()
            .create_user(fake::user())
            .await;
            
        // Assert
        assert_err!(result);
        assert_matches!(result.unwrap_err(), DbError::ConnectionLost);
        
        Ok(())
    }
}
```

## References
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [async-trait Documentation](https://docs.rs/async-trait/latest/async_trait/)
- [tokio Documentation](https://docs.rs/tokio/latest/tokio/)
- [Arc Documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [Testing in Rust](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [mockall Crate](https://docs.rs/mockall/latest/mockall/)
- [bb8 Connection Pool](https://docs.rs/bb8/latest/bb8/) 