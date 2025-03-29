---
title: "Service Registration Pattern"
description: "Design and implementation of the service registration pattern in Navius"
category: patterns
tags:
  - patterns
  - service
  - dependency-injection
  - architecture
related:
  - reference/patterns/repository-pattern.md
  - examples/dependency-injection-example.md
last_updated: March 27, 2025
version: 1.0
---

# Service Registration Pattern

## Overview

The Service Registration Pattern in Navius provides a centralized mechanism for registering, retrieving, and managing services throughout an application. This pattern facilitates dependency injection, promotes loose coupling between components, and improves testability.

## Problem Statement

Modern applications often consist of multiple interdependent services that need to work together. This creates several challenges:

1. **Service Discovery**: How do components locate the services they depend on?
2. **Lifecycle Management**: How are service instances created, shared, and potentially disposed?
3. **Dependency Resolution**: How are dependencies between services managed?
4. **Configuration Injection**: How are services configured based on application settings?
5. **Testability**: How can services be easily mocked or replaced in tests?

## Solution: Service Registration Pattern

The Service Registration Pattern in Navius addresses these challenges through a centralized registry that manages service instances and their dependencies:

1. **ServiceRegistry**: A central container for all services
2. **Type-Based Lookup**: Services are registered and retrieved by their type
3. **Dependency Injection**: Services declare their dependencies explicitly
4. **Lifecycle Management**: The registry manages service instantiation and sharing

### Pattern Structure

```
┌───────────────────┐
│  ServiceRegistry  │
└─────────┬─────────┘
          │
          │ contains
          ▼
┌───────────────────┐     depends on     ┌───────────────────┐
│     ServiceA      │◄────────────────────│     ServiceB      │
└───────────────────┘                    └───────────────────┘
          ▲                                       ▲
          │                                       │
          │ implements                            │ implements
          │                                       │
┌───────────────────┐                    ┌───────────────────┐
│   ServiceATrait   │                    │   ServiceBTrait   │
└───────────────────┘                    └───────────────────┘
```

## Implementation

### Core Service Registry

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::core::error::AppError;

pub struct ServiceRegistry {
    services: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    pub fn register<T: 'static + Send + Sync>(&self, service: T) -> Result<(), AppError> {
        let mut services = self.services.write().map_err(|_| {
            AppError::internal_server_error("Failed to acquire write lock on service registry")
        })?;
        
        let type_id = TypeId::of::<T>();
        services.insert(type_id, Box::new(service));
        
        Ok(())
    }

    pub fn get<T: 'static + Clone + Send + Sync>(&self) -> Result<Arc<T>, AppError> {
        let services = self.services.read().map_err(|_| {
            AppError::internal_server_error("Failed to acquire read lock on service registry")
        })?;
        
        let type_id = TypeId::of::<T>();
        
        match services.get(&type_id) {
            Some(service) => {
                if let Some(service_ref) = service.downcast_ref::<T>() {
                    Ok(Arc::new(service_ref.clone()))
                } else {
                    Err(AppError::internal_server_error(
                        format!("Service of type {:?} exists but could not be downcast", type_id)
                    ))
                }
            },
            None => Err(AppError::service_not_found(
                format!("No service of type {:?} found in registry", type_id)
            )),
        }
    }
}
```

### Service Definition

Services in Navius are defined as structs that implement a specific functionality:

```rust
pub struct UserService {
    // Service state
    config: Arc<AppConfig>,
    repository: Arc<dyn UserRepository>,
}

impl UserService {
    // Constructor that accepts dependencies
    pub fn new(config: Arc<AppConfig>, repository: Arc<dyn UserRepository>) -> Self {
        Self {
            config,
            repository,
        }
    }
    
    // Service methods
    pub async fn get_user(&self, id: &str) -> Result<User, AppError> {
        self.repository.find_by_id(id).await
    }
    
    pub async fn create_user(&self, user: User) -> Result<User, AppError> {
        self.repository.save(user).await
    }
}
```

### Service Registration

Services are registered during application startup:

```rust
// Create and configure the service registry
let registry = Arc::new(ServiceRegistry::new());

// Load configuration
let config = load_config()?;

// Create dependencies
let user_repository = Arc::new(PostgresUserRepository::new(config.clone()));

// Create the user service with its dependencies
let user_service = UserService::new(config.clone(), user_repository.clone());

// Register the service in the registry
registry.register(user_service)?;
```

### Service Retrieval

Services are retrieved from the registry when needed:

```rust
async fn handle_get_user(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id): Path<String>,
) -> Result<Json<User>, AppError> {
    // Get the service from the registry
    let user_service = registry.get::<UserService>()?;
    
    // Use the service
    let user = user_service.get_user(&id).await?;
    
    Ok(Json(user))
}
```

## Benefits

1. **Centralized Service Management**: Single point of access for all services
2. **Lifecycle Control**: Registry controls how services are instantiated and shared
3. **Loose Coupling**: Components depend on interfaces, not implementations
4. **Testability**: Services can be easily mocked or replaced in tests
5. **Configuration Injection**: Configuration is consistently provided to services
6. **Type Safety**: Type-based lookup ensures services are properly typed

## Advanced Techniques

### Trait-Based Registration

For greater flexibility, services can be registered based on traits they implement:

```rust
// Define a trait
pub trait Logger: Send + Sync {
    fn log(&self, message: &str);
}

// Implement the trait
pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("LOG: {}", message);
    }
}

// Register based on trait
registry.register_trait::<dyn Logger, _>(ConsoleLogger)?;

// Retrieve based on trait
let logger = registry.get_trait::<dyn Logger>()?;
logger.log("Hello, world!");
```

### Scoped Service Registration

For services with different lifetimes:

```rust
// Singleton scope (default)
registry.register::<UserService>(user_service)?;

// Request scope
registry.register_scoped::<RequestContext>(|| RequestContext::new())?;
```

### Factory Registration

For services that need dynamic creation:

```rust
// Register a factory
registry.register_factory::<Connection>(|| {
    let conn = create_database_connection(config.database_url);
    Box::new(conn)
})?;
```

### Automatic Dependency Resolution

For more advanced dependency injection:

```rust
// Register components
registry.register::<Config>(config)?;
registry.register::<DatabasePool>(pool)?;

// Automatically resolve and create UserService with its dependencies
let user_service = registry.resolve::<UserService>()?;
```

## Implementation Considerations

1. **Thread Safety**: All services and the registry itself must be thread-safe (Send + Sync)
2. **Error Handling**: Well-defined error types for registration and lookup failures
3. **Performance**: Efficient access to services in high-throughput scenarios
4. **Memory Management**: Proper handling of service lifecycle and cleanup
5. **Circular Dependencies**: Detection and prevention of circular dependencies

## Usage Examples

### Basic Service Registration

```rust
// Create registry
let registry = Arc::new(ServiceRegistry::new());

// Register services
registry.register(UserService::new(config.clone(), user_repo.clone()))?;
registry.register(AuthService::new(config.clone(), user_service.clone()))?;

// Create router with registry
let app = Router::new()
    .route("/users", get(get_users))
    .with_state(registry);
```

### Testing with Mock Services

```rust
#[tokio::test]
async fn test_user_service() {
    // Create registry with mock dependencies
    let registry = Arc::new(ServiceRegistry::new());
    let mock_repository = Arc::new(MockUserRepository::new());
    
    // Set up mock expectations
    mock_repository.expect_find_by_id()
        .with(eq("user-1"))
        .returning(|_| Ok(User::new("user-1", "Test User")));
    
    // Register service with mock dependency
    let user_service = UserService::new(Arc::new(AppConfig::default()), mock_repository);
    registry.register(user_service)?;
    
    // Create handler with registry
    let handler = get_user_handler(registry);
    
    // Test the handler
    let response = handler(Path("user-1".to_string())).await;
    
    // Verify response
    assert!(response.is_ok());
    let user = response.unwrap().0;
    assert_eq!(user.id, "user-1");
    assert_eq!(user.name, "Test User");
}
```

## Related Patterns

- **Dependency Injection Pattern**: Service Registration is a form of dependency injection
- **Factory Pattern**: Used for creating service instances
- **Strategy Pattern**: Services often implement different strategies
- **Singleton Pattern**: Services are typically singleton instances
- **Repository Pattern**: Commonly used with service registration for data access

## References

- [Dependency Injection Example](../../examples/dependency-injection-example.md)
- [Custom Service Example](../../examples/custom-service-example.md) 