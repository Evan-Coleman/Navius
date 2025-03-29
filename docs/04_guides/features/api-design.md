---
title: Navius API Design Guide
description: Best practices for designing and implementing APIs in Navius applications
category: guides
tags:
  - api
  - integration
  - design
  - patterns
related:
  - ../development/testing.md
  - ../../reference/architecture/principles.md
  - api-integration.md
last_updated: March 27, 2025
version: 1.0
---

# Navius API Design Guide

## Overview
This guide outlines best practices and patterns for designing APIs in Navius applications. It covers API design principles, implementation approaches, error handling strategies, and performance considerations to help you build consistent, maintainable, and user-friendly APIs.

## Prerequisites
Before using this guide, you should have:

- Basic understanding of RESTful API principles
- Familiarity with Rust and Navius framework basics
- Knowledge of HTTP status codes and request/response patterns

## API Design Principles

Navius follows these core API design principles:

1. **Resource-Oriented Design**: Focus on resources and their representations
2. **Predictable URLs**: Use consistent URL patterns for resources
3. **Proper HTTP Methods**: Use appropriate HTTP methods for operations
4. **Consistent Error Handling**: Standardize error responses
5. **Versioned APIs**: Support API versioning for backward compatibility

## Step-by-step API Design

### 1. Define Your Resources

Start by identifying the core resources in your application domain:

```rust
// Example resource definitions
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 2. Design Resource URLs

Use consistent URL patterns for resources:

| Resource | URL Pattern | Description |
|----------|-------------|-------------|
| Collection | `/api/v1/users` | The collection of all users |
| Individual | `/api/v1/users/{id}` | A specific user by ID |
| Sub-collection | `/api/v1/users/{id}/posts` | All posts for a user |
| Sub-resource | `/api/v1/users/{id}/posts/{post_id}` | A specific post for a user |

### 3. Choose HTTP Methods

Map operations to appropriate HTTP methods:

| Operation | HTTP Method | URL | Description |
|-----------|-------------|-----|-------------|
| List | GET | `/api/v1/users` | Get all users (paginated) |
| Read | GET | `/api/v1/users/{id}` | Get a specific user |
| Create | POST | `/api/v1/users` | Create a new user |
| Update | PUT/PATCH | `/api/v1/users/{id}` | Update a user |
| Delete | DELETE | `/api/v1/users/{id}` | Delete a user |

### 4. Define Request/Response Schemas

Create clear input and output schemas:

```rust
// Request schema
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    
    #[validate(length(min = 8))]
    pub password: String,
}

// Response schema
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}
```

### 5. Implement Route Handlers

Create handlers that process requests:

```rust
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.user_service.get_user(id).await?;
    
    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role.to_string(),
        created_at: user.created_at,
    }))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    // Validate the request
    request.validate()?;
    
    // Create the user
    let user = state.user_service.create_user(request).await?;
    
    // Return 201 Created with the user response
    Ok((
        StatusCode::CREATED,
        Json(UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role.to_string(),
            created_at: user.created_at,
        }),
    ))
}
```

### 6. Register API Routes

Register your API routes with the router:

```rust
pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/users/:id/posts", get(list_user_posts))
}

// In your main router
let api_router = Router::new()
    .nest("/v1", user_routes())
    .layer(ValidateRequestHeaderLayer::bearer())
    .layer(Extension(rate_limiter));
```

## API Error Handling

### Standard Error Response Format

Navius uses a consistent error format:

```json
{
  "error": {
    "type": "validation_error",
    "message": "The request was invalid",
    "details": [
      {
        "field": "email",
        "message": "Must be a valid email address"
      }
    ]
  }
}
```

### Implementing Error Handling

Use the `AppError` type for error handling:

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden")]
    Forbidden,
    
    #[error("Validation error")]
    Validation(#[from] ValidationError),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found", self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", self.to_string()),
            AppError::Validation(e) => (StatusCode::BAD_REQUEST, "validation_error", self.to_string()),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "An internal server error occurred".to_string(),
            ),
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                "A database error occurred".to_string(),
            ),
        };
        
        let error_response = json!({
            "error": {
                "type": error_type,
                "message": message,
                "details": get_error_details(&self),
            }
        });
        
        (status, Json(error_response)).into_response()
    }
}
```

## Validation

### Request Validation

Navius leverages the `validator` crate for request validation:

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    
    #[validate(length(min = 1))]
    pub content: String,
    
    #[serde(default)]
    pub published: bool,
}

// In your handler
pub async fn create_post(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostResponse>), AppError> {
    // Validate the request
    request.validate()?;
    
    // Create the post
    let post = state.post_service.create_post(user_id, request).await?;
    
    // Return 201 Created
    Ok((StatusCode::CREATED, Json(post.into())))
}
```

## Versioning Strategies

Navius supports these API versioning strategies:

### URL Versioning

```
/api/v1/users
/api/v2/users
```

This is implemented by nesting routes:

```rust
let api_router = Router::new()
    .nest("/v1", v1_routes())
    .nest("/v2", v2_routes());
```

### Header Versioning

```
GET /api/users
Accept-Version: v1
```

This requires a custom extractor:

```rust
pub struct ApiVersion(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for ApiVersion
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut RequestParts, _state: &S) -> Result<Self, Self::Rejection> {
        let version = parts
            .headers
            .get("Accept-Version")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("v1")
            .to_string();
            
        Ok(ApiVersion(version))
    }
}

// Using in a handler
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ApiVersion(version): ApiVersion,
) -> Result<Json<UserResponse>, AppError> {
    match version.as_str() {
        "v1" => {
            let user = state.user_service.get_user(id).await?;
            Ok(Json(v1::UserResponse::from(user)))
        }
        "v2" => {
            let user = state.user_service.get_user_with_details(id).await?;
            Ok(Json(v2::UserResponse::from(user)))
        }
        _ => Err(AppError::NotFound),
    }
}
```

## Performance Optimization

### Pagination

Implement consistent pagination for collection endpoints:

```rust
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: usize,
    
    #[serde(default = "default_page_size")]
    pub page_size: usize,
}

fn default_page() -> usize {
    1
}

fn default_page_size() -> usize {
    20
}

// Using in a handler
pub async fn list_users(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<UserResponse>>, AppError> {
    let (users, total) = state.user_service
        .list_users(pagination.page, pagination.page_size)
        .await?;
        
    let response = PaginatedResponse {
        data: users.into_iter().map(UserResponse::from).collect(),
        page: pagination.page,
        page_size: pagination.page_size,
        total,
    };
    
    Ok(Json(response))
}
```

### Filtering and Sorting

Support consistent query parameters for filtering and sorting:

```rust
#[derive(Debug, Deserialize)]
pub struct UserFilterParams {
    pub role: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

// Using in a handler
pub async fn list_users(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(filter): Query<UserFilterParams>,
) -> Result<Json<PaginatedResponse<UserResponse>>, AppError> {
    let (users, total) = state.user_service
        .list_users_with_filter(
            pagination.page,
            pagination.page_size,
            filter,
        )
        .await?;
        
    // Create response
    let response = PaginatedResponse { /*...*/ };
    
    Ok(Json(response))
}
```

## Testing API Endpoints

Navius provides utilities for API testing:

```rust
#[tokio::test]
async fn test_create_user() {
    // Create test app
    let app = TestApp::new().await;
    
    // Create test request
    let request = json!({
        "email": "test@example.com",
        "name": "Test User",
        "password": "password123"
    });
    
    // Send request
    let response = app
        .post("/api/v1/users")
        .json(&request)
        .send()
        .await;
    
    // Assert response
    assert_eq!(response.status(), 201);
    
    let user: UserResponse = response.json().await;
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.name, "Test User");
}
```

## API Documentation

### OpenAPI Integration

Navius supports OpenAPI documentation generation:

```rust
// In your main.rs
let api_docs = OpenApiDocumentBuilder::new()
    .title("Navius API")
    .version("1.0.0")
    .description("API for Navius application")
    .build();
    
// Register documentation routes
let app = Router::new()
    .nest("/api", api_router)
    .nest("/docs", OpenApiRouter::new(api_docs));
```

## Related Documents

- [API Integration Guide](api-integration.md) - Integrating with external APIs
- [Authentication Guide](authentication.md) - API authentication
- [Testing Guide](../development/testing.md) - Testing API endpoints 