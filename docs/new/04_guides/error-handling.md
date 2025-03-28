# Error Handling

This guide explains Navius's approach to error handling, covering error types, propagation, and best practices for dealing with errors in your application.

## Error Philosophy

Navius follows these principles for error handling:

1. **Type-safe errors**: Errors are strongly typed and categorized
2. **Contextual information**: Errors contain helpful context for debugging
3. **Clean propagation**: Errors flow naturally through the application
4. **User-friendly messages**: End users receive appropriate error information
5. **Consistent handling**: Common error handling patterns are established

## Core Error Types

### AppError

The central error type in Navius is `AppError`, which categorizes errors by their nature:

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(ValidationErrors),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
}
```

### Domain-Specific Errors

Each domain in your application can define its own error types that map to `AppError`:

```rust
#[derive(Debug, Error)]
pub enum UserServiceError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Invalid user data: {0}")]
    InvalidUserData(String),
    
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
}

impl From<UserServiceError> for AppError {
    fn from(err: UserServiceError) -> Self {
        match err {
            UserServiceError::UserNotFound(msg) => AppError::NotFound(msg),
            UserServiceError::InvalidUserData(msg) => AppError::InvalidInput(msg),
            UserServiceError::UserAlreadyExists(msg) => AppError::Conflict(msg),
            UserServiceError::DatabaseError(err) => AppError::DatabaseError(err),
        }
    }
}
```

## Error Context

Navius promotes adding context to errors:

```rust
use navius::core::error::{Context, ResultExt};

fn get_user(id: &str) -> Result<User, AppError> {
    let user = database.find_user(id)
        .context(format!("Failed to find user with ID {}", id))?;
    
    Ok(user)
}
```

The `context` method adds additional information to the error without changing its type.

## Error Mapping

To convert between error types, use the `map_err` method:

```rust
fn validate_and_save_user(user_data: UserInput) -> Result<User, AppError> {
    let validated_user = validate_user(user_data)
        .map_err(|e| AppError::ValidationError(e))?;
    
    database.save_user(validated_user)
        .map_err(|e| AppError::DatabaseError(e))?;
        
    Ok(validated_user)
}
```

## HTTP Error Responses

Navius automatically converts `AppError` to HTTP responses:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::ValidationError(errors) => {
                let error_message = format!("Validation error: {:?}", errors);
                (StatusCode::BAD_REQUEST, error_message)
            },
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };
        
        let body = Json(json!({
            "error": {
                "message": error_message,
                "code": status.as_u16(),
            }
        }));
        
        (status, body).into_response()
    }
}
```

This allows handlers to return errors directly:

```rust
async fn get_user_handler(
    Path(user_id): Path<String>,
    Extension(service): Extension<Arc<UserService>>,
) -> Result<Json<User>, AppError> {
    let user = service.get_user(&user_id)?;
    Ok(Json(user))
}
```

## Error Logging

Navius integrates error logging with the error handling system:

```rust
// Middleware that logs errors
async fn error_logging_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Response {
    let response = next.run(req).await;
    
    if response.status().is_server_error() {
        if let Some(error) = response.extensions().get::<AppError>() {
            log::error!("Server error: {:?}", error);
        }
    }
    
    response
}
```

## Validation Errors

For input validation, Navius uses a structured approach:

```rust
use validator::{Validate, ValidationErrors};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserInput {
    #[validate(length(min = 3, max = 50))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8))]
    pub password: String,
}

impl CreateUserInput {
    pub fn validate(&self) -> Result<(), AppError> {
        match self.validate() {
            Ok(_) => Ok(()),
            Err(e) => Err(AppError::ValidationError(e)),
        }
    }
}

// Handler with validation
async fn create_user_handler(
    Json(input): Json<CreateUserInput>,
    Extension(service): Extension<Arc<UserService>>,
) -> Result<Json<User>, AppError> {
    // Validate input
    input.validate()?;
    
    // Process validated input
    let user = service.create_user(input)?;
    Ok(Json(user))
}
```

## Error Handling in Services

Services should use their domain-specific error types:

```rust
pub struct UserService {
    database: Arc<DatabaseConnection>,
}

impl UserService {
    pub fn get_user(&self, id: &str) -> Result<User, UserServiceError> {
        let user = self.database.find_user(id)
            .map_err(|e| match e {
                DatabaseError::NotFound => UserServiceError::UserNotFound(id.to_string()),
                _ => UserServiceError::DatabaseError(e),
            })?;
            
        Ok(user)
    }
}
```

## Error Recovery

For handling recoverable errors:

```rust
pub async fn get_user_with_fallback(
    user_id: &str,
    user_service: &UserService,
    cache_service: &CacheService,
) -> Result<User, AppError> {
    // Try to get from primary service
    match user_service.get_user(user_id) {
        Ok(user) => Ok(user),
        Err(e) => {
            // Log the error
            log::warn!("Primary user service failed: {:?}", e);
            
            // Try fallback
            match cache_service.get_user(user_id) {
                Ok(Some(user)) => {
                    log::info!("Retrieved user from cache fallback");
                    Ok(user)
                }
                _ => Err(e.into()),
            }
        }
    }
}
```

## Panics vs. Errors

Navius distinguishes between errors and panics:

- **Errors** are expected exceptional conditions that can be handled
- **Panics** are unexpected conditions that cannot be reasonably recovered from

```rust
// Example of error handling
fn divide(a: i32, b: i32) -> Result<i32, DivisionError> {
    if b == 0 {
        return Err(DivisionError::DivideByZero);
    }
    Ok(a / b)
}

// Example of when to panic
fn initialize_critical_system() {
    match load_critical_configuration() {
        Ok(config) => {
            // Continue initialization
        }
        Err(e) => {
            log::error!("Critical system initialization failed: {:?}", e);
            panic!("Cannot continue without critical system");
        }
    }
}
```

## Testing Error Handling

Testing error cases is essential:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_not_found() {
        let db = MockDatabase::new();
        db.expect_find_user()
            .with("unknown")
            .returns(Err(DatabaseError::NotFound));
            
        let service = UserService::new(Arc::new(db));
        
        let result = service.get_user("unknown");
        assert!(matches!(result, Err(UserServiceError::UserNotFound(_))));
    }
}
```

## Best Practices

1. **Use Domain-Specific Error Types**: Create error types that reflect your domain
2. **Add Context to Errors**: Include relevant information in error messages
3. **Implement From for Error Conversions**: Make error type conversion explicit
4. **Hide Internal Errors from Users**: Map internal errors to appropriate public errors
5. **Log Detailed Error Information**: Log the full error context for debugging
6. **Test Error Cases**: Write tests specifically for error scenarios
7. **Use the ? Operator**: Leverage Rust's ? for clean error propagation
8. **Return Early for Validation**: Check conditions early and return errors immediately
9. **Structured Error Responses**: Return consistent, structured error responses to clients

## Common Patterns

### The Result Combinator Pattern

```rust
fn process_data(input: &str) -> Result<Output, AppError> {
    input
        .parse::<InputData>()
        .map_err(|e| AppError::InvalidInput(format!("Invalid input: {}", e)))?
        .process()
        .map_err(|e| AppError::InternalServerError(format!("Processing error: {}", e)))?
        .finalize()
        .map_err(|e| AppError::InternalServerError(format!("Finalization error: {}", e)))
}
```

### The Try-Catch Pattern

```rust
fn try_multiple_approaches(input: &str) -> Result<Output, AppError> {
    // Try first approach
    let result = approach_one(input);
    
    // If it fails, try the second approach
    if let Err(e) = &result {
        log::info!("First approach failed: {:?}, trying second approach", e);
        return approach_two(input);
    }
    
    result
}
```

## Related Guides

- [Request Validation](request-validation.md) for validating API inputs
- [Logging](logging.md) for error logging strategies
- [Testing](testing.md) for testing error scenarios 