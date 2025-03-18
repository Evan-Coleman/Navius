# API Resource Abstraction

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
    // Define the fetch function inline to avoid lifetime issues
    let fetch_fn = move |state: &Arc<AppState>, id: i64| -> futures::future::BoxFuture<'static, Result<User>> {
        let state = state.clone(); // Clone the state to avoid lifetime issues
        Box::pin(async move {
            // Your actual API call logic here
            // ...
        })
    };
    
    // Create an API handler with reliability features
    let handler = create_api_handler(
        fetch_fn,
        ApiHandlerOptions {
            use_cache: true,
            use_retries: true,
            max_retry_attempts: 3,
            cache_ttl_seconds: 300,
            detailed_logging: true,
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
    use_cache: bool,              // Whether to use caching
    use_retries: bool,            // Whether to retry failed requests
    max_retry_attempts: u32,      // Maximum number of retry attempts (default: 3)
    cache_ttl_seconds: u64,       // Cache time-to-live in seconds (default: 300)
    detailed_logging: bool,       // Whether to log detailed information (default: true)
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
            max_retry_attempts: 3,
            cache_ttl_seconds: 300,
            detailed_logging: true,
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
            max_retry_attempts: 1,
            cache_ttl_seconds: 60, // Weather data changes frequently
            detailed_logging: false, // High volume endpoint, reduce logging
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

## Extending the Abstraction

This section explains how to extend the API resource abstraction to support new resource types beyond the existing ones.

### Current Limitations

The current implementation has specialized type conversions for certain resource types, but it's designed to be extended.

### Adding Support for a New Resource Type

#### 1. Identify Your Resource Type

For example, let's say you want to add support for a new `Product` type:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    id: i64,
    name: String,
    price: f64,
}
```

#### 2. Implement the ApiResource Trait

```rust
impl ApiResource for Product {
    type Id = i64;
    
    fn resource_type() -> &'static str {
        "product"
    }
    
    fn api_name() -> &'static str {
        "ProductAPI"
    }
}
```

#### 3. Update the Type Conversions

Modify the type conversion functions in `src/utils/api_resource/core.rs`:

```rust
fn convert_cached_resource<R: ApiResource>(cached: impl Any) -> Option<R> {
    // existing code for other types...
    
    // Handle Product resources
    else if type_id == std::any::TypeId::of::<Product>() {
        if let Some(product) = cached.downcast_ref::<Product>() {
            let boxed: Box<dyn Any> = Box::new(product.clone());
            let resource_any: Box<dyn Any> = boxed;
            if let Ok(typed) = resource_any.downcast::<R>() {
                return Some(*typed);
            }
        }
    }
    
    None
}
```

#### 4. Update the Cache Type if Needed

Depending on your needs, you may need to update the cache structure to handle multiple resource types.

## Future Enhancements

Planned enhancements to the pattern include:

1. Generic cache implementation that can work with any resource type
2. Circuit breaker pattern for automatically handling failing services
3. Integration with distributed tracing
4. Dynamic configuration of retry and caching policies 