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

## Error Handling

1. **Consistent Error Types**
   - Use `ApiError` for all API error responses
   - Map domain errors to appropriate HTTP status codes
   - Provide clear error messages that help clients understand the issue

2. **Validation Errors**
   - Use consistent validation error messages
   - For UUID validation: "Invalid UUID format"
   - For required fields: "Field [name] is required"
   - For invalid input: "Invalid [field] format"

3. **HTTP Status Codes**
   - `400 Bad Request`: Client error, malformed request
   - `401 Unauthorized`: Authentication required
   - `403 Forbidden`: Authentication succeeded but insufficient permissions
   - `404 Not Found`: Resource not found
   - `409 Conflict`: Request conflicts with server state
   - `422 Unprocessable Entity`: Validation errors
   - `500 Internal Server Error`: Server-side error

## Response Structure

1. **Consistent Response Format**
   - All responses should follow a consistent structure
   - Use wrapper types where appropriate (e.g., `ApiResponse<T>`)
   - Include metadata for collections (pagination, total count)

2. **Pagination**
   - Use consistent query parameters:
     - `page`: Page number (1-based)
     - `per_page`: Items per page
   - Include pagination metadata in response

3. **Filtering and Sorting**
   - Use consistent parameter naming for filters
   - Support multiple sorting criteria with direction
   - Document supported filter and sort parameters

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