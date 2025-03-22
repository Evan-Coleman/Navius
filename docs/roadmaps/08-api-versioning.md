# API Versioning Roadmap

## Overview
A pragmatic approach to API versioning for Navius, focusing on URL path-based versioning for simplicity and clarity.

## Current State
Our application requires a structured approach to API versioning to support evolution while maintaining backward compatibility.

## Target State
A focused API versioning system that:
- Uses URL path-based versioning for clarity and simplicity
- Securely maintains compatibility with existing clients
- Integrates cleanly with Axum routing
- Provides clear documentation for API consumers
- Minimizes complexity in both implementation and maintenance

## Implementation Progress Tracking

### Phase 1: Core Versioning Infrastructure
1. **URL Path Versioning Implementation**
   - [ ] Define version format standards (v1, v2, etc.)
   - [ ] Implement versioned route registration
   - [ ] Create version-aware router setup
   - [ ] Implement version-specific middleware
   
   *Updated at: Not started*

2. **Version-Aware Routing**
   - [ ] Implement route extraction with version context
   - [ ] Create version fallback mechanism
   - [ ] Build version routing utilities
   - [ ] Ensure security context propagation across versions
   
   *Updated at: Not started*

### Phase 2: API Evolution Support
1. **Backward Compatibility Utilities**
   - [ ] Create data model mapping between versions
   - [ ] Implement request/response transformers
   - [ ] Build version migration utilities
   
   *Updated at: Not started*

2. **Version Documentation**
   - [ ] Implement OpenAPI documentation with version support
   - [ ] Create version upgrade guides
   - [ ] Add deprecation notices for older versions
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 20, 2024
- **Next Milestone**: URL Path Versioning Implementation

## Success Criteria
- New API versions can be introduced without breaking existing clients
- Developers can easily maintain multiple versions of endpoints when needed
- API documentation clearly indicates version differences
- Version routing is transparent to business logic
- Security context is preserved across all versions

## Implementation Notes
This approach focuses on URL path-based versioning as the primary strategy due to its simplicity, transparency to clients, and straightforward implementation. This strategy places the version identifier in the URL path (e.g., `/api/v1/users` vs `/api/v2/users`).

We prioritize this approach because:
1. It's explicit and easily understood by API consumers
2. It works well with Axum's routing system
3. It's compatible with OpenAPI documentation tools
4. It allows for clear separation of concerns in the codebase

While header-based or content-negotiation versioning are more technically elegant, they add complexity that isn't justified for our current needs. We can revisit additional versioning strategies in the future if required.

### Example Implementation: Versioned API with Axum

```rust
use axum::{
    Router,
    routing::{get, post},
    extract::{Path, State, Json},
    middleware,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Version-specific data structures
#[derive(Serialize)]
struct UserV1 {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct UserV2 {
    id: String,
    first_name: String,
    last_name: String,
    email: Option<String>,
}

// Version mapping utilities
impl From<UserV2> for UserV1 {
    fn from(user: UserV2) -> Self {
        UserV1 {
            id: user.id,
            name: format!("{} {}", user.first_name, user.last_name),
        }
    }
}

// API version definition
#[derive(Clone, Debug)]
enum ApiVersion {
    V1,
    V2,
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiVersion::V1 => write!(f, "v1"),
            ApiVersion::V2 => write!(f, "v2"),
        }
    }
}

impl TryFrom<&str> for ApiVersion {
    type Error = String;
    
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "v1" => Ok(ApiVersion::V1),
            "v2" => Ok(ApiVersion::V2),
            _ => Err(format!("Unsupported API version: {}", s)),
        }
    }
}

// Version-specific applications
fn v1_routes() -> Router {
    Router::new()
        .route("/users/:id", get(get_user_v1))
        .route("/users", post(create_user_v1))
}

fn v2_routes() -> Router {
    Router::new()
        .route("/users/:id", get(get_user_v2))
        .route("/users", post(create_user_v2))
}

// Version extraction middleware
async fn version_extractor(
    Path(version): Path<String>,
    mut request: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {
    // Parse and validate version
    let api_version = match ApiVersion::try_from(version.as_str()) {
        Ok(version) => version,
        Err(_) => {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::NOT_FOUND)
                .body(axum::body::Body::from("API version not found"))
                .unwrap();
        }
    };
    
    // Add version to request extensions
    request.extensions_mut().insert(api_version);
    
    // Continue processing
    next.run(request).await
}

// Main application with versioned routes
pub fn create_app() -> Router {
    Router::new()
        .nest("/api/:version", 
            Router::new()
                .merge(v1_routes())
                .merge(v2_routes())
                .layer(middleware::from_fn(version_extractor))
        )
}

// Handler implementations
async fn get_user_v1(
    Path(id): Path<String>,
) -> Json<UserV1> {
    // For this example, just create a dummy user
    Json(UserV1 {
        id,
        name: "John Doe".to_string(),
    })
}

async fn get_user_v2(
    Path(id): Path<String>,
) -> Json<UserV2> {
    // For this example, just create a dummy user
    Json(UserV2 {
        id,
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: Some("john.doe@example.com".to_string()),
    })
}

// V1 create user implementation
#[derive(Deserialize)]
struct CreateUserV1 {
    name: String,
}

async fn create_user_v1(
    Json(payload): Json<CreateUserV1>,
) -> Json<UserV1> {
    // For this example, just echo back a user with a new ID
    Json(UserV1 {
        id: uuid::Uuid::new_v4().to_string(),
        name: payload.name,
    })
}

// V2 create user implementation
#[derive(Deserialize)]
struct CreateUserV2 {
    first_name: String,
    last_name: String,
    email: Option<String>,
}

async fn create_user_v2(
    Json(payload): Json<CreateUserV2>,
) -> Json<UserV2> {
    // For this example, just echo back a user with a new ID
    Json(UserV2 {
        id: uuid::Uuid::new_v4().to_string(),
        first_name: payload.first_name,
        last_name: payload.last_name,
        email: payload.email,
    })
}

// Version-aware OpenAPI documentation
fn create_api_doc() -> utoipa::OpenApi {
    #[derive(utoipa::OpenApi)]
    #[openapi(
        paths(
            get_user_v1,
            create_user_v1,
            get_user_v2,
            create_user_v2,
        ),
        components(
            schemas(UserV1, CreateUserV1, UserV2, CreateUserV2)
        ),
        tags(
            (name = "v1", description = "Version 1 API"),
            (name = "v2", description = "Version 2 API - Current"),
        )
    )]
    struct ApiDoc;
    
    ApiDoc::openapi()
}
```

This example demonstrates:
1. Clear path-based versioning with `/api/v1/...` and `/api/v2/...` routes
2. Version-specific data models that can be mapped between versions
3. A middleware-based approach to extract and validate API versions
4. Separate route definitions for each version
5. OpenAPI documentation with version-specific tags

This implementation supports multiple API versions while keeping the code organized and maintainable. It allows for gradual evolution of the API without breaking existing clients.

## References
- [Axum Routing Documentation](https://docs.rs/axum/latest/axum/routing/index.html)
- [REST API Versioning Practices](https://restfulapi.net/versioning/)
- [Semantic Versioning](https://semver.org/)
- [OpenAPI Version Support](https://swagger.io/docs/specification/api-host-and-base-path/) 