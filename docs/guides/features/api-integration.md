---
title: Navius API Integration Guide
description: Techniques for integrating external APIs into Navius applications
category: guides
tags:
  - api
  - integration
  - external-services
  - client
related:
  - api-design.md
  - ../development/testing.md
  - ../../reference/standards/error-handling.md
last_updated: March 23, 2025
version: 1.0
---

# Navius API Integration Guide

## Overview
This guide explains how to integrate external APIs into your Navius application using the built-in API resource abstraction. It covers client generation, resilience patterns, caching strategies, and testing approaches to create reliable, high-performance API integrations.

## Prerequisites
Before using this guide, ensure you have:

- Basic understanding of HTTP client/server communication
- Familiarity with async programming in Rust
- Knowledge of API concepts (endpoints, status codes, etc.)
- Navius development environment set up

## Key Features

Navius makes it easy to integrate with external APIs by providing:

- 🔄 **Automatic client generation** from OpenAPI schemas
- 🛡️ **Built-in resilience patterns** for reliable API calls
- 💾 **Intelligent caching** to reduce load on downstream APIs
- 🔍 **Type-safe data transformation** using Rust's powerful type system
- 📊 **Detailed metrics and logging** for API calls

## Step-by-step API Integration

### 1. Adding an API Integration

#### Automated Method (Recommended)

The easiest way to add a new API integration is to use the provided script:

```bash
./scripts/add_api.sh <api_name> <api_url> <schema_url> [endpoint_path] [param_name]
```

For example:

```bash
./scripts/add_api.sh petstore https://petstore.swagger.io/v2 https://petstore.swagger.io/v2/swagger.json pet id
```

This will:
1. Generate API client code from the OpenAPI schema
2. Create handler functions for the specified endpoint
3. Configure routes for the new API
4. Add the API to the registry

#### Manual Method

If you prefer to add an API integration manually:

1. **Create an API client**:

```rust
pub struct PetstoreClient {
    base_url: String,
    http_client: Client,
}

impl PetstoreClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            http_client: Client::new(),
        }
    }
    
    pub async fn get_pet(&self, id: i64) -> Result<Pet, ApiError> {
        let url = format!("{}/pet/{}", self.base_url, id);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::RequestFailed(e.to_string()))?;
            
        if !response.status().is_success() {
            return Err(ApiError::ResponseError(
                response.status().as_u16(),
                format!("API returned error: {}", response.status()),
            ));
        }
        
        response
            .json::<Pet>()
            .await
            .map_err(|e| ApiError::DeserializationError(e.to_string()))
    }
}
```

2. **Create models**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    pub id: i64,
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Category>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}
```

3. **Implement API resource trait**:

```rust
use navius::core::api::{ApiResource, ApiError};

impl ApiResource for Pet {
    type Id = i64;
    
    fn resource_type() -> &'static str {
        "pet"
    }
    
    fn api_name() -> &'static str {
        "Petstore"
    }
}
```

4. **Create handler functions**:

```rust
use navius::core::api::{create_api_handler, ApiHandlerOptions};

pub async fn get_pet_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Pet>, AppError> {
    // Create API handler with reliability features
    let handler = create_api_handler(
        |state, id| async move {
            let client = &state.petstore_client;
            client.get_pet(id).await
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

5. **Add routes**:

```rust
// In your router setup
let api_routes = Router::new()
    .route("/pet/:id", get(get_pet_handler));
```

### 2. API Resource Abstractions

The `ApiResource` trait provides the foundation for the API abstraction:

```rust
pub trait ApiResource: Sized + Send + Sync + 'static {
    type Id: Display + Eq + Hash + Clone + Send + Sync + 'static;
    
    fn resource_type() -> &'static str;
    fn api_name() -> &'static str;
}
```

This allows the framework to automatically provide:
- Consistent caching of API responses
- Unified error handling
- Standardized logging patterns
- Metrics collection for API calls
- Retry logic with backoff

### 3. Implementing Reliability Patterns

Navius implements several reliability patterns for API integrations:

#### Caching

API responses are automatically cached using the configured cache implementation:

```rust
// Configure caching options
ApiHandlerOptions {
    use_cache: true,
    cache_ttl_seconds: 300, // Cache for 5 minutes
    // ...
}
```

#### Retry Logic

Failed API calls can be automatically retried with exponential backoff:

```rust
// Configure retry options
ApiHandlerOptions {
    use_retries: true,
    max_retry_attempts: 3,
    // ...
}
```

#### Circuit Breaking

Navius implements circuit breaking to prevent cascading failures:

```rust
// Enable circuit breaking
let circuit_breaker = CircuitBreaker::new(
    "petstore",
    CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 2,
        timeout_ms: 1000,
        half_open_timeout_ms: 5000,
    },
);

// Apply to client
let client = PetstoreClient::new("https://petstore.swagger.io/v2")
    .with_circuit_breaker(circuit_breaker);
```

### 4. Error Handling

Navius provides a standardized error handling pattern for API calls:

```rust
// In your AppError implementation
#[derive(Debug, Error)]
pub enum AppError {
    #[error("API Error: {0}")]
    Api(#[from] ApiError),
    // Other error types...
}

// ApiError is provided by the framework
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
    
    #[error("Circuit open")]
    CircuitOpen,
    
    #[error("Request timeout")]
    Timeout,
}
```

## Advanced Configuration

### Manual Cache Control

You can manually control caching for specific scenarios:

```rust
// Force refresh from the source API
let handler = create_api_handler(
    |state, id| async move {
        let client = &state.petstore_client;
        client.get_pet(id).await
    },
    ApiHandlerOptions {
        use_cache: true,
        force_refresh: true, // Skip cache and update it
        cache_ttl_seconds: 300,
        // ...
    },
);
```

### Custom Cache Keys

For complex scenarios, you can provide custom cache key generation:

```rust
// Custom cache key generation
let handler = create_api_handler_with_options(
    |state, id| async move {
        let client = &state.petstore_client;
        client.get_pet(id).await
    },
    |id| format!("custom:pet:{}", id), // Custom cache key
    ApiHandlerOptions {
        use_cache: true,
        // ...
    },
);
```

### Custom Response Transformation

Transform API responses before returning them:

```rust
// Transform the API response
let handler = create_api_handler_with_transform(
    |state, id| async move {
        let client = &state.petstore_client;
        client.get_pet(id).await
    },
    |pet| {
        // Transform the pet before returning
        PetResponse {
            id: pet.id,
            name: pet.name,
            status: pet.status,
            // Additional transformations...
        }
    },
    ApiHandlerOptions {
        // ...
    },
);
```

## Testing API Integrations

Navius provides utilities for testing API integrations:

```rust
use navius::test::api::{MockApiClient, ResponseBuilder};

#[tokio::test]
async fn test_pet_handler() {
    // Create mock API client
    let mut mock_client = MockApiClient::new();
    
    // Configure mock response
    mock_client.expect_get_pet()
        .with(eq(1))
        .times(1)
        .returning(|_| {
            Ok(Pet {
                id: 1,
                name: "Rex".to_string(),
                status: "available".to_string(),
                category: None,
                tags: vec![],
            })
        });
    
    // Create test app with mock client
    let app = test::build_app().with_api_client(mock_client).await;
    
    // Test the handler
    let response = app
        .get("/pet/1")
        .send()
        .await;
        
    assert_eq!(response.status(), StatusCode::OK);
    
    let pet: Pet = response.json().await;
    assert_eq!(pet.id, 1);
    assert_eq!(pet.name, "Rex");
}
```

## Performance Considerations

When integrating APIs, consider these factors:

1. **Caching Strategy**: 
   - Choose appropriate TTL values based on data freshness requirements
   - Use shorter TTLs for frequently changing data
   - Use longer TTLs for relatively static data

2. **Batch Operations**: 
   - Use batch endpoints where available instead of multiple single-item calls
   - Implement request coalescing for multiple simultaneous requests

3. **Concurrent Requests**: 
   - Use `futures::future::join_all` for parallel API calls
   - Limit concurrency to avoid overwhelming downstream services

4. **Response Size**: 
   - Request only the fields you need if the API supports field filtering
   - Consider pagination for large data sets

5. **Timeouts**: 
   - Configure appropriate timeouts to prevent blocking application threads
   - Use shorter timeouts for user-facing requests
   - Use longer timeouts for background processing

## Troubleshooting

### Common Issues

1. **Connection Timeouts**: 
   - Verify network connectivity to the external API
   - Check if the API endpoint URL is correct
   - Ensure firewall rules allow outbound connections

2. **Authentication Failures**: 
   - Verify API keys or tokens are valid and not expired
   - Check if authentication headers are properly formatted
   - Ensure credentials have proper permissions

3. **Rate Limiting**: 
   - Implement backoff strategies for rate-limited APIs
   - Consider adding request throttling
   - Monitor rate limit headers in responses

4. **Data Format Issues**: 
   - Validate request/response schemas against API documentation
   - Use debug logging to inspect raw request/response bodies
   - Ensure proper content-type headers are set

## Related Documents

- [API Design Guide](api-design.md) - Designing APIs in Navius
- [Authentication Guide](authentication.md) - Handling authentication
- [Testing Guide](../development/testing.md) - Testing API integrations 