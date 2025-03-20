# Pragmatic API Versioning

## Overview
A straightforward approach to API versioning for our Rust Axum backend that enables safe evolution of APIs without breaking client compatibility, focusing primarily on URL path-based versioning for simplicity and clarity.

## Current State
Our application needs a structured approach to API versioning to support API evolution while maintaining backward compatibility with existing clients.

## Target State
A focused API versioning system that:
- Uses URL path-based versioning as the primary strategy
- Securely maintains compatibility across versions
- Integrates seamlessly with Axum routing
- Provides clear API documentation for each version
- Keeps implementation complexity to a minimum

## Implementation Progress Tracking

### Phase 1: Core Versioning Infrastructure
1. **URL Path Versioning**
   - [ ] Implement path-based versioning structure (/v1/api/resource)
   - [ ] Create Axum router factory with version prefix support
   - [ ] Add version extraction utilities
   
   *Updated at: Not started*

2. **Version-Aware Routing**
   - [ ] Build version-specific route registration
   - [ ] Implement route grouping by version
   - [ ] Create default version routing rules
   
   *Updated at: Not started*

3. **Security Context Preservation**
   - [ ] Ensure Entra authentication works consistently across versions
   - [ ] Implement permission checks with version awareness
   - [ ] Add security audit logging for version-specific access
   
   *Updated at: Not started*

### Phase 2: API Evolution Support
1. **Backward Compatibility Utilities**
   - [ ] Create request/response transformation utilities
   - [ ] Implement data field mapping between versions
   - [ ] Add support for handling removed or renamed fields
   
   *Updated at: Not started*

2. **Version Documentation**
   - [ ] Integrate version information in OpenAPI documentation
   - [ ] Implement version-specific schema generation
   - [ ] Add deprecation notices for older versions
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 20, 2024
- **Next Milestone**: URL Path Versioning

## Success Criteria
- New API versions can be introduced without breaking existing clients
- API versioning is transparent to developers and easy to use
- Version-specific documentation is clear and accessible
- Security context is properly maintained across versions
- Implementation has minimal overhead

## Implementation Notes
This approach focuses on URL path-based versioning (/v1/api/resource) as the primary strategy for its simplicity, client compatibility, and ease of implementation with Axum's routing system. Additional versioning strategies can be added later if needed, but a single consistent approach will keep the implementation simple and maintainable.

### Example Implementation

```rust
use axum::{
    routing::{get, post},
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    middleware,
};
use std::sync::Arc;

// Version prefix extractor
pub struct ApiVersion(pub String);

// Version-aware router builder
pub struct VersionedApi {
    inner_routers: Vec<(String, Router)>,
}

impl VersionedApi {
    pub fn new() -> Self {
        Self {
            inner_routers: Vec::new(),
        }
    }
    
    // Add a versioned router
    pub fn version(mut self, version: &str, router: Router) -> Self {
        self.inner_routers.push((version.to_string(), router));
        self
    }
    
    // Build the combined router with versioned paths
    pub fn build(self) -> Router {
        let mut app = Router::new();
        
        // Add each versioned router under its prefix
        for (version, router) in self.inner_routers {
            app = app.nest(&format!("/{}", version), router);
        }
        
        // Add a fallback route to the latest version or documentation
        app = app.route("/", get(|| async {
            "API requires version prefix. Please use /v1/... or see documentation."
        }));
        
        app
    }
}

// Example of creating API versions
pub fn create_api() -> Router {
    // Create a v1 router
    let v1_api = Router::new()
        .route("/users", get(list_users_v1).post(create_user_v1))
        .route("/users/:id", get(get_user_v1));
    
    // Create a v2 router with enhanced functionality
    let v2_api = Router::new()
        .route("/users", get(list_users_v2).post(create_user_v2))
        .route("/users/:id", get(get_user_v2).delete(delete_user_v2))
        .route("/users/:id/profile", get(get_user_profile_v2));
    
    // Combine them in the versioned API
    VersionedApi::new()
        .version("v1", v1_api)
        .version("v2", v2_api)
        .build()
}

// Backward compatibility mapping between versions
async fn list_users_v1() -> Json<Vec<UserV1>> {
    // Fetch users using repository
    let users = fetch_users().await;
    
    // Map to V1 response format
    let v1_users = users.into_iter()
        .map(|user| UserV1 {
            id: user.id,
            name: user.name,
            email: user.email,
            // V1 doesn't include created_at
        })
        .collect();
    
    Json(v1_users)
}

async fn list_users_v2() -> Json<Vec<UserV2>> {
    // Fetch users with enhanced fields
    let users = fetch_users().await;
    
    // Return in V2 format with additional fields
    Json(users)
}

// Simplified handler for user creation in v1
async fn create_user_v1(
    Json(payload): Json<CreateUserV1Request>,
    State(db): State<Arc<DbPool>>,
) -> Result<Json<UserV1>, StatusCode> {
    // Create user with only v1 fields
    let user = db.create_user(
        &payload.name,
        &payload.email,
        &payload.password,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(UserV1 {
        id: user.id,
        name: user.name,
        email: user.email,
    }))
}

// Enhanced handler for user creation in v2
async fn create_user_v2(
    Json(payload): Json<CreateUserV2Request>,
    State(db): State<Arc<DbPool>>,
) -> Result<Json<UserV2>, StatusCode> {
    // Create user with additional v2 fields
    let user = db.create_user_v2(
        &payload.name,
        &payload.email,
        &payload.password,
        payload.preferences,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(UserV2 {
        id: user.id,
        name: user.name,
        email: user.email,
        created_at: user.created_at,
        preferences: user.preferences,
    }))
}

// Data structures for different versions
#[derive(serde::Serialize)]
struct UserV1 {
    id: String,
    name: String,
    email: String,
}

#[derive(serde::Serialize)]
struct UserV2 {
    id: String,
    name: String,
    email: String,
    created_at: String,
    preferences: UserPreferences,
}

#[derive(serde::Deserialize)]
struct CreateUserV1Request {
    name: String,
    email: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct CreateUserV2Request {
    name: String,
    email: String,
    password: String,
    preferences: UserPreferences,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UserPreferences {
    theme: String,
    notifications_enabled: bool,
}

// Integrating with OpenAPI documentation
pub fn api_docs(api_versioned: bool) -> Router {
    let mut openapi = utoipa::OpenApi::new(
        "My API",
        "1.0",
    );
    
    if api_versioned {
        // Add version-specific paths and components
        openapi = openapi
            .tag(utoipa::openapi::Tag {
                name: "v1".to_string(),
                description: Some("API Version 1".to_string()),
                ..Default::default()
            })
            .tag(utoipa::openapi::Tag {
                name: "v2".to_string(),
                description: Some("API Version 2 (latest)".to_string()),
                ..Default::default()
            });
            
        // Add version-specific paths
        openapi = openapi.path("/v1/users", utoipa::Path::new()
            .get(utoipa::operation::Operation::new()
                .summary("List users (v1)")
                .tag("v1")
                .response("200", utoipa::Response::new()
                    .description("List of users")
                    .content("application/json", utoipa::Content::new(
                        utoipa::Component::new("UserV1").to_schema()
                    ))
                )
            )
        );
        
        openapi = openapi.path("/v2/users", utoipa::Path::new()
            .get(utoipa::operation::Operation::new()
                .summary("List users (v2)")
                .tag("v2")
                .response("200", utoipa::Response::new()
                    .description("List of users with additional fields")
                    .content("application/json", utoipa::Content::new(
                        utoipa::Component::new("UserV2").to_schema()
                    ))
                )
            )
        );
    }
    
    // Serve OpenAPI JSON and Swagger UI
    Router::new()
        .route("/openapi.json", get(|| async move { Json(openapi) }))
        .route("/docs", get(swagger_ui))
}
```

## References
- [Axum Routing Documentation](https://docs.rs/axum/latest/axum/routing/index.html)
- [REST API Versioning Practices](https://restfulapi.net/versioning/)
- [Semantic Versioning](https://semver.org/)
- [OpenAPI Version Support](https://swagger.io/specification/#version-3-0-3) 