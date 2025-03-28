---
title: "API Resource Abstraction Pattern"
description: "Documentation about API Resource Abstraction Pattern"
category: reference
tags:
  - api
  - caching
last_updated: March 23, 2025
version: 1.0
---
# API Resource Abstraction Pattern

This document explains the API resource abstraction pattern used in our project, which provides a unified way to handle API resources with built-in reliability features.

## Overview

The API resource abstraction provides a clean, consistent pattern for handling external API interactions with the following features:

- **Automatic caching**: Resources are cached to reduce latency and external API calls
- **Retry mechanism**: Failed API calls are retried with exponential backoff
- **Consistent error handling**: All API errors are handled in a consistent way
- **Standardized logging**: API interactions are logged with consistent format
- **Type safety**: Strong typing ensures correctness at compile time

## Core Components

The abstraction consists of the following components:

1. **ApiResource trait**: Interface that resources must implement
2. **ApiHandlerOptions**: Configuration options for handlers
3. **create_api_handler**: Factory function to create Axum handlers with reliability features
4. **Support functions**: Caching and retry helpers

## Using the Pattern

### 1. Implementing ApiResource for your model

```rust
use crate::utils::api_resource::ApiResource;

// Your model structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: i64,
    name: String,
    email: String,
}

// Implement ApiResource for your model
impl ApiResource for User {
    type Id = i64;  // The type of the ID field

    fn resource_type() -> &'static str {
        "user"  // Used for caching and logging
    }

    fn api_name() -> &'static str {
        "UserService"  // Used for logging
    }
}
```

### 2. Creating a Fetch Function

```rust
async fn fetch_user(state: &Arc<AppState>, id: i64) -> Result<User> {
    let url = format!("{}/users/{}", state.config.user_service_url, id);
    
    // Create a closure that returns the actual request future
    let fetch_fn = || async { state.client.get(&url).send().await };
    
    // Make the API call using the common logger/handler
    api_logger::api_call("UserService", &url, fetch_fn, "User", id).await
}
```

### 3. Creating an API Handler

```rust
pub async fn get_user_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<User>> {
    // Create an API handler with reliability features
    let handler = create_api_handler(
        fetch_user,
        ApiHandlerOptions {
            use_cache: true,
            use_retries: true,
        },
    );
    
    // Execute the handler
    handler(State(state), Path(id)).await
}
```

## Configuration Options

The `ApiHandlerOptions` struct provides the following configuration options:

```rust
struct ApiHandlerOptions {
    use_cache: bool,   // Whether to use caching
    use_retries: bool, // Whether to retry failed requests
}
```

## Best Practices

1. **Keep fetch functions simple**: They should focus on the API call logic
2. **Use consistent naming**: Name conventions help with maintenance
3. **Add appropriate logging**: Additional context helps with debugging
4. **Handle errors gracefully**: Return appropriate error codes to clients
5. **Test thoroughly**: Verify behavior with unit tests for each handler

## Example Use Cases

### Basic Handler with Default Options

```rust
pub async fn get_product_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Product>> {
    create_api_handler(
        fetch_product,
        ApiHandlerOptions {
            use_cache: true,
            use_retries: true,
        },
    )(State(state), Path(id)).await
}
```

### Custom Handler with Specific Options

```rust
pub async fn get_weather_handler(
    State(state): State<Arc<AppState>>,
    Path(location): Path<String>,
) -> Result<Json<Weather>> {
    create_api_handler(
        fetch_weather,
        ApiHandlerOptions {
            use_cache: true,     // Weather data can be cached
            use_retries: false,  // Weather requests shouldn't retry
        },
    )(State(state), Path(location)).await
}
```

## Troubleshooting

### Cache Not Working

If caching isn't working as expected:

1. Verify the `use_cache` option is set to `true`
2. Ensure the `ApiResource` implementation is correct
3. Check if the cache is enabled in the application state

### Retries Not Working

If retries aren't working as expected:

1. Verify the `use_retries` option is set to `true`
2. Check the error type (only service errors are retried)
3. Inspect the logs for retry attempts

## Future Enhancements

Planned enhancements to the pattern include:

1. Configurable retry policies (max attempts, backoff strategy)
2. Cache TTL options per resource type
3. Circuit breaker pattern for failing services 

## Related Documents
- [API Standards](../standards/api-standards.md) - API design guidelines
- [Error Handling](../standards/error-handling.md) - Error handling patterns

