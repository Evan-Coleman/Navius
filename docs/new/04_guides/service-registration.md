# Service Registration

This guide explains how service registration works in Navius, focusing on the dependency injection system that allows services to be easily shared across your application.

## Overview

Service registration in Navius follows these key principles:

1. **Single-instance services**: Each service is typically instantiated once and shared
2. **Dependency injection**: Services can depend on other services
3. **Decoupled components**: Services interact through well-defined interfaces
4. **Runtime registration**: Services are registered and resolved at runtime

## The ServiceRegistry

The core of Navius's service registration system is the `ServiceRegistry` type:

```rust
pub struct ServiceRegistry {
    services: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}
```

This registry maintains a thread-safe collection of services, accessible by their type.

## Registering Services

Services can be registered in several ways:

### Basic Registration

```rust
let mut registry = ServiceRegistry::new();

// Register a service
let logger = Arc::new(LoggerService::new());
registry.register::<LoggerService>(logger);
```

### Registering with Dependencies

```rust
// Register a service with dependencies
let db_connection = Arc::new(DatabaseConnection::new());
registry.register::<DatabaseConnection>(db_connection.clone());

let user_repo = Arc::new(UserRepository::new(db_connection));
registry.register::<UserRepository>(user_repo);
```

### Registering Implementations of Traits

To register a concrete implementation of a trait:

```rust
// Register a concrete implementation for a trait interface
let cache = Arc::new(RedisCache::new());
registry.register_as::<dyn CacheProvider, RedisCache>(cache);
```

## Resolving Services

Once registered, services can be resolved by type:

```rust
// Resolve a service
let logger = registry.resolve::<LoggerService>().unwrap();
logger.log("Service resolved successfully!");

// Resolve a trait implementation
let cache = registry.resolve::<dyn CacheProvider>().unwrap();
cache.get("key");
```

## Complete Example

Here's a more complete example showing service registration and resolution:

```rust
use navius::core::service::ServiceRegistry;
use std::sync::Arc;

// Define services
struct LoggerService;
impl LoggerService {
    fn new() -> Self {
        Self
    }
    
    fn log(&self, message: &str) {
        println!("LOG: {}", message);
    }
}

struct UserService {
    logger: Arc<LoggerService>,
}

impl UserService {
    fn new(logger: Arc<LoggerService>) -> Self {
        Self { logger }
    }
    
    fn get_user(&self, id: &str) -> Result<String, String> {
        self.logger.log(&format!("Fetching user {}", id));
        Ok(format!("User {}", id))
    }
}

fn main() {
    // Create registry
    let mut registry = ServiceRegistry::new();
    
    // Register services
    let logger = Arc::new(LoggerService::new());
    registry.register::<LoggerService>(logger.clone());
    
    let user_service = Arc::new(UserService::new(logger));
    registry.register::<UserService>(user_service);
    
    // Resolve and use services
    let user_svc = registry.resolve::<UserService>().unwrap();
    let user = user_svc.get_user("123").unwrap();
    println!("Retrieved: {}", user);
}
```

## Using ServiceRegistry with AppState

In a typical Navius application, services are registered in the `AppState`:

```rust
pub struct AppState {
    registry: Arc<ServiceRegistry>,
}

impl AppState {
    pub fn new() -> Self {
        let mut registry = ServiceRegistry::new();
        
        // Register core services
        let config = Arc::new(ConfigService::new());
        registry.register::<ConfigService>(config.clone());
        
        let logger = Arc::new(LoggerService::new(config.clone()));
        registry.register::<LoggerService>(logger.clone());
        
        let db = Arc::new(DatabaseService::new(config.clone(), logger.clone()));
        registry.register::<DatabaseService>(db.clone());
        
        Self {
            registry: Arc::new(registry),
        }
    }
    
    pub fn get<T: 'static + ?Sized>(&self) -> Arc<T> {
        self.registry.resolve::<T>()
            .expect(&format!("Service {} not registered", std::any::type_name::<T>()))
    }
}
```

## Best Practices

1. **Use Traits for Service Interfaces**: Define services using traits to allow multiple implementations

   ```rust
   pub trait CacheProvider: Send + Sync {
       fn get(&self, key: &str) -> Option<String>;
       fn set(&self, key: &str, value: String);
   }
   ```

2. **Register Early, Resolve Late**: Register all services during application startup, resolve as needed

3. **Use Option or Result for Optional Services**: Some services might be conditionally available

   ```rust
   let metrics = registry.resolve::<MetricsService>().ok();
   if let Some(metrics) = metrics {
       metrics.record("api.request", 1);
   }
   ```

4. **Avoid Circular Dependencies**: Design your service hierarchy to avoid circular dependencies

5. **Group Related Services**: Organize services by domain functionality rather than technical concerns

## Advantages of Navius's Service Registration

1. **Type Safety**: Services are registered and resolved with full type information
2. **Thread Safety**: Services can be safely shared across threads
3. **Dependency Management**: Clear visualization of service dependencies
4. **Testability**: Services can be easily mocked or replaced in tests
5. **Flexibility**: Services can be conditionally registered based on configuration

## Common Patterns

### Feature-Gated Services

```rust
#[cfg(feature = "metrics")]
{
    let metrics = Arc::new(MetricsService::new(config.clone()));
    registry.register::<MetricsService>(metrics);
}
```

### Service Factory Pattern

```rust
struct RepositoryFactory {
    db_connection: Arc<DatabaseConnection>,
}

impl RepositoryFactory {
    fn new(db_connection: Arc<DatabaseConnection>) -> Self {
        Self { db_connection }
    }
    
    fn create_user_repository(&self) -> Arc<UserRepository> {
        Arc::new(UserRepository::new(self.db_connection.clone()))
    }
    
    fn create_product_repository(&self) -> Arc<ProductRepository> {
        Arc::new(ProductRepository::new(self.db_connection.clone()))
    }
}
```

## Related Guides

- [Dependency Injection](dependency-injection.md) for more details on DI patterns
- [Application Structure](application-structure.md) for overall application organization
- [Configuration](configuration.md) for configuring services 