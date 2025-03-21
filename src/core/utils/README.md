# Core Utility Modules

This directory contains the core utility implementations that power the application's utility features. These utilities provide foundational functionality that can be extended by users in the `src/utils` directory.

## Modules

- `api_logger` - Core API logging functionality
- `api_resource` - API resource abstractions
- `openapi` - OpenAPI specification utilities

## API Resource

The API resource module provides a high-level abstraction for working with API resources, handling common concerns like:

- Caching
- Retries
- Error handling
- Logging
- Metrics

### Key Components

- `ApiResource` trait - A trait representing an API resource entity
- `ApiHandlerOptions` - Configuration options for API handlers
- `create_api_handler` - Factory function for creating API request handlers
- `ApiResourceRegistry` - Registry for tracking API resources

### Example Usage

```rust
// Create a handler for fetching a pet resource
let pet_handler = create_api_handler(
    fetch_pet_from_api,
    ApiHandlerOptions {
        use_cache: true,
        use_retries: true,
        max_retry_attempts: 3,
        cache_ttl_seconds: 300,
        detailed_logging: true,
    }
);

// Register the pet resource type in the registry
register_resource::<Pet>(&app_state, None)?;
```

## API Logger

The API logger module provides utilities for logging API operations, including:

- Request logging
- Response logging
- Error logging
- Cache operation logging

## OpenAPI Utilities

The OpenAPI module provides utilities for working with OpenAPI specifications, including:

- Serving OpenAPI spec files
- Validating OpenAPI specifications
- Reading and writing spec files 