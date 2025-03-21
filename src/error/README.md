# Application Error Handling

This directory provides a user-friendly interface to the application's error handling system. It serves as a wrapper around the core error handling system located in `src/core/error`.

## Usage

To use the error handling system in your application code:

```rust
use crate::error::{AppError, Result};

// Return the pre-defined Result type which is Result<T, AppError>
fn my_function() -> Result<String> {
    // Use ? to propagate errors
    let data = some_operation()?;
    
    // Or convert errors explicitly
    let value = external_call().map_err(|e| AppError::external(e))?;
    
    Ok(value)
}

// Create custom errors when needed
fn process_data(input: &str) -> Result<()> {
    if input.is_empty() {
        return Err(AppError::validation("Input cannot be empty"));
    }
    
    if input.len() > 100 {
        return Err(AppError::validation("Input too long, max 100 characters"));
    }
    
    Ok(())
}
```

## Common Error Types

The `AppError` enum provides several common error types:

- `AppError::not_found()` - Resource not found (404)
- `AppError::validation()` - Validation error (400)
- `AppError::unauthorized()` - Unauthorized access (401)
- `AppError::forbidden()` - Forbidden access (403)
- `AppError::internal()` - Internal server error (500)
- `AppError::external()` - Error from external service (503)
- `AppError::conflict()` - Conflict with existing data (409)

## Adding Custom Error Types

If you need to add custom error types, you can extend the AppError type in this directory:

```rust
// In src/error/error_types.rs
impl AppError {
    // Add a custom error constructor
    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::RateLimited,
            message: message.into(),
            status_code: StatusCode::TOO_MANY_REQUESTS,
            ..Default::default()
        }
    }
}

// Add a new error kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    // ... existing kinds ...
    RateLimited,
}
```

## Error Middleware

The error handling system automatically converts errors to HTTP responses with appropriate status codes. This is done through the `error_handler` middleware, which is applied to all routes.

You don't need to manually convert errors to responses - just return a `Result<T, AppError>` from your handlers and the middleware will handle the rest. 