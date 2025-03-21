# Core Error Handling

This directory contains the core error handling system used by the application. It provides a unified approach to error handling, logging, and middleware for the entire application.

## Components

- `error_types.rs`: Core error types (AppError) and Result type
- `logger.rs`: Error logging utilities 
- `middleware.rs`: Error handling middleware for Axum
- `result_ext.rs`: Extensions to Result for more convenient error handling
- `mod.rs`: Module definitions and exports

## Design

The error handling system is designed to:

1. Provide a consistent error handling approach throughout the application
2. Convert different error types into a unified format
3. Log errors with appropriate context and details
4. Automatically convert errors into HTTP responses with appropriate status codes
5. Support custom error extensions when needed

## Key Features

- **Unified Error Type**: The `AppError` type represents all errors in the application
- **Context-Aware Logging**: Errors are logged with request context when available
- **HTTP Mapping**: Errors automatically map to appropriate HTTP response codes
- **Result Extensions**: Utility methods to make working with Results more convenient
- **Error Middleware**: Automatic conversion of errors to HTTP responses

## Usage

The core error handling is not meant to be used directly by application code. Instead, use the application-level error module in `src/error`, which provides a more user-friendly interface.

If you need to interact with the core error system directly, use the following approach:

```rust
use crate::core::error::{AppError, Result};

fn some_function() -> Result<T> {
    // Use ? to propagate errors
    let data = some_operation()?;
    
    // Or convert errors explicitly
    let value = external_call().map_err(|e| AppError::external(e))?;
    
    Ok(value)
} 