---
title: "Error Handling Example"
description: "Implementing robust error handling in Navius applications"
category: examples
tags:
  - examples
  - error-handling
  - api
related:
  - examples/rest-api-example.md
  - reference/patterns/service-registration-pattern.md
last_updated: March 26, 2025
version: 1.0
---

# Error Handling Example

This example demonstrates implementing robust error handling in Navius applications, focusing on creating a centralized error system with consistent error responses.

## Project Structure

```
error-handling-example/
├── Cargo.toml
├── config/
│   └── default.yaml
└── src/
    ├── main.rs                 # Application entry point
    ├── handlers.rs             # Example API handlers
    ├── models.rs               # Domain models
    └── error.rs                # Error handling system
```

## Implementation

### src/error.rs

```rust
use navius::http::{Response, StatusCode};
use navius::http::header;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use thiserror::Error;

/// Represents all possible error types in the application
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Validation error")]
    Validation(Vec<ValidationError>),
    
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("External service error: {0}")]
    ExternalService(String),
}

/// Database-specific errors
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
}

/// Validation error for a specific field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }
    
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }
    
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }
    
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }
    
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }
    
    pub fn validation(errors: Vec<ValidationError>) -> Self {
        Self::Validation(errors)
    }
    
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::InternalServerError(message.into())
    }
    
    pub fn external_service(message: impl Into<String>) -> Self {
        Self::ExternalService(message.into())
    }
    
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ExternalService(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
    
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
            Self::Database(_) => "DATABASE_ERROR",
            Self::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
        }
    }
}

/// Convert the application error to an HTTP response
impl From<AppError> for Response {
    fn from(error: AppError) -> Self {
        let status = error.status_code();
        let should_hide_details = status.is_server_error();
        
        // Log server errors
        if should_hide_details {
            if let Self::Database(db_error) = &error {
                log::error!("Database error: {}", db_error);
            } else {
                log::error!("Server error: {}", error);
            }
        }
        
        // Prepare error response
        let mut body = json!({
            "status": status.as_u16(),
            "code": error.error_code(),
            "message": if should_hide_details {
                "An internal server error occurred".to_string()
            } else {
                error.to_string()
            }
        });
        
        // Add validation errors if present
        if let Self::Validation(validation_errors) = &error {
            body["details"] = json!({
                "errors": validation_errors
            });
        }
        
        // Create response
        let body_string = serde_json::to_string(&body).unwrap_or_else(|_| {
            r#"{"status":500,"code":"JSON_ERROR","message":"Failed to serialize error"}"#.to_string()
        });
        
        let mut response = Response::new(body_string.into());
        *response.status_mut() = status;
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        
        response
    }
}

/// Extension trait for validation errors
pub trait ValidatorErrorExt {
    fn to_app_error(&self) -> AppError;
}

impl ValidatorErrorExt for validator::ValidationErrors {
    fn to_app_error(&self) -> AppError {
        let mut errors = Vec::new();
        
        for (field, field_errors) in self.field_errors() {
            for error in field_errors {
                let code = error.code.clone();
                let message = error.message
                    .clone()
                    .unwrap_or_else(|| "Invalid value".into())
                    .to_string();
                
                errors.push(ValidationError {
                    field: field.to_string(),
                    code: code.to_string(),
                    message,
                });
            }
        }
        
        AppError::validation(errors)
    }
}
```

### src/models.rs

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    
    #[validate(length(min = 8, max = 100))]
    pub password: String,
}
```

### src/handlers.rs

```rust
use navius::http::StatusCode;
use navius::web::{Json, Path, State};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, DatabaseError, ValidatorErrorExt, ValidationError};
use crate::models::{CreateUserRequest, User};

// Example handler with validation
pub async fn create_user(
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    // Validate the request
    if let Err(validation_errors) = payload.validate() {
        return Err(validation_errors.to_app_error());
    }
    
    // Example of business logic validation
    if payload.email.ends_with("@example.com") {
        let error = ValidationError {
            field: "email".to_string(),
            code: "DOMAIN_NOT_ALLOWED".to_string(),
            message: "example.com emails are not allowed".to_string(),
        };
        return Err(AppError::validation(vec![error]));
    }
    
    // Create user (would normally talk to a database)
    let user = User {
        id: Uuid::new_v4().to_string(),
        email: payload.email,
        name: payload.name,
        roles: vec!["USER".to_string()],
    };
    
    // Return success response
    Ok((StatusCode::CREATED, Json(user)))
}

// Example handler demonstrating not found error
pub async fn get_user_by_id(
    Path(id): Path<String>,
) -> Result<Json<User>, AppError> {
    // Simulate a database lookup
    if id == "not-found" {
        return Err(AppError::not_found(format!("User with ID {} not found", id)));
    }
    
    // Simulate a database error
    if id == "db-error" {
        return Err(AppError::Database(DatabaseError::Query(
            "Error executing query: relation \"users\" does not exist".to_string()
        )));
    }
    
    // Success case
    let user = User {
        id,
        email: "user@example.com".to_string(),
        name: "Example User".to_string(),
        roles: vec!["USER".to_string()],
    };
    
    Ok(Json(user))
}

// Example handler demonstrating authorization errors
pub async fn admin_only() -> Result<Json<serde_json::Value>, AppError> {
    // Simulate an authorization check
    let is_admin = false;
    
    if !is_admin {
        return Err(AppError::forbidden("Admin access required"));
    }
    
    Ok(Json(json!({ "success": true })))
}

// Example handler demonstrating external service errors
pub async fn external_service() -> Result<Json<serde_json::Value>, AppError> {
    // Simulate an external service call
    let service_available = false;
    
    if !service_available {
        return Err(AppError::external_service("Payment gateway is currently unavailable"));
    }
    
    Ok(Json(json!({ "success": true })))
}
```

### src/main.rs

```rust
mod error;
mod handlers;
mod models;

use navius::http::StatusCode;
use navius::routing::{get, post, Router};
use navius::Application;
use navius::middleware;
use std::convert::Infallible;
use std::net::SocketAddr;

// Global error handler middleware
async fn handle_error(err: error::AppError) -> navius::http::Response {
    err.into()
}

// Demonstration middleware that logs errors
async fn log_errors<B>(
    req: navius::http::Request<B>,
    next: navius::middleware::Next<B>
) -> Result<navius::http::Response, Infallible> {
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    
    // Continue with the request
    let response = next.run(req).await;
    
    // Log any errors
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        log::error!("{} {} - Status: {}", method, path, status);
    }
    
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();
    
    // Create router with handlers
    let app = Router::new()
        .route("/users", post(handlers::create_user))
        .route("/users/:id", get(handlers::get_user_by_id))
        .route("/admin", get(handlers::admin_only))
        .route("/external", get(handlers::external_service))
        .layer(middleware::from_fn(log_errors))
        .layer(middleware::from_fn_with_state(handle_error));
    
    // Run the application
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Server running at http://{}", addr);
    
    navius::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
```

### Cargo.toml

```toml
[package]
name = "error-handling-example"
version = "0.1.0"
edition = "2021"

[dependencies]
navius = "0.1.0"
tokio = { version = "1.28", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
validator = { version = "0.16", features = ["derive"] }
log = "0.4"
env_logger = "0.10"
uuid = { version = "1.3", features = ["v4"] }
```

## Testing the Error Handling

### Running the Example

```bash
cargo run
```

### Testing Validation Errors

```bash
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "invalid-email",
    "name": "A",
    "password": "short"
  }'
```

Response:
```json
{
  "status": 400,
  "code": "VALIDATION_ERROR",
  "message": "Validation error",
  "details": {
    "errors": [
      {
        "field": "email",
        "code": "email",
        "message": "Invalid email format"
      },
      {
        "field": "name",
        "code": "length",
        "message": "Name must be at least 2 characters"
      },
      {
        "field": "password",
        "code": "length",
        "message": "Password must be at least 8 characters"
      }
    ]
  }
}
```

### Testing Not Found Errors

```bash
curl http://localhost:8080/users/not-found
```

Response:
```json
{
  "status": 404,
  "code": "NOT_FOUND",
  "message": "User with ID not-found not found"
}
```

### Testing Database Errors

```bash
curl http://localhost:8080/users/db-error
```

Response:
```json
{
  "status": 500,
  "code": "DATABASE_ERROR",
  "message": "An internal server error occurred"
}
```

### Testing Authorization Errors

```bash
curl http://localhost:8080/admin
```

Response:
```json
{
  "status": 403,
  "code": "FORBIDDEN",
  "message": "Admin access required"
}
```

### Testing External Service Errors

```bash
curl http://localhost:8080/external
```

Response:
```json
{
  "status": 503,
  "code": "EXTERNAL_SERVICE_ERROR",
  "message": "Payment gateway is currently unavailable"
}
```

## Key Concepts Demonstrated

1. **Centralized Error Handling**: A unified `AppError` type for representing all possible errors.
2. **Error Categories**: Different error types for various scenarios (validation, authorization, database, etc.).
3. **Consistent Responses**: All errors are converted to JSON responses with a consistent structure.
4. **Error Hiding**: Internal server errors don't expose implementation details to clients.
5. **Validation Errors**: Detailed validation feedback for invalid input data.
6. **Error Logging**: Server errors are automatically logged.
7. **HTTP Status Codes**: Appropriate status codes for different error types.

## Best Practices

1. **Use Thiserror**: The `thiserror` crate makes it easy to define custom error types.
2. **Consistent Error Format**: All error responses follow the same JSON structure.
3. **Hide Internal Details**: Don't expose internal error details in production.
4. **Appropriate Status Codes**: Use the right HTTP status code for each error type.
5. **Detailed Validation Errors**: For validation failures, return specific field errors.
6. **Error Conversion**: Implement `From<OtherError> for AppError` for all error types used in the application.
7. **Error Middleware**: Use middleware to handle error conversion and logging consistently.

## Related Documentation

- [REST API Example](./rest-api-example.md)
- [Service Registration Pattern](../reference/patterns/service-registration-pattern.md) 