---
title: "Error Handling Example"
description: ""
category: "Documentation"
tags: []
last_updated: "March 28, 2025"
version: "1.0"
---

---
title: "Comprehensive Error Handling in Navius"
description: "A complete guide to implementing robust, consistent error handling in Navius applications with centralized error types, validation error handling, middleware integration, and client-friendly error responses"
category: examples
tags:
  - error-handling
  - validation
  - middleware
  - http-status-codes
  - api-responses
  - logging
  - thiserror
related:
  - 02_examples/rest-api-example.md
  - 02_examples/custom-service-example.md
  - 04_guides/api-design.md
last_updated: March 27, 2025
version: 1.1
status: stable
---

# Error Handling Example

This example demonstrates implementing robust error handling in Navius applications, focusing on creating a centralized error system with consistent error responses.

## Overview

Effective error handling is crucial for building reliable and user-friendly applications. This example shows how to:

- Create a unified error type system for your entire application
- Convert various error types to consistent HTTP responses
- Validate user input and provide helpful validation error messages
- Handle business logic errors, database errors, and external service failures
- Log errors appropriately while hiding sensitive details from users
- Integrate error handling with middleware for consistent application-wide behavior

By implementing these patterns, your application will provide clear feedback to users when errors occur, while maintaining security and making debugging easier for developers.

## Quick Navigation

- [Project Structure](#project-structure)
- [Implementation](#implementation)
  - [Error System](#srcerrorrs)
  - [Models](#srcmodelsrs)
  - [Handlers](#srchandlersrs)
  - [Application Setup](#srcmainrs)
- [Testing Error Scenarios](#testing-the-error-handling)
- [Key Concepts](#key-concepts)
- [Error Handling Best Practices](#best-practices)
- [Common Error Patterns](#common-error-patterns)
- [Error Response Structures](#error-response-structures)
- [Integrating with External Services](#integrating-with-external-services)
- [Error Handling in Async Code](#error-handling-in-async-code)
- [Security Considerations](#security-considerations)
- [Debugging Tips](#debugging-tips)

## Prerequisites

Before working with this example, you should be familiar with:

- Rust programming basics
- HTTP status codes and RESTful API concepts
- Basic understanding of error handling in Rust (Result, Error traits)

Required dependencies:
- Rust 1.70 or newer
- Navius 0.1.0 or newer
- tokio for asynchronous operations
- thiserror for defining error types
- validator for request validation
- serde for serialization/deserialization

## Project Structure

```
error-handling-example/
├── Cargo.toml                # Project dependencies
├── config/
│   └── default.yaml         # Configuration file
└── src/
    ├── main.rs              # Application entry point with error middleware
    ├── handlers.rs          # Example API handlers demonstrating various errors
    ├── models.rs            # Domain models with validation rules
    └── error.rs             # Centralized error handling system
```

## Implementation

### src/error.rs

```
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
            if let AppError::Database(db_error) = &error {
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
        if let AppError::Validation(validation_errors) = &error {
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

```
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

```
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

```
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

```
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

```
cargo run
```

### Testing Validation Errors

```
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "invalid-email",
    "name": "A",
    "password": "short"
  }'
```

Response:
```
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

```
curl http://localhost:8080/users/not-found
```

Response:
```
{
  "status": 404,
  "code": "NOT_FOUND",
  "message": "User with ID not-found not found"
}
```

### Testing Database Errors

```
curl http://localhost:8080/users/db-error
```

Response:
```
{
  "status": 500,
  "code": "DATABASE_ERROR",
  "message": "An internal server error occurred"
}
```

### Testing Authorization Errors

```
curl http://localhost:8080/admin
```

Response:
```
{
  "status": 403,
  "code": "FORBIDDEN",
  "message": "Admin access required"
}
```

### Testing External Service Errors

```
curl http://localhost:8080/external
```

Response:
```
{
  "status": 503,
  "code": "EXTERNAL_SERVICE_ERROR",
  "message": "Payment gateway is currently unavailable"
}
```

## Key Concepts

1. **Centralized Error Types**
   - A unified `AppError` enum represents all application errors
   - Each variant corresponds to a common error scenario (not found, unauthorized, etc.)
   - Factory methods make it easy to create errors with meaningful messages

2. **Consistent Error Responses**
   - All errors are converted to JSON with the same structure
   - Includes status code, error code, and human-readable message
   - Special handling for validation errors to include detailed field information

3. **Error Hiding for Security**
   - Internal server errors don't expose implementation details to clients
   - Details are logged server-side for debugging
   - User only sees a generic message for server errors

4. **Middleware Integration**
   - Error handling middleware processes all errors consistently
   - Logging middleware tracks error occurrences
   - Centralized handling ensures consistent behavior across the application

5. **Validation Framework**
   - Input validation using the validator crate
   - Extension trait converts validation errors to application errors
   - Both schema validation and business rule validation are demonstrated

## Best Practices

### Error Type Design

1. **Use Thiserror for Custom Errors**
   - The `thiserror` crate makes it easy to define error types with good formatting
   - Derive `Debug` and `Error` traits for all error types
   - Implement clear `Display` formatting for human-readable messages

2. **Hierarchical Error Types**
   - Use nested error types for different domains (e.g., `DatabaseError` inside `AppError`)
   - Implement `From` conversions to easily convert between error types
   - Group related errors as variants of a domain-specific enum

3. **Factory Methods**
   - Provide factory methods for creating common errors
   - Use descriptive names like `not_found` or `unauthorized`
   - Accept generic string-like parameters for flexible message formatting

### Response Formatting

1. **Consistent Error Format**
   - All error responses should follow the same JSON structure
   - Include a numeric status code, string error code, and message
   - Use meaningful error codes that help identify the error type

2. **Status Code Mapping**
   - Map each error type to the appropriate HTTP status code
   - Use standard codes (404 for not found, 400 for bad request, etc.)
   - Match the semantics of the error to the appropriate status code

3. **Detailed Validation Errors**
   - For validation failures, include field-specific error details
   - Provide the field name, error code, and a helpful message
   - Structure validation errors in a way that's easy for clients to process

### Security and Logging

1. **Hide Internal Details**
   - Don't expose internal error details in responses to clients
   - Hide database errors, stack traces, and implementation details
   - Log full error details server-side for debugging

2. **Appropriate Logging Levels**
   - Log server errors at ERROR level
   - Log client errors (400-level) at INFO or WARN level
   - Include relevant context (request path, method, etc.) in logs

3. **Error Tracing**
   - Consider adding request IDs to correlate errors with requests
   - Use structured logging to make errors searchable
   - In production, ensure errors are monitored and alerted on

## Common Error Patterns

### Input Validation

Input validation should happen as early as possible in request handling:

```
// Schema validation using validator crate
if let Err(validation_errors) = payload.validate() {
    return Err(validation_errors.to_app_error());
}

// Business rule validation
if !is_valid_business_rule(&payload) {
    return Err(AppError::validation(vec![
        ValidationError {
            field: "field_name".to_string(),
            code: "BUSINESS_RULE".to_string(),
            message: "Business rule violation description".to_string(),
        }
    ]));
}
```

### Resource Not Found

When a requested resource doesn't exist:

```
async fn get_resource(id: &str) -> Result<Resource, AppError> {
    match db.find_by_id(id).await {
        Some(resource) => Ok(resource),
        None => Err(AppError::not_found(format!("Resource with ID {} not found", id)))
    }
}
```

### Authorization Checks

When checking if a user has permission:

```
fn check_permission(user: &User, resource: &Resource) -> Result<(), AppError> {
    // Check if user owns the resource
    if resource.owner_id != user.id {
        return Err(AppError::forbidden("You don't have permission to access this resource"));
    }
    
    // Check if user has required role
    if !user.roles.contains(&"ADMIN".to_string()) {
        return Err(AppError::forbidden("This action requires admin privileges"));
    }
    
    Ok(())
}
```

### External Service Integration

When calling external services:

```
async fn call_payment_gateway(payment: &Payment) -> Result<PaymentResponse, AppError> {
    match payment_client.process_payment(payment).await {
        Ok(response) => Ok(response),
        Err(PaymentError::ServiceUnavailable(msg)) => {
            Err(AppError::external_service(format!("Payment gateway unavailable: {}", msg)))
        },
        Err(PaymentError::InvalidRequest(msg)) => {
            Err(AppError::bad_request(format!("Invalid payment request: {}", msg)))
        },
        Err(e) => {
            log::error!("Unexpected payment error: {:?}", e);
            Err(AppError::internal_server_error("Failed to process payment"))
        }
    }
}
```

## Error Response Structures

### Basic Error Response

```
{
  "status": 404,
  "code": "NOT_FOUND",
  "message": "User with ID 123 not found"
}
```

### Validation Error Response

```
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
        "field": "password",
        "code": "min_length",
        "message": "Password must be at least 8 characters"
      }
    ]
  }
}
```

### Server Error Response

Note how implementation details are hidden:

```
{
  "status": 500,
  "code": "DATABASE_ERROR",
  "message": "An internal server error occurred"
}
```

## Integrating with External Services

When integrating with external services, follow these patterns for error handling:

1. **Error Categorization**
   - Map external service errors to appropriate application error types
   - Distinguish between temporary failures and permanent errors
   - Use appropriate status codes (e.g., 503 for unavailable services)

2. **Retry Strategies**
   - Implement retries for transient errors
   - Use exponential backoff with jitter
   - Set appropriate timeouts

3. **Circuit Breaking**
   - Prevent cascading failures with circuit breakers
   - Fail fast when a service is known to be down
   - Provide fallback behavior when possible

Example integration:

```
async fn call_with_retry<F, T, E>(
    operation: F,
    max_retries: usize,
) -> Result<T, AppError> 
where
    F: Fn() -> Future<Output = Result<T, E>>,
    E: Into<AppError>,
{
    let mut retries = 0;
    let mut delay = 100; // ms
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                if retries >= max_retries {
                    return Err(err.into());
                }
                
                retries += 1;
                tokio::time::sleep(Duration::from_millis(delay)).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
}
```

## Error Handling in Async Code

Async error handling has some unique considerations:

1. **Error Propagation**
   - Use `?` operator to propagate errors in async functions
   - Remember that `.await` can produce errors that need to be handled

2. **Cancellation**
   - Handle task cancellation gracefully
   - Clean up resources even when tasks are cancelled

3. **Timeouts**
   - Add timeouts to all external operations
   - Convert timeout errors to appropriate application errors

Example with timeout:

```
async fn with_timeout<T>(
    future: impl Future<Output = Result<T, AppError>>,
    duration: Duration,
) -> Result<T, AppError> {
    match tokio::time::timeout(duration, future).await {
        Ok(result) => result,
        Err(_) => Err(AppError::external_service("Operation timed out")),
    }
}
```

## Security Considerations

1. **Information Disclosure**
   - Never expose stack traces, SQL queries, or internal paths in errors
   - Be careful with validation errors exposing system details
   - Sanitize error messages from third-party libraries

2. **Error Enumeration**
   - Consider if detailed validation errors could aid attackers
   - For security-sensitive fields, consider generic errors

3. **Logging Sensitive Data**
   - Don't log passwords, tokens, or personal data
   - Be careful what you include in error details
   - Use redaction in logs for sensitive fields

4. **Rate Limiting**
   - Apply rate limiting to error-prone endpoints
   - Prevent brute force attacks through repeated errors
   - Consider progressive delays for repeated failures

## Debugging Tips

1. **Error Context**
   - Add context to errors as they propagate up the stack
   - Consider using the `anyhow` crate for context in development
   - Include relevant operation parameters in error messages

2. **Error Tracing**
   - Use unique identifiers for request tracing
   - Include these identifiers in error responses
   - Log the full context with these identifiers

3. **Development vs. Production**
   - Consider more verbose errors in development
   - Use feature flags to control error detail level:

```
#[cfg(debug_assertions)]
fn format_error(err: &AppError) -> String {
    format!("Detailed error: {:?}", err)
}

#[cfg(not(debug_assertions))]
fn format_error(err: &AppError) -> String {
    format!("Error: {}", err)
}
```

## Next Steps

Now that you've implemented a robust error handling system, consider:

1. Exploring more advanced error handling with observability tools
2. Creating a client-side error handling library to interpret your errors
3. Implementing custom error serialization for different output formats (XML, etc.)
4. Adding internationalization (i18n) support for error messages

## Related Examples

For additional examples and error handling patterns, see:
- [REST API Example](rest-api-example.md) for a complete API implementation
- [Custom Service Example](custom-service-example.md) for service-level error handling
- [Authentication Example](authentication-example.md) for security-related error handling 
