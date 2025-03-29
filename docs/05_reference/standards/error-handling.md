---
title: Navius Error Handling Reference
description: Standard error handling patterns and practices for Navius applications
category: reference
tags:
  - errors
  - exceptions
  - standards
  - best-practices
related:
  - ../architecture/principles.md
  - ../../guides/features/api-design.md
  - ../../guides/features/api-integration.md
last_updated: March 27, 2025
version: 1.0
---

# Navius Error Handling Reference

## Overview
This reference document details the standard error handling patterns used throughout the Navius framework. Consistent error handling is essential for creating maintainable and reliable applications. This guide covers error types, propagation strategies, and integration with the HTTP layer.

## Core Error Types

Navius organizes errors into several layers of abstraction:

### Application Errors (`AppError`)

The top-level error type represents application-level errors that can be directly mapped to HTTP responses:

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,
    
    #[error("Not authorized to access this resource")]
    Unauthorized,
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Validation error")]
    Validation(#[from] ValidationError),
    
    #[error("Database error")]
    Database(#[from] DatabaseError),
    
    #[error("API error")]
    Api(#[from] ApiError),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}
```

### Domain-Specific Errors

Module-specific errors that represent failures in particular domains:

```rust
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    ConnectionFailed(String),
    
    #[error("Query error: {0}")]
    QueryFailed(String),
    
    #[error("Resource not found")]
    NotFound,
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    #[error("Transaction error: {0}")]
    TransactionFailed(String),
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    
    #[error("Response error ({0}): {1}")]
    ResponseError(u16, String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Resource not found")]
    NotFound,
    
    #[error("API timeout")]
    Timeout,
}
```

### Validation Errors

Structured validation errors for request validation:

```rust
#[derive(Debug, Error)]
pub struct ValidationError {
    #[error("Validation failed: {0} errors")]
    pub errors: Vec<FieldError>,
}

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub code: String,
    pub message: String,
}
```

## Error Conversion

Errors are automatically converted between layers using the `From` trait:

```rust
impl From<DatabaseError> for AppError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::NotFound => AppError::NotFound,
            DatabaseError::ConnectionFailed(msg) => AppError::Internal(format!("Database connection failed: {}", msg)),
            DatabaseError::QueryFailed(msg) => AppError::Internal(format!("Database query failed: {}", msg)),
            DatabaseError::ConstraintViolation(msg) => AppError::BadRequest(format!("Constraint violation: {}", msg)),
            DatabaseError::TransactionFailed(msg) => AppError::Internal(format!("Transaction failed: {}", msg)),
        }
    }
}

impl From<ApiError> for AppError {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::NotFound => AppError::NotFound,
            ApiError::RequestFailed(msg) => AppError::Internal(format!("API request failed: {}", msg)),
            ApiError::ResponseError(status, msg) if status == 401 => AppError::Unauthorized,
            ApiError::ResponseError(status, msg) if status == 403 => AppError::Forbidden(msg),
            ApiError::ResponseError(_, msg) => AppError::Internal(format!("API response error: {}", msg)),
            ApiError::DeserializationError(msg) => AppError::Internal(format!("API deserialization error: {}", msg)),
            ApiError::Timeout => AppError::Internal("API request timed out".to_string()),
        }
    }
}
```

## HTTP Response Integration

Errors are converted to HTTP responses using the `IntoResponse` trait:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_response) = match &self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND, 
                ErrorResponse::new("not_found", "Resource not found"),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED, 
                ErrorResponse::new("unauthorized", "Not authorized to access this resource"),
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN, 
                ErrorResponse::new("forbidden", msg),
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST, 
                ErrorResponse::new("bad_request", msg),
            ),
            AppError::Validation(validation_error) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error_type: "validation_error".to_string(),
                    message: "Validation failed".to_string(),
                    details: Some(validation_error.errors.clone()),
                },
            ),
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("database_error", "A database error occurred"),
            ),
            AppError::Api(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("api_error", "An API error occurred"),
            ),
            AppError::Internal(msg) => {
                // Log internal error details but don't expose them in response
                tracing::error!(error = %msg, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::new("internal_error", "An internal server error occurred"),
                )
            },
        };
        
        // Create the HTTP response
        (status, Json(error_response)).into_response()
    }
}
```

## Error Response Format

Navius uses a consistent JSON error response format:

```json
{
  "error": {
    "type": "validation_error",
    "message": "Validation failed",
    "details": [
      {
        "field": "email",
        "code": "invalid_format",
        "message": "Must be a valid email address"
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

This is implemented using the `ErrorResponse` struct:

```rust
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
}

impl ErrorResponse {
    pub fn new(error_type: &str, message: &str) -> Self {
        Self {
            error: ErrorDetail {
                error_type: error_type.to_string(),
                message: message.to_string(),
                details: None,
            },
        }
    }
    
    pub fn with_details(error_type: &str, message: &str, details: Vec<FieldError>) -> Self {
        Self {
            error: ErrorDetail {
                error_type: error_type.to_string(),
                message: message.to_string(),
                details: Some(details),
            },
        }
    }
}
```

## Error Propagation Patterns

### Result Type

All functions that can fail return a `Result` type:

```rust
pub async fn get_user(id: Uuid) -> Result<User, AppError> {
    let user = repository.find_by_id(id).await?;
    
    match user {
        Some(user) => Ok(user),
        None => Err(AppError::NotFound),
    }
}
```

### Using the `?` Operator

The `?` operator is used for clean error propagation:

```rust
pub async fn update_user(id: Uuid, data: UpdateUser) -> Result<User, AppError> {
    // Validate the request
    data.validate()?;
    
    // Get the user
    let user = repository.find_by_id(id).await?;
    
    match user {
        Some(mut user) => {
            // Update the user
            user.name = data.name;
            user.email = data.email;
            
            // Save the user
            repository.update(&user).await?;
            
            Ok(user)
        },
        None => Err(AppError::NotFound),
    }
}
```

### Context for Error Enrichment

For adding context to errors, use the `context` method from the `anyhow` crate:

```rust
use anyhow::Context;

pub async fn process_payment(payment_id: Uuid) -> Result<Payment, AppError> {
    let payment = repository
        .find_by_id(payment_id)
        .await
        .context("Failed to find payment")?;
    
    let result = payment_service
        .process(payment)
        .await
        .context("Failed to process payment")?;
    
    Ok(result)
}
```

## Logging Errors

Errors are logged using the `tracing` crate:

```rust
match repository.find_by_id(id).await {
    Ok(Some(user)) => Ok(user),
    Ok(None) => {
        tracing::debug!(user_id = %id, "User not found");
        Err(AppError::NotFound)
    },
    Err(err) => {
        tracing::error!(
            error = %err,
            user_id = %id,
            "Failed to find user"
        );
        Err(AppError::Database(err))
    }
}
```

## Error Handling in Controllers

Controller functions are concise, focusing on converting domain results to HTTP responses:

```rust
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.user_service.get_user(id).await?;
    Ok(Json(user.into()))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    // Validate the request (validation error will be automatically converted)
    request.validate()?;
    
    // Create the user
    let user = state.user_service.create_user(request).await?;
    
    // Return 201 Created with the user
    Ok((StatusCode::CREATED, Json(user.into())))
}
```

## Testing Error Handling

Error handling should be thoroughly tested:

```rust
#[tokio::test]
async fn test_get_user_not_found() {
    // Create mock repository
    let mut mock_repository = MockUserRepository::new();
    
    // Set expectation: user not found
    mock_repository
        .expect_find_by_id()
        .with(eq(user_id))
        .times(1)
        .returning(|_| Ok(None));
    
    // Create service with mock
    let service = UserService::new(mock_repository);
    
    // Execute test
    let result = service.get_user(user_id).await;
    
    // Verify result is NotFound error
    assert!(matches!(result, Err(AppError::NotFound)));
}

#[tokio::test]
async fn test_create_user_validation_error() {
    // Create app for testing
    let app = test::build_app().await;
    
    // Send invalid request
    let response = app
        .post("/api/users")
        .json(&json!({
            "email": "not-an-email",
            "password": "123"  // too short
        }))
        .send()
        .await;
    
    // Verify status code
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Verify error response format
    let error: ErrorResponse = response.json().await;
    assert_eq!(error.error.error_type, "validation_error");
    assert!(error.error.details.is_some());
    
    // Verify validation details
    let details = error.error.details.unwrap();
    assert_eq!(details.len(), 2);  // Two validation errors
}
```

## Best Practices

### 1. Be Specific About Errors

Use specific error types rather than generic errors:

```rust
// Good
fn validate_user(user: &User) -> Result<(), ValidationError> {
    // Implementation
}

// Avoid
fn validate_user(user: &User) -> Result<(), Box<dyn Error>> {
    // Implementation
}
```

### 2. Don't Expose Internal Errors

Never expose internal error details to clients:

```rust
// Good
match db_error {
    DatabaseError::NotFound => AppError::NotFound,
    err => {
        tracing::error!(error = %err, "Database error");
        AppError::Internal("A database error occurred".to_string())
    }
}

// Avoid
AppError::Internal(format!("SQL error: {}", db_error))
```

### 3. Add Context to Errors

Always add context to errors for debugging:

```rust
// Good
let user = repository
    .find_by_id(id)
    .await
    .context("Failed to find user")?;

// Avoid
let user = repository.find_by_id(id).await?;
```

### 4. Validate Early

Validate input at the earliest possible point:

```rust
// Good
pub async fn create_user(request: CreateUserRequest) -> Result<User, AppError> {
    // Validate first
    request.validate()?;
    
    // Then proceed with business logic
    // ...
}
```

## Related Documents

- [Architecture Principles](../architecture/principles.md) - Core architectural principles
- [API Design Guide](../../guides/features/api-design.md) - Designing APIs
- [API Integration Guide](../../guides/features/api-integration.md) - Integrating with external APIs 
