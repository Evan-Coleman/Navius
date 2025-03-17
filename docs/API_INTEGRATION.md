# Adding a New API Endpoint with Downstream Integration

This guide explains how to add a new endpoint to the Rust Backend that consumes a downstream API and transforms its response.

## Overview

We'll implement a new endpoint that:
1. Accepts a user request
2. Makes a request to a downstream API
3. Transforms the response
4. Returns the transformed data to the user

For this example, we'll use the [JSONPlaceholder API](https://jsonplaceholder.typicode.com/), which is a free REST API for testing that provides a simple OpenAPI schema.

## Automated Script

The easiest way to add a new API integration is to use our automation script:

```bash
./scripts/add_api.sh <api_name> <api_url> <schema_url> [endpoint_path] [param_name]
```

For example, to add the JSONPlaceholder API:

```bash
./scripts/add_api.sh jsonplaceholder https://jsonplaceholder.typicode.com https://jsonplaceholder.typicode.com/swagger.json posts id
```

The script will:
1. Download the OpenAPI schema
2. Generate API client code
3. Create necessary model files
4. Create handler files
5. Update configuration files
6. Add the new route to the router

After running the script, you can build and test your new endpoint immediately.

## Manual Step-by-Step Integration Guide

If you prefer to add the API manually or need to customize the integration beyond what the script provides, follow these steps:

### 1. Generate API Client from OpenAPI Schema

First, we need to download the OpenAPI schema and generate a Rust client using the OpenAPI Generator.

```bash
# Create a directory for the new API
mkdir -p src/openapi/jsonplaceholder

# Download the OpenAPI schema
curl -o src/openapi/jsonplaceholder/swagger.json https://jsonplaceholder.typicode.com/swagger.json

# Create a configuration file for the OpenAPI Generator
cat > src/openapi/jsonplaceholder/config.yaml << EOF
generatorName: rust
outputDir: ./src/jsonplaceholder_api
additionalProperties:
  packageName: jsonplaceholder
  serverFramework: axum
EOF

# Create the generation script
cat > src/openapi/jsonplaceholder/generate-api.sh << EOF
#!/bin/bash

set -e

# Clean up previous generated files if they exist
if [ -d "./src/jsonplaceholder_api" ]; then
    echo "Cleaning up previous generated files..."
    rm -rf ./src/jsonplaceholder_api
fi

# Run OpenAPI Generator
echo "Running OpenAPI Generator..."
openapi-generator-cli generate -i ./src/openapi/jsonplaceholder/swagger.json -c ./src/openapi/jsonplaceholder/config.yaml

# Create a module file
echo "Creating module declaration file..."
cat > ./src/jsonplaceholder_api/mod.rs << EOF2
pub mod models;
EOF2

echo "API generation complete."
EOF

# Make the script executable
chmod +x src/openapi/jsonplaceholder/generate-api.sh

# Run the script to generate the API client
./src/openapi/jsonplaceholder/generate-api.sh
```

### 2. Update Module Structure

Add the new API module to `src/lib.rs`:

```rust
// ... existing code ...

/// JSONPlaceholder API client
pub mod jsonplaceholder_api;

// ... existing code ...
```

### 3. Create Models for the New Endpoint

Create a file for our custom JSONPlaceholder models:

```rust
// src/models/jsonplaceholder.rs
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Enhanced post model with additional metadata
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EnhancedPost {
    /// Post ID
    #[schema(example = 1)]
    pub id: i32,
    
    /// Post title
    #[schema(example = "Enhanced: sunt aut facere repellat provident")]
    pub title: String,
    
    /// Post body with word count and reading time
    #[schema(example = "Body text with 120 words. Reading time: 2 minutes.")]
    pub enhanced_body: String,
    
    /// User ID who created the post
    #[schema(example = 1)]
    pub user_id: i32,
    
    /// Word count in the post body
    #[schema(example = 120)]
    pub word_count: usize,
    
    /// Estimated reading time in minutes
    #[schema(example = 2)]
    pub reading_time_minutes: usize,
    
    /// Dominant sentiment in the post (positive, negative, neutral)
    #[schema(example = "neutral")]
    pub sentiment: String,
}
```

Update `src/models/mod.rs` to include the new module:

```rust
pub mod schemas;
pub mod jsonplaceholder;

// Re-export commonly used models
pub use schemas::{
    // ... existing exports ...
};
pub use jsonplaceholder::EnhancedPost;
```

### 4. Create a Handler for the New Endpoint

Create a new handler file:

```rust
// src/handlers/posts.rs
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use tracing::{info, warn};

use crate::app::AppState;
use crate::models::jsonplaceholder::EnhancedPost;

/// Handler for the enhanced posts endpoint
#[utoipa::path(
    get,
    path = "/posts/{id}/enhanced",
    params(
        ("id" = i32, Path, description = "Post ID to fetch and enhance")
    ),
    responses(
        (status = 200, description = "Enhanced post retrieved successfully", body = EnhancedPost),
        (status = 404, description = "Post not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "posts"
)]
pub async fn get_enhanced_post(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<EnhancedPost>, (axum::http::StatusCode, String)> {
    info!("Fetching and enhancing post with ID: {}", id);

    // Construct the URL to fetch a post by ID
    let url = format!("{}/posts/{}", state.config.api.jsonplaceholder_url, id);
    
    // Make request to JSONPlaceholder API
    let response = state
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            warn!("Failed to fetch post: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch post: {}", e),
            )
        })?;

    // Check if response is successful
    if response.status().is_success() {
        // Parse the JSONPlaceholder post
        let post: jsonplaceholder_api::models::Post = response.json().await.map_err(|e| {
            warn!("Failed to parse post: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse post: {}", e),
            )
        })?;

        // Transform the post data
        let body = post.body.as_deref().unwrap_or_default();
        let words = body.split_whitespace().count();
        let reading_time = (words as f64 / 200.0).ceil() as usize;  // Assuming 200 words per minute
        
        // Simple sentiment analysis (very basic)
        let sentiment = if body.contains("great") || body.contains("good") || body.contains("happy") {
            "positive"
        } else if body.contains("bad") || body.contains("terrible") || body.contains("sad") {
            "negative"
        } else {
            "neutral"
        };
        
        // Create enhanced post
        let enhanced_post = EnhancedPost {
            id: post.id.unwrap_or_default(),
            title: post.title.unwrap_or_default(),
            enhanced_body: format!("{}. Word count: {}. Reading time: {} minute(s).", 
                              body, words, reading_time),
            user_id: post.user_id.unwrap_or_default(),
            word_count: words,
            reading_time_minutes: reading_time,
            sentiment: sentiment.to_string(),
        };
        
        info!("Successfully enhanced post ID: {}", id);
        Ok(Json(enhanced_post))
    } else if response.status() == axum::http::StatusCode::NOT_FOUND {
        warn!("Post with ID {} not found", id);
        Err((axum::http::StatusCode::NOT_FOUND, format!("Post with ID {} not found", id)))
    } else {
        warn!("API returned error status: {}", response.status());
        Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("API returned error status: {}", response.status()),
        ))
    }
}
```

Update `src/handlers/mod.rs` to include the new module:

```rust
pub mod data;
pub mod health;
pub mod metrics;
pub mod pet;
pub mod posts;  // Add this line

// Re-export handlers
pub use data::get_data;
pub use health::health_check;
pub use metrics::metrics;
pub use pet::get_pet_by_id;
pub use posts::get_enhanced_post;  // Add this line
```

### 5. Update Configuration

Update `src/config/app_config.rs` to add the JSONPlaceholder URL:

```rust
/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub cat_fact_url: String,
    pub petstore_url: String,
    pub jsonplaceholder_url: String,  // Add this line
    pub api_key: Option<String>,
}
```

Update `config/default.yaml` to include the JSONPlaceholder URL:

```yaml
# ... existing config ...

api:
  cat_fact_url: "https://catfact.ninja/fact"
  petstore_url: "https://petstore3.swagger.io/api/v3"
  jsonplaceholder_url: "https://jsonplaceholder.typicode.com"  # Add this line
  api_key: null

# ... rest of config ...
```

### 6. Update Router

Update `src/app/router.rs` to add the new route:

In the `ApiDoc` struct, add the new path and schema:

```rust
/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Pet Store API",
        version = "1.0.0",
        description = "A sample Pet Store server with OpenAPI model integration"
    ),
    paths(
        crate::handlers::health::health_check,
        crate::handlers::metrics::metrics,
        crate::handlers::data::get_data,
        crate::handlers::pet::get_pet_by_id,
        crate::handlers::posts::get_enhanced_post,  // Add this line
    ),
    components(
        schemas(
            HealthCheckResponse,
            MetricsResponse,
            Data,
            ApiResponse,
            PetSchema,
            CategorySchema,
            TagSchema,
            StatusSchema,
            crate::cache::CacheStats,
            crate::models::jsonplaceholder::EnhancedPost,  // Add this line
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "metrics", description = "Prometheus metrics endpoints"),
        (name = "data", description = "Data endpoints"),
        (name = "pets", description = "Pet endpoints"),
        (name = "posts", description = "Post endpoints"),  // Add this line
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;
```

And in the `create_router` function, add the new route:

```rust
/// Create the application router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/metrics", get(handlers::metrics::metrics))
        .route("/data", get(handlers::data::get_data))
        .route("/pet/{id}", get(handlers::pet::get_pet_by_id))
        .route("/posts/{id}/enhanced", get(handlers::posts::get_enhanced_post))  // Add this line
        .with_state(state)
}
```

### 7. Test the New Endpoint

Once you've implemented these changes, rebuild and run the application:

```bash
cargo build
./run_server.sh
```

The `run_server.sh` script preserves your manual API registry settings by default.

Test the new endpoint with curl:

```bash
curl http://localhost:3000/posts/1/enhanced
```

You should receive an enhanced post with additional metadata.

### 8. Add Unit Tests (Optional)

Create a test file for the new handler:

```rust
// src/handlers/posts_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Path;
    use axum::Json;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_get_enhanced_post() {
        // Mock implementation would go here
    }
}
```

## Adding a Different API

To integrate a different API, follow these same steps but modify them for your specific API:

1. Find the OpenAPI schema for your API
2. Generate a Rust client from the schema
3. Create models for your enhanced data
4. Create handlers for the new endpoints
5. Update configuration to include the new API URL
6. Update the router to add the new routes
7. Test the new endpoints

## Tips and Best Practices

1. **Error Handling**: Always implement proper error handling and return appropriate HTTP status codes.

2. **Rate Limiting**: Consider implementing rate limiting for API calls to avoid hitting external API limits.

3. **Caching**: Use the existing cache mechanism for responses from external APIs to reduce load and improve performance.

4. **Authentication**: If the external API requires authentication, store API keys securely in environment variables.

5. **Retries**: Implement a retry mechanism for failed requests to improve reliability.

6. **Validation**: Validate input parameters before making downstream API calls.

7. **Documentation**: Keep your OpenAPI documentation up-to-date with all endpoints.

8. **Testing**: Write unit and integration tests for new endpoints.

9. **Monitoring**: Add appropriate metrics tracking for the new endpoints.

## Resources

- [JSONPlaceholder API Documentation](https://jsonplaceholder.typicode.com/)
- [OpenAPI Generator Documentation](https://openapi-generator.tech/)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Utoipa Documentation](https://docs.rs/utoipa/latest/utoipa/) 