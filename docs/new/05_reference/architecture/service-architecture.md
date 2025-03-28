# Service Architecture

This document outlines the service architecture used throughout the Navius framework, focusing on the design patterns and implementation details that enable flexible, extensible service composition.

## Core Concepts

The Navius service architecture is built around several key concepts:

1. **Service Traits**: Defined interfaces that services implement
2. **Service Implementations**: Concrete implementations of service traits
3. **Service Registry**: A central registry for accessing services
4. **Service Dependencies**: Explicit declaration of service dependencies
5. **Service Lifecycle**: Management of service initialization and cleanup

## Service Organization

```
src/
├── core/
│   ├── services/
│   │   ├── traits/         # Service trait definitions
│   │   ├── implementations/ # Default implementations
│   │   └── registry.rs     # Service registry
└── app/
    └── services/
        └── implementations/ # Application-specific implementations
```

## Service Traits

Service traits define the interface that service implementations must provide. They are typically defined in the `core/services/traits` directory:

```rust
// In src/core/services/traits/cache.rs
pub trait CacheService: Send + Sync + 'static {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError>;
    async fn set(&self, key: &str, value: String, ttl: Duration) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
    async fn clear(&self) -> Result<(), CacheError>;
}
```

Key aspects of service traits:
- They should be as minimal as possible, focusing on core functionality
- They should include appropriate bounds (Send, Sync, 'static) for async usage
- They should return Result types with specific error types
- They should be well-documented with examples

## Service Implementations

Service implementations provide concrete implementations of service traits. They are typically defined in the `core/services/implementations` directory for core implementations and `app/services/implementations` for application-specific implementations:

```rust
// In src/core/services/implementations/memory_cache.rs
pub struct MemoryCacheService {
    cache: RwLock<HashMap<String, CacheEntry>>,
}

impl MemoryCacheService {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl CacheService for MemoryCacheService {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let cache = self.cache.read().await;
        let entry = cache.get(key);
        
        match entry {
            Some(entry) if !entry.is_expired() => Ok(Some(entry.value.clone())),
            _ => Ok(None),
        }
    }
    
    // Other method implementations...
}
```

Key aspects of service implementations:
- They should implement the service trait fully
- They should be configurable through constructor parameters
- They should be well-tested with unit tests
- They should properly handle error conditions
- They may implement multiple service traits if appropriate

## Service Registry

The service registry is a central component for accessing services. It is responsible for:
- Storing service instances
- Providing type-safe access to services
- Managing service dependencies
- Ensuring services are initialized in the correct order

```rust
// In src/core/services/registry.rs
pub struct ServiceRegistry {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }
    
    pub fn register<S: Any + Send + Sync>(&mut self, service: S) {
        let type_id = TypeId::of::<S>();
        self.services.insert(type_id, Box::new(service));
    }
    
    pub fn get<S: Any + Send + Sync>(&self) -> Option<&S> {
        let type_id = TypeId::of::<S>();
        self.services.get(&type_id).and_then(|boxed| boxed.downcast_ref::<S>())
    }
}
```

## Service Dependencies

Services often depend on other services. These dependencies should be explicitly declared and injected through constructors:

```rust
// In src/core/services/implementations/tiered_cache.rs
pub struct TieredCacheService<P: CacheService, S: CacheService> {
    primary: P,
    secondary: S,
}

impl<P: CacheService, S: CacheService> TieredCacheService<P, S> {
    pub fn new(primary: P, secondary: S) -> Self {
        Self { primary, secondary }
    }
}

impl<P: CacheService, S: CacheService> CacheService for TieredCacheService<P, S> {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        // Try primary cache first
        match self.primary.get(key).await? {
            Some(value) => Ok(Some(value)),
            None => {
                // Try secondary cache
                match self.secondary.get(key).await? {
                    Some(value) => {
                        // Populate primary cache
                        let _ = self.primary.set(key, value.clone(), Duration::from_secs(3600)).await;
                        Ok(Some(value))
                    },
                    None => Ok(None),
                }
            }
        }
    }
    
    // Other method implementations...
}
```

Key aspects of service dependencies:
- Dependencies should be injected through constructors
- Generic type parameters should be used for flexibility
- Services should depend on traits, not concrete implementations
- Dependencies should be well-documented

## Service Initialization

Services are typically initialized during application startup through the service registry:

```rust
// In src/app/startup.rs
pub fn initialize_services(config: &AppConfig) -> ServiceRegistry {
    let mut registry = ServiceRegistry::new();
    
    // Create and register database service
    let db_service = PostgresDatabaseService::new(&config.database);
    registry.register::<dyn DatabaseService>(Box::new(db_service));
    
    // Create and register cache service
    let cache_service = RedisCacheService::new(&config.cache);
    registry.register::<dyn CacheService>(Box::new(cache_service));
    
    // Create and register user service, which depends on database service
    let db_service = registry.get::<dyn DatabaseService>().unwrap();
    let user_service = UserService::new(db_service);
    registry.register::<dyn UserService>(Box::new(user_service));
    
    registry
}
```

## Service Discovery

Services can be discovered and accessed through the service registry:

```rust
// In a request handler
pub async fn handle_request(
    Path(user_id): Path<String>,
    State(registry): State<Arc<ServiceRegistry>>,
) -> impl IntoResponse {
    // Get user service from registry
    let user_service = match registry.get::<dyn UserService>() {
        Some(service) => service,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, "User service not found").into_response(),
    };
    
    // Use the service
    match user_service.get_user(&user_id).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "User not found").into_response(),
    }
}
```

## Best Practices

### Keep Services Focused

Each service should have a single, well-defined responsibility. Services that try to do too much become difficult to test and maintain.

### Use Dependency Injection

Services should receive their dependencies through constructors, not create them internally. This enables easier testing and flexibility.

### Test Services in Isolation

Each service should be testable in isolation, without requiring its dependencies to be fully implemented. Use mocks or stubs for dependencies in tests.

### Document Service Contracts

Service traits should be well-documented, including example usage, error conditions, and performance characteristics.

### Consider Service Lifecycle

Services may need initialization (connecting to databases, loading caches) and cleanup (closing connections, flushing data). Ensure these are properly handled.

### Error Handling

Services should use specific error types that provide meaningful information about what went wrong. Generic error types make debugging difficult.

## Related Documents

- [Provider Architecture](provider-architecture.md)
- [Dependency Injection](../guides/dependency-injection.md)
- [Service Registration Pattern](../reference/patterns/service-registration-pattern.md) 