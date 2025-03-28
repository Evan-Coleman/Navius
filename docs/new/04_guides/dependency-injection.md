# Dependency Injection

This guide explains how dependency injection works in Navius, covering the principles, patterns, and best practices for managing service dependencies.

## What is Dependency Injection?

Dependency Injection (DI) is a design pattern where components receive their dependencies from an external source rather than creating them internally. In Navius, this means:

1. Services declare their dependencies explicitly in constructors
2. Dependencies are created and managed by a central registry
3. Service lifetimes are controlled by the application container
4. Components are decoupled for easier testing and maintenance

## Benefits of Dependency Injection

- **Decoupling**: Services depend on abstractions, not concrete implementations
- **Testability**: Dependencies can be easily mocked for testing
- **Flexibility**: Implementation details can change without affecting consumers
- **Maintainability**: Dependencies are explicit and visible
- **Reusability**: Services can be reused with different dependencies

## Dependency Injection in Navius

Navius implements dependency injection through several mechanisms:

1. **Constructor Injection**: Services receive dependencies in their constructors
2. **Service Registry**: Central container manages service instances
3. **Axum Extensions**: Web handlers receive dependencies via the framework

## Constructor Injection

The most common form of DI in Navius is constructor injection:

```rust
use std::sync::Arc;

struct UserService {
    database: Arc<dyn DatabaseProvider>,
    logger: Arc<LoggerService>,
    config: Arc<ConfigService>,
}

impl UserService {
    pub fn new(
        database: Arc<dyn DatabaseProvider>,
        logger: Arc<LoggerService>,
        config: Arc<ConfigService>
    ) -> Self {
        Self {
            database,
            logger,
            config,
        }
    }
    
    pub fn get_user(&self, id: &str) -> Result<User, Error> {
        self.logger.log(&format!("Getting user {}", id));
        
        let timeout = self.config.get_int("database.timeout").unwrap_or(30);
        self.database.get_user_with_timeout(id, timeout)
    }
}
```

Key points:
- Dependencies are explicitly declared in the constructor
- Services typically take `Arc<T>` to share ownership
- Interfaces (traits) can be used for abstraction

## Service Registry Pattern

For registration and resolution of services, Navius uses a service registry:

```rust
use navius::core::service::ServiceRegistry;
use std::sync::Arc;

fn main() {
    // Create service registry
    let mut registry = ServiceRegistry::new();
    
    // Register services with dependencies
    let config = Arc::new(ConfigService::new());
    registry.register::<ConfigService>(config.clone());
    
    let logger = Arc::new(LoggerService::new(config.clone()));
    registry.register::<LoggerService>(logger.clone());
    
    let db = Arc::new(PostgresDatabase::new(config.clone()));
    registry.register_as::<dyn DatabaseProvider, PostgresDatabase>(db.clone());
    
    let user_service = Arc::new(UserService::new(db, logger, config));
    registry.register::<UserService>(user_service);
    
    // Resolve and use a service
    let users = registry.resolve::<UserService>().unwrap();
    let user = users.get_user("123").unwrap();
}
```

## Using Traits for Abstraction

Navius encourages using traits to define service interfaces:

```rust
pub trait DatabaseProvider: Send + Sync {
    fn get_user(&self, id: &str) -> Result<User, Error>;
    fn get_user_with_timeout(&self, id: &str, timeout: u64) -> Result<User, Error>;
    fn create_user(&self, user: User) -> Result<User, Error>;
    fn update_user(&self, user: User) -> Result<User, Error>;
    fn delete_user(&self, id: &str) -> Result<(), Error>;
}

// Concrete implementation
pub struct PostgresDatabase {
    config: Arc<ConfigService>,
    pool: Pool<Postgres>,
}

impl DatabaseProvider for PostgresDatabase {
    fn get_user(&self, id: &str) -> Result<User, Error> {
        // Implementation using PostgreSQL
    }
    
    // Other method implementations...
}

// Alternative implementation for testing
pub struct MockDatabase {
    users: HashMap<String, User>,
}

impl DatabaseProvider for MockDatabase {
    fn get_user(&self, id: &str) -> Result<User, Error> {
        // Implementation using in-memory storage
        self.users.get(id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("User not found: {}", id)))
    }
    
    // Other method implementations...
}
```

## Dependency Injection in Web Handlers

For Axum web handlers, Navius supports DI through Axum's extension mechanism:

```rust
use axum::{
    extract::{Path, Extension},
    response::Json,
};
use std::sync::Arc;

async fn get_user_handler(
    Path(user_id): Path<String>,
    Extension(user_service): Extension<Arc<UserService>>,
) -> Json<User> {
    let user = user_service.get_user(&user_id).unwrap();
    Json(user)
}

// Setting up the router with extensions
fn build_router(registry: &ServiceRegistry) -> Router {
    let user_service = registry.resolve::<UserService>().unwrap();
    
    Router::new()
        .route("/users/:id", get(get_user_handler))
        .layer(Extension(user_service))
}
```

## Application State

For more complex applications, Navius uses an `AppState` struct to manage all dependencies:

```rust
pub struct AppState {
    registry: ServiceRegistry,
}

impl AppState {
    pub fn new() -> Self {
        let mut registry = ServiceRegistry::new();
        
        // Register all services...
        
        Self { registry }
    }
    
    pub fn get<T: 'static + ?Sized>(&self) -> Arc<T> {
        self.registry.resolve::<T>()
            .expect(&format!("Service {} not registered", std::any::type_name::<T>()))
    }
}

// Using AppState in router
fn build_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/users/:id", get(get_user_handler))
        .layer(Extension(app_state.get::<UserService>()))
}
```

## Common Patterns

### Factory Pattern

For services that need to create other services:

```rust
struct RepositoryFactory {
    db_connection: Arc<DatabaseConnection>,
    logger: Arc<LoggerService>,
}

impl RepositoryFactory {
    pub fn new(
        db_connection: Arc<DatabaseConnection>,
        logger: Arc<LoggerService>,
    ) -> Self {
        Self {
            db_connection,
            logger,
        }
    }
    
    pub fn create_user_repository(&self) -> Arc<UserRepository> {
        Arc::new(UserRepository::new(
            self.db_connection.clone(),
            self.logger.clone(),
        ))
    }
    
    pub fn create_product_repository(&self) -> Arc<ProductRepository> {
        Arc::new(ProductRepository::new(
            self.db_connection.clone(),
            self.logger.clone(),
        ))
    }
}
```

### Lazy Initialization

For services with expensive initialization:

```rust
struct LazyService {
    config: Arc<ConfigService>,
    connection: Mutex<Option<DatabaseConnection>>,
}

impl LazyService {
    pub fn new(config: Arc<ConfigService>) -> Self {
        Self {
            config,
            connection: Mutex::new(None),
        }
    }
    
    pub fn get_connection(&self) -> Result<&DatabaseConnection, Error> {
        let mut conn = self.connection.lock().unwrap();
        
        if conn.is_none() {
            let db_url = self.config.get_string("database.url")
                .ok_or_else(|| Error::Configuration("Database URL not found".to_string()))?;
                
            *conn = Some(DatabaseConnection::connect(&db_url)?);
        }
        
        Ok(conn.as_ref().unwrap())
    }
}
```

### Service Collections

For managing multiple similar services:

```rust
struct HealthCheckRegistry {
    checks: Vec<Arc<dyn HealthCheck>>,
}

impl HealthCheckRegistry {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }
    
    pub fn register(&mut self, check: Arc<dyn HealthCheck>) {
        self.checks.push(check);
    }
    
    pub async fn check_all(&self) -> HealthStatus {
        // Run all health checks
    }
}
```

## Testing with Dependency Injection

DI makes testing much easier:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[test]
    fn test_user_service() {
        // Create mock dependencies
        let config = Arc::new(MockConfigService::new());
        let logger = Arc::new(MockLoggerService::new());
        let database = Arc::new(MockDatabase::new());
        
        // Configure mocks
        database.expect_get_user()
            .with("123")
            .returns(Ok(User { id: "123".to_string(), name: "Test User".to_string() }));
            
        // Create service with mock dependencies
        let user_service = UserService::new(database, logger, config);
        
        // Test the service
        let user = user_service.get_user("123").unwrap();
        assert_eq!(user.id, "123");
        assert_eq!(user.name, "Test User");
    }
}
```

## Best Practices

1. **Use Traits for Abstraction**: Define service interfaces using traits
2. **Constructor Injection**: Pass dependencies through constructors
3. **Single Responsibility**: Each service should have a single responsibility
4. **Explicit Dependencies**: Make all dependencies explicit in constructors
5. **Central Registration**: Register all services in one place
6. **Avoid Service Locator**: Don't use service locator pattern within services
7. **Immutable Services**: Design services to be immutable after construction
8. **Circular Dependency Prevention**: Avoid circular dependencies between services

## Common Pitfalls

1. **Circular Dependencies**: A depends on B which depends on A
   - Solution: Break circular dependency with a third service or restructure
   
2. **Service Locator Anti-pattern**: Services fetch dependencies directly from registry
   - Solution: Always use constructor injection

3. **Deep Dependency Trees**: Services with too many layers of dependencies
   - Solution: Use factories or restructure services

4. **Mock Complexity**: Tests require many complex mocks
   - Solution: Simplify service interfaces and responsibilities

## Related Guides

- [Service Registration](service-registration.md) for managing service instances
- [Application Structure](application-structure.md) for overall application organization
- [Testing](testing.md) for testing services and dependencies 