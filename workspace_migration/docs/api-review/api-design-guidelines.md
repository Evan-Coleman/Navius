# Navius API Design Guidelines

## Overview

This document outlines the standardized patterns and best practices for API design within the Navius framework, specifically for the Full Stack Integration Example. These guidelines ensure consistency, maintainability, and usability across all API endpoints.

## RESTful API Standards

### Route Structure

1. **Resource-Based Routes**
   - Routes should be named after resources, not actions
   - Use plural nouns for collection resources (e.g., `/api/users`, `/api/tasks`)
   - Use singular resource with ID for specific resources (e.g., `/api/users/:id`)

2. **HTTP Methods**
   - Use appropriate HTTP methods for operations:
     - `GET`: Retrieve data
     - `POST`: Create new resources
     - `PUT`: Update existing resources
     - `DELETE`: Remove resources
   - Avoid using HTTP methods for actions that don't match their semantic meaning

3. **Sub-Resources**
   - Represent relationships using sub-resources:
     - `GET /api/categories/:id/tasks` - Get tasks for a category
     - `GET /api/tasks/:id/comments` - Get comments for a task

4. **Route Organization**
   - Group routes by resource type
   - Add clear comments to distinguish between:
     - Collection routes (`/api/resources`)
     - Individual resource routes (`/api/resources/:id`)
     - Sub-resource routes (`/api/resources/:id/sub-resources`)
     - Action routes (when necessary, e.g., `/api/resources/:id/specific-action`)

### Route Configuration Example

```rust
server
    // Collection routes
    .route(
        "/api/categories",
        get(get_categories).layer(requires_auth(registry.clone())),
    )
    .route(
        "/api/categories",
        post(create_category)
            .layer(requires_manager())
            .layer(requires_auth(registry.clone())),
    )
    // Individual resource routes
    .route(
        "/api/categories/:id",
        get(get_category).layer(requires_auth(registry.clone())),
    )
    // Sub-resource routes
    .route(
        "/api/categories/:id/tasks",
        get(get_tasks_by_category).layer(requires_auth(registry.clone())),
    )
```

## Authentication and Authorization

1. **Authentication Middleware**
   - Apply authentication using the `requires_auth` middleware for protected routes
   - Stack multiple middleware layers with the most specific first:
     ```rust
     .layer(requires_manager())
     .layer(requires_auth(registry.clone()))
     ```

2. **User Extractors**
   - Use the `CurrentUser` extractor to access authenticated user information
   - Apply appropriate role-based access control:
     - `requires_auth`: Basic authentication
     - `requires_admin`: Administrator access
     - `requires_manager`: Manager access
     - `requires_self_or_admin`: User can only access own resources (or admin)

3. **Public Endpoints**
   - Clearly document public endpoints that don't require authentication
   - Health check endpoints should always be public

## Controller Design

1. **Function Structure**
   - Each controller function should handle one specific operation
   - Use clear parameter names that reflect their purpose
   - Include documentation comments explaining the function's purpose and parameters

2. **Parameter Extraction**
   - Use Axum's extractors consistently:
     - `Path<T>` for path parameters
     - `Query<T>` for query parameters
     - `Json<T>` for request bodies
     - `State<Arc<ServiceRegistry>>` for accessing services
     - `CurrentUser` for authenticated user information

3. **Example Controller Function**
   ```rust
   /// Get a category by ID
   pub async fn get_category(
       Path(id): Path<Uuid>,
       State(registry): State<Arc<ServiceRegistry>>,
       current_user: CurrentUser,
   ) -> Result<Json<CategoryResponse>, ApiError> {
       // Implementation...
   }
   ```

## Response Structures

### Standard Response Envelope

All API responses should use the `ApiResponse<T>` wrapper for consistency:

```rust
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: T,
    /// Response metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}
```

Example response:

```json
{
  "data": {
    // Response data specific to the endpoint
  },
  "meta": {
    "request_id": "req-uuid-1234-5678",
    "additional": {
      // Optional additional metadata
    }
  }
}
```

Implementation:

```rust
pub async fn get_resource(
    // Parameters...
) -> Result<Json<ApiResponse<ResourceResponse>>> {
    // Implementation...
    Ok(Json(ApiResponse::new(resource_response)))
}
```

### Paginated Responses

All list endpoints should use the `PaginatedResponse<T>` wrapper:

```rust
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// Data items
    pub data: Vec<T>,
    /// Pagination metadata
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    /// Current page number
    pub page: u32,
    /// Number of items per page
    pub per_page: u32,
    /// Total number of items
    pub total_items: u64,
    /// Total number of pages
    pub total_pages: u32,
    /// Has more pages
    pub has_more: bool,
}
```

Example paginated response:

```json
{
  "data": [
    // Array of items
  ],
  "meta": {
    "page": 1,
    "per_page": 20,
    "total_items": 243,
    "total_pages": 13,
    "has_more": true
  }
}
```

Implementation:

```rust
pub async fn get_resources(
    Query(params): Query<ResourceListParams>,
    // Other parameters...
) -> Result<Json<PaginatedResponse<ResourceResponse>>> {
    // Implementation...
    let response = PaginatedResponse::from_params(
        resource_responses,
        &params.pagination,
        total_count,
    );
    Ok(Json(response))
}
```

### Standardized Pagination Parameters

All list endpoints should use these standard pagination parameters:

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct PaginationParams {
    /// Page number (1-based)
    pub page: Option<u32>,
    /// Number of items per page
    pub per_page: Option<u32>,
}
```

Default values: page=1, per_page=20

### Standard Sorting Parameters

All list endpoints should use these standard sorting parameters:

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct SortParams {
    /// Field to sort by
    pub sort: Option<String>,
    /// Sort direction (asc or desc)
    pub order: Option<String>,
}
```

## Error Handling

1. **Consistent Error Types**
   - Use `ApiError` for all API error responses
   - Map domain errors to appropriate HTTP status codes
   - Provide clear error messages that help clients understand the issue

2. **Standardized Error Response**

   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ErrorResponse {
       /// Error code
       pub code: ErrorCode,
       /// Error message
       pub message: String,
       /// Error details (for validation errors)
       #[serde(skip_serializing_if = "Option::is_none")]
       pub details: Option<ErrorDetails>,
       /// Request ID for tracking
       #[serde(skip_serializing_if = "Option::is_none")]
       pub request_id: Option<String>,
   }
   ```

   Example error response:

   ```json
   {
     "code": "VALIDATION_ERROR",
     "message": "Invalid input parameters",
     "details": {
       "field_errors": [
         {
           "field": "email",
           "message": "Invalid email format"
         }
       ]
     },
     "request_id": "req-uuid-1234-5678"
   }
   ```

3. **Error Codes**

   All errors should use one of these standard error codes:

   ```rust
   pub enum ErrorCode {
       /// Bad request - client error
       BadRequest,
       /// Unauthorized - authentication required
       Unauthorized,
       /// Forbidden - insufficient permissions
       Forbidden,
       /// Not found - resource not found
       NotFound,
       /// Conflict - resource already exists or state conflict
       Conflict,
       /// Validation error - invalid input
       ValidationError,
       /// Internal server error
       InternalServerError,
   }
   ```

4. **HTTP Status Codes**
   - `400 Bad Request`: Client error, malformed request
   - `401 Unauthorized`: Authentication required
   - `403 Forbidden`: Authentication succeeded but insufficient permissions
   - `404 Not Found`: Resource not found
   - `409 Conflict`: Request conflicts with server state
   - `422 Unprocessable Entity`: Validation errors
   - `500 Internal Server Error`: Server-side error

## Testing

1. **Test Coverage**
   - Test all success and error paths
   - Test authentication and authorization requirements
   - Test with invalid inputs to verify validation

2. **API Tests**
   - Create integration tests that test the API as a client would
   - Test response status codes and body content
   - Test resource creation, retrieval, update, and deletion

## Documentation

1. **Code Documentation**
   - Add doc comments to all public controller functions
   - Document request/response types
   - Explain authentication requirements

2. **OpenAPI/Swagger Documentation**
   - Generate OpenAPI documentation
   - Include examples for common operations
   - Document authentication requirements

## Appendix

### Common Patterns

**Authentication Controllers**
```rust
pub async fn login(
    Json(credentials): Json<LoginRequest>,
    State(registry): State<Arc<ServiceRegistry>>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Implementation...
}
```

**User Controllers**
```rust
pub async fn get_user(
    Path(id): Path<Uuid>,
    State(registry): State<Arc<ServiceRegistry>>,
    current_user: CurrentUser,
) -> Result<Json<UserResponse>, ApiError> {
    // Implementation...
}
```

**Resource Controllers**
```rust
pub async fn create_resource(
    Json(request): Json<CreateResourceRequest>,
    State(registry): State<Arc<ServiceRegistry>>,
    current_user: CurrentUser,
) -> Result<Json<ResourceResponse>, ApiError> {
    // Implementation...
}
``` 