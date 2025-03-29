---
title: "Custom Service Example"
description: ""
category: "Documentation"
tags: []
last_updated: "March 28, 2025"
version: "1.0"
---

---
title: "Creating and Using Custom Services in Navius"
description: "Comprehensive guide to designing, implementing and registering custom services with dependency injection in Navius applications"
category: examples
tags:
  - services
  - dependency-injection
  - service-registry
  - business-logic
  - application-state
  - testability
related:
  - 02_examples/dependency-injection-example.md
  - 02_examples/rest-api-example.md
  - 04_guides/service-design-patterns.md
last_updated: March 27, 2025
version: 1.1
status: stable
---

# Custom Service Example

This example demonstrates how to create and register custom services in a Navius application. Services are the building blocks for business logic in Navius, and understanding how to create and use them is essential for developing robust applications.

## Overview

Services in Navius encapsulate business logic and provide a way to organize functionality into cohesive, reusable components. The service pattern promotes:

- **Separation of concerns** - keeping business logic separate from request handling
- **Testability** - making it easier to unit test business logic
- **Reusability** - allowing functionality to be used across different parts of the application
- **Dependency injection** - enabling loose coupling between components

This example creates a user management system with notification capabilities, demonstrating how to design, implement, register, and use custom services in a Navius application.

## Quick Navigation

- [Project Structure](#project-structure)
- [Implementation](#implementation)
  - [Service Registry](#core-service-registry)
  - [User Model](#user-model)
  - [User Service](#user-service)
  - [Notification Service](#notification-service)
  - [API Handlers](#api-handlers)
  - [Application Entry Point](#application-entry-point)
- [Configuration](#configuration)
- [Running the Example](#running-the-example)
- [Testing the API](#testing-the-api)
- [Key Concepts](#key-concepts)
- [Best Practices](#best-practices)
- [Design Patterns](#design-patterns)
- [Testing Services](#testing-services)
- [Common Pitfalls](#common-pitfalls)
- [Advanced Topics](#advanced-topics)

## Prerequisites

Before working with this example, you should be familiar with:

- Rust programming basics
- Basic understanding of web services architecture
- HTTP and RESTful principles
- Dependency injection concepts (helpful but not required)

Required dependencies:
- Rust 1.70 or newer
- Navius 0.1.0 or newer
- tokio for asynchronous operations
- axum for HTTP routing
- serde for serialization/deserialization

## Project Structure

```
custom-service-example/
├── Cargo.toml
├── config/
│   └── default.yaml
└── src/
    ├── main.rs                  # Application entry point
    ├── app/
    │   ├── mod.rs
    │   ├── api/                 # HTTP handlers
    │   │   ├── mod.rs
    │   │   └── user_handler.rs  # User API handlers
    │   ├── models/              # Domain models
    │   │   ├── mod.rs
    │   │   └── user.rs          # User model
    │   └── services/            # Business logic
    │       ├── mod.rs
    │       ├── user_service.rs  # User management service
    │       └── notification_service.rs # Notification service
    └── core/                    # Core framework components
        ├── mod.rs
        ├── config.rs            # Configuration loading
        ├── error.rs             # Error handling
        ├── router.rs            # Router setup
        └── services/
            ├── mod.rs
            └── service_registry.rs # DI container
```

## Implementation

### Core Service Registry

A key part of Navius is the `ServiceRegistry`, which facilitates dependency injection and service management.

#### `core/services/service_registry.rs`

```
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::core::error::AppError;

#[derive(Default)]
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

    pub fn get<T: 'static + Send + Sync>(&self) -> Result<Arc<T>, AppError> {
        let services = self.services.read().map_err(|_| {
            AppError::internal_server_error("Failed to acquire read lock on service registry")
        })?;
        
        let type_id = TypeId::of::<T>();
        
        match services.get(&type_id) {
            Some(service) => {
                // Attempt to downcast to requested type
                if let Some(service_ref) = service.downcast_ref::<T>() {
                    // Clone and wrap in Arc
                    let service_clone = service_ref.clone();
                    Ok(Arc::new(service_clone))
                } else {
                    Err(AppError::internal_server_error(
                        format!("Service of type {:?} exists but could not be downcast to requested type", type_id)
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

### User Model

#### `app/models/user.rs`

```
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User {{ id: {}, name: {}, email: {} }}", self.id, self.name, self.email)
    }
}
```

### User Service

#### `app/services/user_service.rs`

```
use crate::app::models::user::User;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct UserService {
    users: Arc<Mutex<HashMap<String, User>>>,
}

impl UserService {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        
        // Add some sample users
        users.insert("1".to_string(), User {
            id: "1".to_string(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        });
        
        users.insert("2".to_string(), User {
            id: "2".to_string(),
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        });
        
        Self {
            users: Arc::new(Mutex::new(users)),
        }
    }
    
    pub fn get_user(&self, id: &str) -> Option<User> {
        let users = self.users.lock().unwrap();
        users.get(id).cloned()
    }
    
    pub fn get_all_users(&self) -> Vec<User> {
        let users = self.users.lock().unwrap();
        users.values().cloned().collect()
    }
    
    pub fn create_user(&self, user: User) -> User {
        let mut users = self.users.lock().unwrap();
        users.insert(user.id.clone(), user.clone());
        user
    }
    
    pub fn update_user(&self, id: &str, user: User) -> Option<User> {
        let mut users = self.users.lock().unwrap();
        
        if users.contains_key(id) {
            users.insert(id.to_string(), user.clone());
            Some(user)
        } else {
            None
        }
    }
    
    pub fn delete_user(&self, id: &str) -> bool {
        let mut users = self.users.lock().unwrap();
        users.remove(id).is_some()
    }
}
```

### Notification Service

#### `app/services/notification_service.rs`

```
use crate::app::models::user::User;
use std::sync::Arc;

#[derive(Clone)]
pub struct NotificationService {
    pub enabled: bool,
}

impl NotificationService {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
    
    pub fn send_welcome_notification(&self, user: &User) {
        if !self.enabled {
            println!("Notifications disabled");
            return;
        }
        
        println!("Sending welcome notification to {}: Welcome to our platform!", user.email);
    }
    
    pub fn send_update_notification(&self, user: &User) {
        if !self.enabled {
            println!("Notifications disabled");
            return;
        }
        
        println!("Sending update notification to {}: Your profile was updated", user.email);
    }
}
```

### API Handlers

#### `app/api/user_handler.rs`

```
use crate::app::models::user::User;
use crate::app::services::user_service::UserService;
use crate::app::services::notification_service::NotificationService;
use crate::core::error::AppError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

// AppState with service registry
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub notification_service: Arc<NotificationService>,
}

// Get all users
pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = state.user_service.get_all_users();
    Ok(Json(users))
}

// Get user by ID
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.get_user(&id)
        .ok_or_else(|| AppError::not_found(format!("User with ID {} not found", id)))?;
    
    Ok(Json(user))
}

// Create new user
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(user): Json<User>,
) -> Result<impl IntoResponse, AppError> {
    let created_user = state.user_service.create_user(user);
    
    // Send notification using the notification service
    state.notification_service.send_welcome_notification(&created_user);
    
    Ok((StatusCode::CREATED, Json(created_user)))
}

// Update existing user
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(user): Json<User>,
) -> Result<Json<User>, AppError> {
    let updated_user = state.user_service.update_user(&id, user)
        .ok_or_else(|| AppError::not_found(format!("User with ID {} not found", id)))?;
    
    // Send notification using the notification service
    state.notification_service.send_update_notification(&updated_user);
    
    Ok(Json(updated_user))
}

// Delete user
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let deleted = state.user_service.delete_user(&id);
    
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("User with ID {} not found", id)))
    }
}
```

### Application Entry Point

#### `main.rs`

```
use std::net::SocketAddr;
use std::sync::Arc;
use axum::{Router, routing::{get, post, put, delete}};
use crate::app::api::user_handler::{AppState, get_users, get_user, create_user, update_user, delete_user};
use crate::app::services::user_service::UserService;
use crate::app::services::notification_service::NotificationService;
use crate::core::config::load_config;
use crate::core::error::AppError;
use crate::core::services::service_registry::ServiceRegistry;

mod app;
mod core;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = load_config()?;
    
    // Create service registry
    let registry = ServiceRegistry::new();
    
    // Register custom services
    let user_service = UserService::new();
    registry.register(user_service.clone())?;
    
    let notification_service = NotificationService::new(config.notifications_enabled);
    registry.register(notification_service.clone())?;
    
    // Create app state
    let app_state = Arc::new(AppState {
        user_service: Arc::new(user_service),
        notification_service: Arc::new(notification_service),
    });
    
    // Build router
    let app = Router::new()
        .route("/users", get(get_users))
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
        .with_state(app_state);
    
    // Extract server address
    let addr: SocketAddr = format!("{}:{}", config.server.address, config.server.port).parse()
        .map_err(|_| AppError::configuration_error("Invalid server address"))?;
    
    // Start server
    tracing::info!("Starting server on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| AppError::internal_server_error(format!("Server error: {}", e)))?;
    
    Ok(())
}
```

## Configuration

### `config/default.yaml`

```
server:
  address: "127.0.0.1"
  port: 3000

app_name: "Custom Service Example"
notifications_enabled: true
```

## Running the Example

1. Clone the Navius repository:
   ```bash
   git clone https://github.com/navius/examples.git
   cd examples/custom-service-example
   ```

2. Build and run the application:
   ```bash
   cargo run
   ```

3. The server will start on `http://localhost:3000`

## Testing the API

Test the endpoints using curl or any HTTP client:

### Get all users

```
curl http://localhost:3000/users
```

Sample response:
```
[
  {"id":"1","name":"Alice","email":"alice@example.com"},
  {"id":"2","name":"Bob","email":"bob@example.com"}
]
```

### Get a specific user

```
curl http://localhost:3000/users/1
```

Sample response:
```
{"id":"1","name":"Alice","email":"alice@example.com"}
```

### Create a new user

```
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"id": "3", "name": "Charlie", "email": "charlie@example.com"}'
```

Sample response:
```
{"id":"3","name":"Charlie","email":"charlie@example.com"}
```

### Update a user

```
curl -X PUT http://localhost:3000/users/3 \
  -H "Content-Type: application/json" \
  -d '{"id": "3", "name": "Charlie Updated", "email": "charlie@example.com"}'
```

Sample response:
```
{"id":"3","name":"Charlie Updated","email":"charlie@example.com"}
```

### Delete a user

```
curl -X DELETE http://localhost:3000/users/3
```

## Key Concepts

1. **Service-Oriented Architecture**
   - Services encapsulate business logic and domain operations
   - Each service has a clear responsibility and domain focus
   - Services can be composed and used by other services or handlers

2. **Dependency Injection**
   - Services are registered with a central registry
   - Components request services from the registry instead of creating them
   - This promotes loose coupling and testability

3. **Service Lifecycle Management**
   - Services can be stateful or stateless
   - The registry handles creation, retrieval, and ownership
   - Arc (Atomic Reference Counting) is used to share services safely

4. **Service Communication**
   - Services can communicate with each other
   - In this example, the User service operations trigger Notification service actions
   - Services can be composed to build complex behavior from simple components

## Best Practices

### Service Design

1. **Single Responsibility Principle**
   - Each service should focus on a single domain or functionality
   - Keep services small and focused (e.g., UserService handles user CRUD, NotificationService handles notifications)

2. **Interface Segregation**
   - Consider using traits to define service interfaces
   - Clients should only depend on methods they actually use

3. **Immutability**
   - Prefer immutable data structures when possible
   - Use interior mutability patterns (Mutex, RwLock) for thread-safe state management

4. **Configuration Injection**
   - Configure services based on application settings
   - Pass configuration during service initialization rather than hardcoding values

### Error Handling

1. **Propagate Errors Appropriately**
   - Use Result<T, E> for operations that can fail
   - Define clear error types that provide useful information

2. **Graceful Failure**
   - Services should fail gracefully and communicate failures clearly
   - Handle expected failure cases directly in the service

3. **Transaction Safety**
   - Ensure operations that modify multiple resources are atomic or can be rolled back
   - Consider using transactions for database operations

### Testing

1. **Testability**
   - Design services with testing in mind
   - Use dependency injection to mock dependencies in tests

2. **Unit Testing**
   - Test service methods in isolation
   - Mock any external dependencies

3. **Integration Testing**
   - Test service interactions together
   - Verify that services work correctly when composed

## Design Patterns

### Repository Pattern

For services that manage data, consider using the Repository pattern:

```
pub trait UserRepository {
    fn find_by_id(&self, id: &str) -> Option<User>;
    fn find_all(&self) -> Vec<User>;
    fn create(&self, user: User) -> User;
    fn update(&self, id: &str, user: User) -> Option<User>;
    fn delete(&self, id: &str) -> bool;
}

// Then inject repository into service
pub struct UserService {
    repository: Arc<dyn UserRepository>,
}
```

### Factory Pattern

For complex service creation:

```
pub struct ServiceFactory {
    config: AppConfig,
}

impl ServiceFactory {
    pub fn create_user_service(&self) -> UserService {
        // Create and configure UserService
    }
    
    pub fn create_notification_service(&self) -> NotificationService {
        NotificationService::new(self.config.notifications_enabled)
    }
}
```

### Decorator Pattern

Add functionality to services without modifying them:

```
pub struct LoggingNotificationService {
    inner: Arc<NotificationService>,
}

impl LoggingNotificationService {
    pub fn send_welcome_notification(&self, user: &User) {
        tracing::info!("Sending welcome notification to {}", user.email);
        self.inner.send_welcome_notification(user)
    }
}
```

## Testing Services

### Unit Testing Example

```
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_service_crud() {
        let service = UserService::new();
        
        // Test creating a user
        let user = User {
            id: "test".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let created = service.create_user(user.clone());
        assert_eq!(created.id, "test");
        
        // Test retrieving a user
        let retrieved = service.get_user("test").unwrap();
        assert_eq!(retrieved.name, "Test User");
        
        // Test updating a user
        let updated_user = User {
            id: "test".to_string(),
            name: "Updated User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let updated = service.update_user("test", updated_user).unwrap();
        assert_eq!(updated.name, "Updated User");
        
        // Test deleting a user
        let deleted = service.delete_user("test");
        assert!(deleted);
        
        // Verify user is gone
        assert!(service.get_user("test").is_none());
    }
}
```

### Testing Services with Mocks

```
// Using mockall to create a mock NotificationService
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    
    mock! {
        NotificationService {
            fn send_welcome_notification(&self, user: &User);
        }
    }
    
    #[test]
    fn test_user_creation_triggers_notification() {
        // Create mock
        let mut mock_notification = MockNotificationService::new();
        mock_notification
            .expect_send_welcome_notification()
            .with(predicate::function(|user: &User| user.id == "test"))
            .times(1)
            .return_const(());
            
        // Test that creating a user triggers a notification
        // (Implementation depends on how the services are integrated)
    }
}
```

## Common Pitfalls

1. **Circular Dependencies**
   - Services that depend on each other can create circular dependencies
   - Solution: Use interfaces/traits, restructure services, or introduce mediator services

2. **Over-Engineering**
   - Creating too many small services can lead to complexity
   - Balance between cohesion and simplicity

3. **Thread Safety Issues**
   - Services in web applications need to be thread-safe
   - Use Arc, Mutex, and RwLock appropriately
   - Be cautious of deadlocks when multiple locks are held

4. **Improper Error Handling**
   - Not propagating errors correctly
   - Not providing useful error information

5. **Service Lifetime Mismatches**
   - Some services may have different lifecycles (e.g., per-request vs. application-wide)
   - Be explicit about service lifetimes

## Advanced Topics

### Service Lifetimes

Different services may have different lifetimes:

- **Singleton**: One instance for the entire application
- **Scoped**: One instance per scope (e.g., per request)
- **Transient**: New instance each time it's requested

### Conditional Service Registration

Register different implementations based on configuration:

```
if config.use_mock_services {
    registry.register(MockUserService::new())?;
} else {
    registry.register(RealUserService::new(db_connection))?;
}
```

### Async Services

For services that perform async operations:

```
#[async_trait]
pub trait AsyncUserService {
    async fn get_user(&self, id: &str) -> Result<User, AppError>;
    async fn create_user(&self, user: User) -> Result<User, AppError>;
    // ...
}
```

### Service Middleware

Intercept and modify service behavior:

```
pub struct ServiceMiddleware<S> {
    inner: S,
    before: Box<dyn Fn() -> ()>,
    after: Box<dyn Fn() -> ()>,
}

impl<S> ServiceMiddleware<S> {
    // Proxy methods to inner service with before/after hooks
}
```

## Next Steps

- Explore the [Dependency Injection Example](02_examples/dependency-injection-example.md) for more advanced DI patterns
- Learn about structured error handling in the [Error Handling Example](02_examples/error-handling-example.md)
- See services in action with a complete API in the [REST API Example](02_examples/rest-api-example.md)
</rewritten_file> 
