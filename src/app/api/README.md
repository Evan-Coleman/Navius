# User-Facing API Endpoints

This directory contains user-facing API endpoints that can be extended or customized by application developers.

## Purpose

The `app/api` directory is intended for:

- Custom API endpoint implementations
- Extending core API functionality
- Creating application-specific endpoints

## Implementation Guidelines

When adding new API endpoints:

1. Create new files for logically grouped endpoints
2. Follow the routing pattern established in `app/router.rs`
3. Use proper error handling with the core error types
4. Document your endpoints with OpenAPI annotations

## Example

```rust
// src/app/api/users.rs
use axum::{
    routing::{get, post},
    Router,
};
use crate::core::auth::AuthLayer;
use crate::models::User;

pub fn routes() -> Router {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        .layer(AuthLayer::new())
}

async fn list_users() -> impl IntoResponse {
    // Implementation
}

// Other handler functions...
```

Then register these routes in `app/router.rs` to make them available. 