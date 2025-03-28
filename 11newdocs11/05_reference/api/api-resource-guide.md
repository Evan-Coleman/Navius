---
title: "Navius API Resource Abstraction"
description: "Documentation about Navius API Resource Abstraction"
category: reference
tags:
  - api
  - caching
  - development
  - documentation
  - integration
  - performance
last_updated: March 23, 2025
version: 1.0
---
# Navius API Resource Abstraction

## Overview

Navius includes a powerful API resource abstraction pattern for building reliable, efficient, and fault-tolerant API handlers. This pattern provides a standardized approach to building APIs that integrate with external services while maintaining high reliability and performance.

## Key Features

- **💾 Automatic caching** of API responses to reduce latency and external API calls
- **🔄 Retry mechanism** with exponential backoff for handling transient failures
- **🛡️ Circuit breaking** to prevent cascading failures when downstream services are degraded
- **⏱️ Timeout management** to ensure requests don't hang indefinitely
- **🔍 Consistent error handling** across all API endpoints
- **📝 Standardized logging** for API interactions
- **✅ Type safety** through Rust's type system

## Using the API Resource Abstraction

The API resource abstraction can be used to create robust handlers for your API endpoints that interact with external services.

### Step 1: Implement the `ApiResource` Trait

First, implement the `ApiResource` trait for your model:

```rust
use crate::core::api::ApiResource;

impl ApiResource for MyModel {
    type Id = i64;  // The type of your ID field
    
    fn resource_type() -> &'static str {
        "myresource"  // Used for cache keys and logging
    }
    
    fn api_name() -> &'static str {
        "MyService"  // Used for logging
    }
}
```

### Step 2: Create a Handler Function

Next, create a handler function using the abstraction:

```rust
use axum::{extract::Path, extract::State, Json};
use std::sync::Arc;
use crate::core::api::{create_api_handler, ApiHandlerOptions};
use crate::app::AppState;
use crate::models::MyModel;

pub async fn get_my_resource_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<MyModel>> {
    // Create an API handler with reliability features
    let handler = create_api_handler(
        |state, id| async move {
            // Your actual API call logic here
            // For example:
            let client = &state.api_client;
            let response = client.get(&format!("https://api.example.com/resources/{}", id))
                .send()
                .await?
                .json::<MyModel>()
                .await?;
            
            Ok(response)
        },
        ApiHandlerOptions {
            use_cache: true,
            use_retries: true,
            max_retry_attempts: 3,
            cache_ttl_seconds: 300,
            detailed_logging: true,
        },
    );
    
    handler(State(state), Path(id)).await
}
```

### Step 3: Register Your Handler

Register your handler function with a route:

```rust
use axum::{Router, routing::get};

pub fn my_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/resources/:id", get(get_my_resource_handler))
}
```

## API Handler Options

The `ApiHandlerOptions` struct allows you to customize the behavior of your API handlers:

| Option | Description |
|--------|-------------|
| `use_cache` | Enable/disable response caching |
| `cache_ttl_seconds` | Time-to-live for cache entries in seconds |
| `use_retries` | Enable/disable automatic retries |
| `max_retry_attempts` | Maximum number of retry attempts |
| `retry_initial_delay_ms` | Initial delay before first retry in milliseconds |
| `retry_backoff_factor` | Exponential backoff factor for retries |
| `detailed_logging` | Enable/disable detailed logging of API interactions |
| `timeout_seconds` | Request timeout in seconds |
| `circuit_breaker_enabled` | Enable/disable circuit breaker pattern |

## Advanced Usage

### Custom Cache Keys

You can customize how cache keys are generated:

```rust
let handler = create_api_handler_with_options(
    // API call function
    |state, id| async move { /* ... */ },
    
    // Standard options
    ApiHandlerOptions { /* ... */ },
    
    // Custom cache key function
    |id: &str| format!("custom-prefix:{}:suffix", id),
);
```

### Error Mapping

You can provide custom error mapping for specific error types:

```rust
let handler = create_api_handler_with_error_mapping(
    // API call function
    |state, id| async move { /* ... */ },
    
    // Standard options
    ApiHandlerOptions { /* ... */ },
    
    // Custom error mapping
    |err| match err {
        ApiError::NotFound => AppError::ResourceNotFound("The requested resource was not found".into()),
        ApiError::Timeout => AppError::ServiceUnavailable("The service is currently unavailable".into()),
        _ => AppError::InternalError("An internal error occurred".into()),
    },
);
```

## Performance Considerations

- **Cache sizing**: Configure appropriate cache sizes in your configuration to balance memory usage and performance
- **TTL values**: Set appropriate TTL values based on how frequently your data changes
- **Circuit breaker thresholds**: Tune circuit breaker thresholds based on your downstream services' behavior

## Example: Complete Implementation

Here's a complete example of implementing the API resource abstraction:

```rust
// Model definition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub description: String,
}

// Implement ApiResource trait
impl ApiResource for Product {
    type Id = i64;
    
    fn resource_type() -> &'static str {
        "product"
    }
    
    fn api_name() -> &'static str {
        "ProductCatalogAPI"
    }
}

// Handler implementation
pub async fn get_product_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Product>, AppError> {
    let handler = create_api_handler(
        |state, id| async move {
            let client = &state.product_client;
            let response = client.get_product(id).await?;
            Ok(response)
        },
        ApiHandlerOptions {
            use_cache: true,
            cache_ttl_seconds: 3600, // Products change infrequently
            use_retries: true,
            max_retry_attempts: 2,
            detailed_logging: true,
            timeout_seconds: 5,
            circuit_breaker_enabled: true,
        },
    );
    
    handler(State(state), Path(id)).await
}

// Router registration
pub fn product_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/products/:id", get(get_product_handler))
}
```

## Best Practices

1. **Cache wisely**: Only cache resources that change infrequently
2. **Set appropriate TTLs**: Balance freshness and performance
3. **Use retries judiciously**: Only retry idempotent operations
4. **Configure timeouts**: Set realistic timeouts based on downstream service performance
5. **Enable detailed logging in development**: But consider performance in production
6. **Use circuit breakers**: Prevent cascading failures when services are degraded

For more advanced usage scenarios, see the API Integration documentation and the full API Reference. 

## Related Documents
- [API Standards](../standards/api-standards.md) - API design guidelines
- [Error Handling](../standards/error-handling.md) - Error handling patterns

