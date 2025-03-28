---
title: "Building RESTful APIs with Navius"
description: "Comprehensive guide to creating robust and scalable REST APIs with Navius framework"
category: examples
tags: rest, api, http, crud, restful, json, endpoints, web-services, controllers, handlers, middleware, validation
related_documents: 
  - 02_examples/database-integration-example.md
  - 02_examples/error-handling-example.md
  - 01_getting_started/first-steps.md
  - 03_reference/01_api/router-api.md
last_updated: 2025-03-27
version: 1.1
status: stable
---

# Building RESTful APIs with Navius

## Overview

This guide demonstrates how to build a fully-featured RESTful API using the Navius framework. RESTful APIs provide a standardized approach for building web services that allow client applications to interact with your data and services. With Navius, you can create robust, performant, and maintainable REST endpoints with minimal boilerplate.

In this example, we'll build a task management API with CRUD operations, proper error handling, request validation, authentication, rate limiting, and documentation. By the end, you'll have a production-ready API following REST best practices.

## Quick Navigation

- [Overview](#overview)
- [Project Structure](#project-structure)
- [Implementation](#implementation)
  - [Set Up Dependencies](#set-up-dependencies)
  - [Define Data Models](#define-data-models)
  - [Create API Handlers](#create-api-handlers)
  - [Configure Routes](#configure-routes)
  - [Implement Authentication](#implement-authentication)
  - [Add Request Validation](#add-request-validation)
  - [Implement Error Handling](#implement-error-handling)
  - [Add Middleware](#add-middleware)
  - [Main Application](#main-application)
- [Testing Your API](#testing-your-api)
- [API Documentation](#api-documentation)
- [Best Practices](#best-practices)
- [Advanced Techniques](#advanced-techniques)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before starting this example, make sure you have:

- Navius framework installed (v0.5.0 or higher)
- Rust toolchain (1.65.0 or higher)
- Basic understanding of HTTP and REST principles
- Cargo package manager

## Project Structure

This example follows a structured approach to organizing your REST API project:

```
rest-api-example/
├── Cargo.toml                  # Project dependencies
├── config/
│   └── default.yaml            # Configuration values
├── src/
│   ├── main.rs                 # Application entry point
│   ├── api/
│   │   ├── mod.rs              # API module exports
│   │   ├── task_controller.rs  # Task API endpoints
│   │   └── auth_controller.rs  # Authentication endpoints
│   ├── models/
│   │   ├── mod.rs              # Models module exports
│   │   ├── task.rs             # Task data model
│   │   └── user.rs             # User data model
│   ├── repositories/
│   │   ├── mod.rs              # Repository module exports
│   │   ├── task_repository.rs  # Task storage and retrieval
│   │   └── user_repository.rs  # User storage and retrieval
│   ├── services/
│   │   ├── mod.rs              # Services module exports
│   │   ├── task_service.rs     # Task business logic
│   │   └── auth_service.rs     # Authentication logic
│   └── utils/
│       ├── mod.rs              # Utilities module exports
│       ├── middleware.rs       # Custom middleware
│       ├── validation.rs       # Request validation helpers
│       └── error_handler.rs    # Centralized error handling
└── tests/
    ├── api_tests.rs            # API integration tests
    └── service_tests.rs        # Service unit tests
```

## Implementation

### Set Up Dependencies

First, set up your project dependencies. Navius provides a clean, idiomatic way to build REST APIs in Rust with features like request validation, route handlers, error handling, and JSON serialization/deserialization.

### Define Data Models

Next, define the data models for your API. This includes the product structure and any associated request and response models.

### Create API Handlers

Create the API handlers for your endpoints. This includes the logic for handling requests and responses.

### Configure Routes

Configure the routes for your API. This includes setting up the routing system and associating routes with handlers.

### Implement Authentication

Implement authentication for your API. This includes setting up middleware for authentication and authorization.

### Add Request Validation

Add request validation to ensure that incoming requests are valid and sanitized.

### Implement Error Handling

Implement error handling to provide consistent and meaningful error responses.

### Add Middleware

Add middleware to your API for additional functionality, such as logging or rate limiting.

### Main Application

Finally, set up the main application and run the server.

## Testing Your API

You can test the API using curl, Postman, or any HTTP client. Here are some examples:

### List Products

```bash
curl -X GET http://localhost:8080/products
```

With filters:

```bash
curl -X GET "http://localhost:8080/products?category=electronics&min_price=100&page=1&limit=5"
```

### Get Product by ID

```bash
curl -X GET http://localhost:8080/products/123e4567-e89b-12d3-a456-426614174000
```

### Create Product

```bash
curl -X POST http://localhost:8080/products \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Wireless Headphones",
    "description": "High-quality wireless headphones with noise cancellation",
    "price": 199.99,
    "sku": "WH-2023-001",
    "stock": 50,
    "category": "electronics"
  }'
```

### Update Product

```bash
curl -X PUT http://localhost:8080/products/123e4567-e89b-12d3-a456-426614174000 \
  -H "Content-Type: application/json" \
  -d '{
    "price": 179.99,
    "stock": 45
  }'
```

### Delete Product

```bash
curl -X DELETE http://localhost:8080/products/123e4567-e89b-12d3-a456-426614174000
```

## API Documentation

The API provides the following endpoints:

| Method | Path | Description |
|--------|------|-------------|
| GET | /products | List all products with optional filtering and pagination |
| GET | /products/:id | Get a specific product by ID |
| POST | /products | Create a new product |
| PUT | /products/:id | Update an existing product |
| DELETE | /products/:id | Delete a product |

### Query Parameters for GET /products

| Parameter | Type | Description |
|-----------|------|-------------|
| page | integer | Page number for pagination (default: 1) |
| limit | integer | Number of items per page (default: 10) |
| category | string | Filter products by category |
| min_price | float | Filter products with price >= min_price |
| max_price | float | Filter products with price <= max_price |

## Best Practices

When building REST APIs with Navius, consider the following best practices:

1. **Use Proper HTTP Methods**:
   - GET for retrieving resources
   - POST for creating new resources
   - PUT for updating entire resources
   - PATCH for partial updates
   - DELETE for removing resources

2. **Return Appropriate Status Codes**:
   - 200 OK for successful GET, PUT, PATCH, DELETE
   - 201 Created for successful POST
   - 204 No Content for successful DELETE without response body
   - 400 Bad Request for invalid input
   - 404 Not Found for missing resources
   - 409 Conflict for resource conflicts
   - 500 Internal Server Error for server errors

3. **Validate Input**:
   - Always validate and sanitize input data
   - Return helpful validation error messages
   - Use the validator crate with #[derive(Validate)]

4. **Resource Naming**:
   - Use plural nouns for resources (e.g., /products not /product)
   - Use kebab-case for multi-word resources (e.g., /order-items)
   - Keep URLs simple and intuitive

5. **Implement Pagination**:
   - Provide pagination for list endpoints
   - Include metadata about pagination (total, page, limit)
   - Use query parameters for pagination control

6. **Proper Error Handling**:
   - Create consistent error response structures
   - Include meaningful error messages
   - Use typed errors with thiserror

## Advanced Techniques

### Resource-Based Routing

Organize routes based on resources for clarity:

```rust
Router::new()
    .nest("/products", products_routes())
    .nest("/orders", orders_routes())
    .nest("/customers", customers_routes())
```

### Response Envelope

For more detailed responses, consider using a response envelope:

```rust
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    metadata: Option<ResponseMetadata>,
}

#[derive(Serialize)]
struct ResponseMetadata {
    total: usize,
    page: usize,
    limit: usize,
}
```

### Service Registration

Make services available to the application:

```rust
let app = Application::new()
    .register_extension(product_service)
    .register_extension(order_service)
    .register_routes(api_routes())
    .build()?;
```

## Troubleshooting

### Common Issues

1. **404 Not Found for Valid Routes**:
   - Check route registration order
   - Verify URL path parameters
   - Check for trailing slashes

2. **Serialization Errors**:
   - Ensure all fields in your models implement Serialize/Deserialize
   - Check for circular references
   - Verify date/time formats

3. **Connection Errors**:
   - Verify network configuration
   - Check firewall settings
   - Ensure correct host/port in config

### Debugging Tips

1. Use tracing to debug request flow:

```rust
tracing::debug!("Processing product request: {:?}", product);
```

2. Enable more detailed logs by setting log level to debug or trace in your configuration.

3. Use middleware to log request/response information:

```rust
app.middleware(LoggingMiddleware::new())
```

## Next Steps

After mastering the basics of REST APIs with Navius, consider:

1. Adding authentication and authorization
2. Implementing more advanced filtering and sorting
3. Adding metrics and monitoring
4. Exploring GraphQL as an alternative to REST
5. Implementing WebSockets for real-time updates

For more examples, see the [GraphQL Example](../02_examples/graphql-example.md) or [Dependency Injection Example](../02_examples/dependency-injection-example.md).

### Task Service Implementation

Now let's implement the service layer that contains our business logic:

```rust
// src/services/task_service.rs
use crate::models::task::{CreateTaskRequest, Task, TaskStatus, UpdateTaskRequest};
use crate::repositories::task_repository::TaskRepository;
use crate::utils::error_handler::{ApiError, Result};
use navius::di::{AutoService, Inject, ServiceRegistry};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskServiceError {
    #[error("Task not found: {0}")]
    NotFound(String),
    
    #[error("Operation not permitted: {0}")]
    Forbidden(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl From<TaskServiceError> for ApiError {
    fn from(error: TaskServiceError) -> Self {
        match error {
            TaskServiceError::NotFound(msg) => ApiError::NotFound(msg),
            TaskServiceError::Forbidden(msg) => ApiError::Forbidden(msg),
            TaskServiceError::DatabaseError(msg) => ApiError::InternalError(msg),
            TaskServiceError::ValidationError(msg) => ApiError::BadRequest(msg),
        }
    }
}

pub trait TaskService: Send + Sync {
    fn get_user_tasks(
        &self, 
        user_id: &str, 
        status: Option<String>, 
        priority: Option<i32>, 
        tag: Option<String>
    ) -> Result<Vec<Task>>;
    
    fn get_task(&self, task_id: &str, user_id: &str) -> Result<Task>;
    fn create_task(&self, req: CreateTaskRequest, user_id: &str) -> Result<Task>;
    fn update_task(&self, task_id: &str, req: UpdateTaskRequest, user_id: &str) -> Result<Task>;
    fn delete_task(&self, task_id: &str, user_id: &str) -> Result<()>;
}

#[derive(AutoService)]
pub struct TaskServiceImpl {
    task_repository: Arc<dyn TaskRepository>,
}

impl TaskServiceImpl {
    pub fn new(task_repository: Arc<dyn TaskRepository>) -> Self {
        Self { task_repository }
    }
}

impl TaskService for TaskServiceImpl {
    fn get_user_tasks(
        &self, 
        user_id: &str, 
        status: Option<String>, 
        priority: Option<i32>, 
        tag: Option<String>
    ) -> Result<Vec<Task>> {
        // Convert status string to enum if provided
        let status_filter = match status {
            Some(status_str) => match status_str.as_str() {
                "pending" => Some(TaskStatus::Pending),
                "in_progress" => Some(TaskStatus::InProgress),
                "completed" => Some(TaskStatus::Completed),
                "archived" => Some(TaskStatus::Archived),
                _ => None,
            },
            None => None,
        };
        
        // Retrieve tasks with filters
        let tasks = self.task_repository.find_by_user(
            user_id, 
            status_filter, 
            priority, 
            tag.as_deref()
        )?;
        
        Ok(tasks)
    }
    
    fn get_task(&self, task_id: &str, user_id: &str) -> Result<Task> {
        // Get task by ID
        let task = self.task_repository.find_by_id(task_id)?
            .ok_or_else(|| TaskServiceError::NotFound(format!("Task with ID {} not found", task_id)))?;
            
        // Verify ownership
        if task.user_id != user_id {
            return Err(TaskServiceError::Forbidden("You don't have permission to access this task".to_string()).into());
        }
        
        Ok(task)
    }
    
    fn create_task(&self, req: CreateTaskRequest, user_id: &str) -> Result<Task> {
        // Create a new task from the request
        let mut task = Task::new(req.title, user_id.to_string());
        
        // Set optional fields
        task.description = req.description;
        task.due_date = req.due_date;
        
        if let Some(priority) = req.priority {
            task.priority = priority;
        }
        
        if let Some(tags) = req.tags {
            task.tags = tags;
        }
        
        // Save the task
        let created_task = self.task_repository.save(task)?;
        
        Ok(created_task)
    }
    
    fn update_task(&self, task_id: &str, req: UpdateTaskRequest, user_id: &str) -> Result<Task> {
        // Get existing task
        let mut task = self.get_task(task_id, user_id)?;
        
        // Update fields if provided
        if let Some(title) = req.title {
            task.title = title;
        }
        
        if let Some(description) = req.description {
            task.description = Some(description);
        }
        
        if let Some(status) = req.status {
            task.status = status;
        }
        
        if let Some(due_date) = req.due_date {
            task.due_date = Some(due_date);
        }
        
        if let Some(priority) = req.priority {
            task.priority = priority;
        }
        
        if let Some(tags) = req.tags {
            task.tags = tags;
        }
        
        // Update timestamp
        task.updated_at = chrono::Utc::now();
        
        // Save the updated task
        let updated_task = self.task_repository.save(task)?;
        
        Ok(updated_task)
    }
    
    fn delete_task(&self, task_id: &str, user_id: &str) -> Result<()> {
        // First verify that the task exists and belongs to the user
        self.get_task(task_id, user_id)?;
        
        // Delete the task
        self.task_repository.delete(task_id)?;
        
        Ok(())
    }
}

### Authentication Service Implementation

Now, let's implement the authentication service:

```rust
// src/services/auth_service.rs
use crate::models::user::{LoginRequest, RegisterRequest, TokenResponse, User, UserRole};
use crate::repositories::user_repository::UserRepository;
use crate::utils::error_handler::{ApiError, Result};
use crate::utils::middleware::AuthClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use navius::config::Config;
use navius::di::{AutoService, Inject, ServiceRegistry};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthServiceError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Token error: {0}")]
    TokenError(String),
}

impl From<AuthServiceError> for ApiError {
    fn from(error: AuthServiceError) -> Self {
        match error {
            AuthServiceError::AuthenticationFailed(msg) => ApiError::Unauthorized(msg),
            AuthServiceError::RegistrationFailed(msg) => ApiError::BadRequest(msg),
            AuthServiceError::DatabaseError(msg) => ApiError::InternalError(msg),
            AuthServiceError::TokenError(msg) => ApiError::InternalError(msg),
        }
    }
}

pub trait AuthService: Send + Sync {
    fn register(&self, req: RegisterRequest) -> Result<User>;
    fn login(&self, req: LoginRequest) -> Result<TokenResponse>;
}

#[derive(AutoService)]
pub struct AuthServiceImpl {
    user_repository: Arc<dyn UserRepository>,
    config: Arc<Config>,
}

impl AuthServiceImpl {
    pub fn new(user_repository: Arc<dyn UserRepository>, config: Arc<Config>) -> Self {
        Self { user_repository, config }
    }
    
    fn hash_password(&self, password: &str) -> String {
        // In a real application, use a proper password hashing algorithm
        // This is a simplistic example
        format!("hashed_{}", password)
    }
    
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        // In a real application, properly verify the password against the hash
        // This is a simplistic example
        hash == format!("hashed_{}", password)
    }
    
    fn generate_token(&self, user_id: &str, username: &str, role: &UserRole) -> Result<TokenResponse> {
        let secret = self.config.get_string("jwt.secret")
            .unwrap_or_else(|_| "default_very_secret_key".to_string());
        
        let expires_in = self.config.get_int("jwt.expires_in_seconds")
            .unwrap_or(3600); // Default: 1 hour
            
        let exp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::seconds(expires_in))
            .expect("Valid timestamp")
            .timestamp();
            
        let claims = AuthClaims {
            sub: user_id.to_string(),
            exp: exp as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            username: username.to_string(),
            role: format!("{:?}", role),
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| AuthServiceError::TokenError(format!("Token generation failed: {}", e)))?;
        
        Ok(TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }
}

impl AuthService for AuthServiceImpl {
    fn register(&self, req: RegisterRequest) -> Result<User> {
        // Check if username already exists
        if let Some(_) = self.user_repository.find_by_username(&req.username)? {
            return Err(AuthServiceError::RegistrationFailed(
                format!("Username '{}' is already taken", req.username)
            ).into());
        }
        
        // Check if email already exists
        if let Some(_) = self.user_repository.find_by_email(&req.email)? {
            return Err(AuthServiceError::RegistrationFailed(
                format!("Email '{}' is already registered", req.email)
            ).into());
        }
        
        // Hash password
        let password_hash = self.hash_password(&req.password);
        
        // Create user
        let mut user = User::new(req.username, req.email, password_hash);
        user.full_name = req.full_name;
        
        // Save user
        let created_user = self.user_repository.save(user)?;
        
        Ok(created_user)
    }
    
    fn login(&self, req: LoginRequest) -> Result<TokenResponse> {
        // Find user by username
        let user = self.user_repository.find_by_username(&req.username)?
            .ok_or_else(|| AuthServiceError::AuthenticationFailed(
                "Invalid username or password".to_string()
            ))?;
            
        // Verify password
        if !self.verify_password(&req.password, &user.password_hash) {
            return Err(AuthServiceError::AuthenticationFailed(
                "Invalid username or password".to_string()
            ).into());
        }
        
        // Generate token
        let token = self.generate_token(&user.id, &user.username, &user.role)?;
        
        Ok(token)
    }
}

### Implement Error Handling

Let's create a centralized error handling system:

```rust
// src/utils/error_handler.rs
use navius::http::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::ValidationErrors;

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Validation error")]
    ValidationError(ValidationErrors),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::JsonError(_) => StatusCode::BAD_REQUEST,
        }
    }
    
    pub fn to_response(&self) -> Response {
        let status = self.status_code();
        
        let details = match self {
            ApiError::ValidationError(errors) => {
                // Transform validation errors into a more readable format
                let error_map = errors.field_errors();
                let details_map: serde_json::Value = serde_json::to_value(error_map).unwrap_or_default();
                Some(details_map)
            },
            _ => None,
        };
        
        let body = ErrorResponse {
            status: status.as_u16().to_string(),
            message: self.to_string(),
            details,
        };
        
        match serde_json::to_string(&body) {
            Ok(json) => Response::builder()
                .status(status)
                .header("Content-Type", "application/json")
                .body(json.into())
                .unwrap_or_else(|_| Response::empty(StatusCode::INTERNAL_SERVER_ERROR)),
            Err(_) => Response::empty(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

impl From<ApiError> for Response {
    fn from(error: ApiError) -> Self {
        error.to_response()
    }
}

### Implement Authentication Middleware

Let's add middleware for JWT authentication:

```rust
// src/utils/middleware.rs
use crate::utils::error_handler::{ApiError, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
use navius::http::{Body, HeaderMap, Request, Response};
use navius::router::Middleware;
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String,         // User ID
    pub exp: usize,          // Expiration time
    pub iat: usize,          // Issued at
    pub username: String,    // Username
    pub role: String,        // User role
}

#[derive(Clone)]
pub struct AuthMiddleware {
    jwt_secret: String,
}

impl AuthMiddleware {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }
    
    fn extract_token(&self, headers: &HeaderMap) -> Option<String> {
        headers.get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|auth_header| {
                if auth_header.starts_with("Bearer ") {
                    Some(auth_header[7..].to_string())
                } else {
                    None
                }
            })
    }
}

impl Middleware for AuthMiddleware {
    fn process(&self, mut req: Request<Body>) -> Ready<Result<Request<Body>, Response>> {
        // Extract token from headers
        let token = match self.extract_token(req.headers()) {
            Some(token) => token,
            None => return ready(Err(ApiError::Unauthorized("No authorization token provided".to_string()).into())),
        };
        
        // Decode and validate token
        let claims = match decode::<AuthClaims>(
            &token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(token_data) => token_data.claims,
            Err(e) => return ready(Err(ApiError::Unauthorized(format!("Invalid token: {}", e)).into())),
        };
        
        // Add claims to request extensions for later use
        req.extensions_mut().insert(claims);
        
        ready(Ok(req))
    }
}

### Main Application 

Finally, let's put everything together in our main application:

```rust
// src/main.rs
mod api;
mod models;
mod repositories;
mod services;
mod utils;

use crate::api::auth_controller::AuthController;
use crate::api::task_controller::TaskController;
use crate::repositories::task_repository::InMemoryTaskRepository;
use crate::repositories::user_repository::InMemoryUserRepository;
use crate::services::auth_service::AuthServiceImpl;
use crate::services::task_service::TaskServiceImpl;
use crate::utils::middleware::AuthMiddleware;
use navius::app::{NaviusApp, AppConfig};
use navius::config::Config;
use navius::di::ServiceRegistry;
use navius::env::{Environment, EnvironmentConfig};
use navius::logger;
use navius::router::Router;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Navius application
    let mut app = NaviusApp::new(AppConfig {
        name: "rest-api-example".to_string(),
        environment: Environment::load(EnvironmentConfig::default()),
        ..Default::default()
    });
    
    // Configure services
    app.configure_services(|registry| {
        // Register repositories
        let task_repo = Arc::new(InMemoryTaskRepository::new());
        let user_repo = Arc::new(InMemoryUserRepository::new());
        
        registry.register::<dyn crate::repositories::task_repository::TaskRepository>(task_repo);
        registry.register::<dyn crate::repositories::user_repository::UserRepository>(user_repo);
        
        // Register services
        let config = registry.resolve::<Config>().unwrap();
        
        let task_service = Arc::new(TaskServiceImpl::new(
            registry.resolve::<dyn crate::repositories::task_repository::TaskRepository>().unwrap()
        ));
        
        let auth_service = Arc::new(AuthServiceImpl::new(
            registry.resolve::<dyn crate::repositories::user_repository::UserRepository>().unwrap(),
            config.clone()
        ));
        
        registry.register::<dyn crate::services::task_service::TaskService>(task_service);
        registry.register::<dyn crate::services::auth_service::AuthService>(auth_service);
    });
    
    // Configure routing
    app.configure_router(|router| {
        // Get the JWT secret from config
        let jwt_secret = router.service_registry()
            .resolve::<Config>()
            .unwrap()
            .get_string("jwt.secret")
            .unwrap_or_else(|_| "default_very_secret_key".to_string());
        
        // Create auth middleware
        let auth_middleware = AuthMiddleware::new(jwt_secret);
        
        // Set up controllers
        let task_service = router.service_registry()
            .resolve::<dyn crate::services::task_service::TaskService>()
            .unwrap();
            
        let auth_service = router.service_registry()
            .resolve::<dyn crate::services::auth_service::AuthService>()
            .unwrap();
        
        let task_controller = TaskController::new(task_service);
        let auth_controller = AuthController::new(auth_service);
        
        // Register public routes (no auth required)
        for route in auth_controller.register_routes() {
            router.add_route(route);
        }
        
        // Register protected routes (auth required)
        for route in task_controller.register_routes() {
            router.add_route_with_middleware(route, auth_middleware.clone());
        }
        
        // Add a health check endpoint
        router.get("/api/health", |_| {
            Response::json(&serde_json::json!({
                "status": "ok",
                "version": env!("CARGO_PKG_VERSION"),
            })).unwrap()
        });
    });
    
    // Start the application
    logger::info!("Starting REST API example application");
    app.start().await?;
    
    Ok(())
}
```

### Configuration

Let's create our configuration file:

```yaml
# config/default.yaml
server:
  host: "127.0.0.1"
  port: 8080
  
logging:
  level: info
  format: json
  
jwt:
  secret: "your-secret-key-here-make-it-long-and-secure"
  expires_in_seconds: 3600 # 1 hour
  
cors:
  allowed_origins: ["*"]
  allowed_methods: ["GET", "POST", "PUT", "DELETE"]
  max_age_secs: 86400
```

## Testing Your API

### Unit Testing

Let's write some unit tests for our service layer:

```rust
// tests/service_tests.rs
#[cfg(test)]
mod tests {
    use crate::models::task::{CreateTaskRequest, Task, TaskStatus};
    use crate::repositories::task_repository::MockTaskRepository;
    use crate::services::task_service::{TaskService, TaskServiceImpl};
    use std::sync::Arc;

    #[test]
    fn test_create_task() {
        // Arrange
        let mut mock_repo = MockTaskRepository::new();
        
        // Set up expectations
        mock_repo.expect_save()
            .times(1)
            .returning(|task| {
                Ok(task)
            });
            
        let service = TaskServiceImpl::new(Arc::new(mock_repo));
        
        let request = CreateTaskRequest {
            title: "Test Task".to_string(),
            description: Some("Test Description".to_string()),
            due_date: None,
            priority: Some(1),
            tags: Some(vec!["test".to_string()]),
        };
        
        // Act
        let result = service.create_task(request, "user123");
        
        // Assert
        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.user_id, "user123");
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_get_task_not_found() {
        // Arrange
        let mut mock_repo = MockTaskRepository::new();
        
        // Set up expectations
        mock_repo.expect_find_by_id()
            .times(1)
            .returning(|_| {
                Ok(None)
            });
            
        let service = TaskServiceImpl::new(Arc::new(mock_repo));
        
        // Act
        let result = service.get_task("nonexistent", "user123");
        
        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("not found"));
    }
}
```

### Integration Testing

We can also write integration tests for our API endpoints:

```rust
// tests/api_tests.rs
#[cfg(test)]
mod tests {
    use crate::api::task_controller::TaskController;
    use crate::models::task::{CreateTaskRequest, TaskResponse};
    use crate::utils::middleware::AuthClaims;
    use navius::http::{Body, Request, StatusCode};
    use navius::router::RouteHandler;
    use navius::test::TestClient;
    use std::sync::Arc;

    fn setup_test_client() -> TestClient {
        // Create a test client with mock services
        // ...
        TestClient::new()
    }

    #[test]
    fn test_create_task_endpoint() {
        // Arrange
        let client = setup_test_client();
        
        let auth_claims = AuthClaims {
            sub: "user123".to_string(),
            exp: 0,
            iat: 0,
            username: "testuser".to_string(),
            role: "User".to_string(),
        };
        
        let request = CreateTaskRequest {
            title: "Test Task".to_string(),
            description: Some("Test Description".to_string()),
            due_date: None,
            priority: Some(1),
            tags: Some(vec!["test".to_string()]),
        };
        
        // Act
        let response = client
            .post("/api/tasks")
            .with_extension(auth_claims)
            .json(&request)
            .send();
            
        // Assert
        assert_eq!(response.status(), StatusCode::CREATED);
        
        let task: TaskResponse = response.json();
        assert_eq!(task.title, "Test Task");
    }

    #[test]
    fn test_get_tasks_endpoint() {
        // Arrange
        let client = setup_test_client();
        
        let auth_claims = AuthClaims {
            sub: "user123".to_string(),
            exp: 0,
            iat: 0,
            username: "testuser".to_string(),
            role: "User".to_string(),
        };
        
        // Act
        let response = client
            .get("/api/tasks")
            .with_extension(auth_claims)
            .send();
            
        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        
        let tasks: Vec<TaskResponse> = response.json();
        assert!(!tasks.is_empty());
    }
}
```

## API Documentation

### Swagger/OpenAPI Documentation

To document your API, you can use Swagger/OpenAPI:

```rust
// src/utils/swagger.rs
use navius::router::Router;
use navius::http::{Response, StatusCode};
use std::fs;

pub fn add_swagger_routes(router: &mut Router) {
    // Serve the Swagger UI
    router.get("/api/docs", |_| {
        let html = include_str!("../../assets/swagger-ui.html");
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(html.into())
            .unwrap()
    });
    
    // Serve the OpenAPI specification
    router.get("/api/docs/openapi.json", |_| {
        let spec = include_str!("../../assets/openapi.json");
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(spec.into())
            .unwrap()
    });
}
```

## Best Practices

When building REST APIs with Navius, follow these best practices:

1. **Use RESTful Naming Conventions**:
   - Use nouns for resources (e.g., `/tasks`, not `/getTask`)
   - Use plural forms for collection endpoints
   - Use HTTP methods appropriately (GET, POST, PUT, DELETE)

2. **Version Your API**:
   - Include version in the URL (e.g., `/api/v1/tasks`)
   - Or use an Accept header (`Accept: application/vnd.company.v1+json`)

3. **Use Proper Status Codes**:
   - 200: OK
   - 201: Created
   - 204: No Content
   - 400: Bad Request
   - 401: Unauthorized
   - 403: Forbidden
   - 404: Not Found
   - 500: Internal Server Error

4. **Implement Authentication and Authorization**:
   - Use JWT tokens for stateless authentication
   - Implement role-based access control

5. **Handle Errors Consistently**:
   - Return detailed error messages
   - Include error codes for better client handling
   - Log errors server-side

6. **Validate Input**:
   - Always validate and sanitize user input
   - Return clear validation errors

7. **Implement Rate Limiting**:
   - Protect your API from abuse
   - Return 429 Too Many Requests when limits are exceeded

8. **Use HTTPS**:
   - Always use HTTPS in production
   - Redirect HTTP to HTTPS

9. **Document Your API**:
   - Use OpenAPI/Swagger for documentation
   - Include examples in your documentation

10. **Implement Pagination for Collections**:
    - Use limit and offset or page and per_page parameters
    - Return total count and pagination links

## Advanced Techniques

### Caching

Implement caching to improve API performance:

```rust
// src/utils/cache_middleware.rs
use navius::http::{Body, Request, Response, StatusCode};
use navius::router::Middleware;
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct CacheEntry {
    response: Response,
    expires_at: Instant,
}

pub struct CacheMiddleware {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    ttl: Duration,
}

impl CacheMiddleware {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }
    
    fn cache_key(&self, req: &Request<Body>) -> String {
        format!("{}{}", req.method(), req.uri())
    }
}

impl Middleware for CacheMiddleware {
    fn process(&self, req: Request<Body>) -> Ready<Result<Request<Body>, Response>> {
        // Only cache GET requests
        if req.method() != "GET" {
            return ready(Ok(req));
        }
        
        let key = self.cache_key(&req);
        
        // Check if cached response exists and is valid
        if let Ok(mut cache) = self.cache.lock() {
            if let Some(entry) = cache.get(&key) {
                if entry.expires_at > Instant::now() {
                    // Return cached response
                    return ready(Err(entry.response.clone()));
                } else {
                    // Remove expired entry
                    cache.remove(&key);
                }
            }
        }
        
        // Continue with request processing
        ready(Ok(req))
    }
    
    fn post_process(&self, req: &Request<Body>, mut res: Response) -> Response {
        // Only cache successful GET requests
        if req.method() == "GET" && res.status().is_success() {
            let key = self.cache_key(req);
            
            // Clone the response for caching
            let cache_res = res.clone();
            
            // Add to cache
            if let Ok(mut cache) = self.cache.lock() {
                cache.insert(key, CacheEntry {
                    response: cache_res,
                    expires_at: Instant::now() + self.ttl,
                });
            }
            
            // Add cache headers to response
            res.headers_mut().insert("Cache-Control", format!("max-age={}", self.ttl.as_secs()).parse().unwrap());
        }
        
        res
    }
}
```

### API Versioning

Implement API versioning to support multiple API versions:

```rust
// src/utils/versioning.rs
use navius::router::{Router, RouteHandler};

pub struct VersionedRouter {
    routers: Vec<(String, Router)>,
}

impl VersionedRouter {
    pub fn new() -> Self {
        Self {
            routers: Vec::new(),
        }
    }
    
    pub fn version(&mut self, version: &str) -> &mut Router {
        let version_str = version.to_string();
        
        if let Some(index) = self.routers.iter().position(|(v, _)| v == &version_str) {
            &mut self.routers[index].1
        } else {
            self.routers.push((version_str, Router::new()));
            let index = self.routers.len() - 1;
            &mut self.routers[index].1
        }
    }
    
    pub fn build(self) -> Router {
        let mut main_router = Router::new();
        
        for (version, router) in self.routers {
            for (path, handler) in router.routes() {
                let versioned_path = format!("/api/{}{}", version, path);
                main_router.add_route(RouteHandler::new_with_path(versioned_path, handler.handler()));
            }
        }
        
        main_router
    }
}
```

## Troubleshooting

### Common Issues and Solutions

1. **401 Unauthorized Errors**:
   - Check if the JWT token is valid and not expired
   - Ensure the token is properly formatted in the Authorization header
   - Verify that the JWT secret matches between token generation and validation

2. **404 Not Found Errors**:
   - Verify that the route is registered correctly
   - Check for typos in the URL path
   - Ensure that resource IDs are in the expected format

3. **500 Internal Server Errors**:
   - Look at server logs for detailed error information
   - Check for unhandled exceptions in your code
   - Verify that all dependencies are available

4. **Slow API Responses**:
   - Implement caching for frequently accessed resources
   - Check for N+1 query problems in your data access layer
   - Consider adding database indexes for frequently queried fields

5. **Validation Errors**:
   - Use the validator crate to implement input validation
   - Return clear error messages about validation failures
   - Include examples of valid requests in your API documentation