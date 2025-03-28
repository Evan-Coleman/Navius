---
title: "Custom Service Example"
description: "How to create and register custom services in Navius"
category: examples
tags:
  - examples
  - services
  - dependency-injection
related:
  - examples/basic-application-example.md
  - examples/dependency-injection-example.md
  - guides/service-registration.md
last_updated: March 26, 2025
version: 1.0
---

# Custom Service Example

This example demonstrates how to create and register custom services in a Navius application. Services are the building blocks for business logic in Navius, and understanding how to create and use them is essential for developing robust applications.

## Project Structure

```
custom-service-example/
├── Cargo.toml
├── config/
│   └── default.yaml
└── src/
    ├── main.rs
    ├── app/
    │   ├── mod.rs
    │   ├── api/
    │   │   ├── mod.rs
    │   │   └── user_handler.rs
    │   ├── models/
    │   │   ├── mod.rs
    │   │   └── user.rs
    │   └── services/
    │       ├── mod.rs
    │       ├── user_service.rs
    │       └── notification_service.rs
    └── core/
        ├── mod.rs
        ├── config.rs
        ├── error.rs
        ├── router.rs
        └── services/
            ├── mod.rs
            └── service_registry.rs
```

## Implementation

### Core Service Registry

A key part of Navius is the `ServiceRegistry`, which facilitates dependency injection and service management.

#### `core/services/service_registry.rs`

```rust
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

### Custom Services Implementation

#### `app/models/user.rs`

```rust
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

#### `app/services/user_service.rs`

```rust
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

#### `app/services/notification_service.rs`

```rust
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

### Service Usage in Handlers

#### `app/api/user_handler.rs`

```rust
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

#### `main.rs`

```rust
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

```yaml
server:
  address: "127.0.0.1"
  port: 3000

app_name: "Custom Service Example"
notifications_enabled: true
```

## Running the Example

1. Clone the Navius repository
2. Navigate to the `examples/custom-service-example` directory
3. Run the application:

```bash
cargo run
```

4. Test the endpoints:

```bash
# Get all users
curl http://localhost:3000/users

# Get a specific user
curl http://localhost:3000/users/1

# Create a new user
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"id": "3", "name": "Charlie", "email": "charlie@example.com"}'

# Update a user
curl -X PUT http://localhost:3000/users/3 \
  -H "Content-Type: application/json" \
  -d '{"id": "3", "name": "Charlie Updated", "email": "charlie@example.com"}'

# Delete a user
curl -X DELETE http://localhost:3000/users/3
```

## Key Concepts Demonstrated

1. **Creating Custom Services**: Defining service classes with specific functionality
2. **Service Registration**: Adding services to the ServiceRegistry
3. **Dependency Injection**: Injecting services into handlers
4. **Service Interaction**: Having services work together (UserService and NotificationService)
5. **State Management**: Sharing application state with handlers

## Best Practices

1. **Service Design**: Keep services focused on a specific domain or functionality
2. **Immutability**: Use immutable data structures when possible
3. **Error Handling**: Properly propagate errors up the call stack
4. **Testing**: Create services with testability in mind
5. **Configuration**: Configure services based on application settings

## Advanced Topics

- **Service Lifetimes**: Understanding Arc and ownership patterns
- **Service Dependencies**: Managing services that depend on other services
- **Mock Services**: Creating test doubles for dependent services
- **Conditional Services**: Registering different implementations based on configuration

## Next Steps

- [Dependency Injection Example](dependency-injection-example.md): More advanced dependency injection patterns
- [Repository Pattern Example](repository-pattern-example.md): Using services with data repositories
- [Error Handling Example](error-handling-example.md): Comprehensive error handling strategies
</rewritten_file> 