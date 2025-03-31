# Navius API Design Guidelines

## Introduction

These guidelines establish standardized patterns for API development in the Navius framework. Following these guidelines ensures consistency, improves maintainability, and enhances the developer experience across all Navius applications.

## Core Principles

1. **Consistency** - APIs should behave predictably across all endpoints
2. **Security by Default** - Authentication requirements should be explicit 
3. **Error Clarity** - Error responses should be meaningful and consistent
4. **Resource-Oriented Design** - APIs should follow REST principles where appropriate
5. **Performance Awareness** - API design should consider performance implications

## Controller Design

### Function Signatures

All controller functions should follow this pattern:

```rust
pub async fn endpoint_name(
    State(registry): State<Arc<ServiceRegistry>>,
    current_user: CurrentUser,  // When authentication is required
    Path(id_str): Path<String>, // For path parameters
    Query(params): Query<QueryParams>, // For query parameters 
    Json(request): Json<RequestType>, // For request body
) -> Result<Json<ResponseType>> {
    // Implementation
}
```

### Authentication

1. Include the `CurrentUser` extractor in all endpoints that require authentication
2. Make the parameter name either `current_user` or prefixed with an underscore if unused
3. Extract the user ID early in the function with: `let user_id = current_user.0;`

```rust
// Example with authentication
pub async fn get_user_tasks(
    State(registry): State<Arc<ServiceRegistry>>,
    current_user: CurrentUser,
    Query(params): Query<TaskListParams>,
) -> Result<Json<Vec<TaskResponse>>> {
    let user_id = current_user.0;
    // Implementation
}

// Example without authentication (public endpoint)
pub async fn health_check(
    State(registry): State<Arc<ServiceRegistry>>,
) -> Result<Json<HealthResponse>> {
    // Implementation
}
```

### Parameter Handling

#### Path Parameters

1. Always extract path parameters as strings first
2. Parse and validate parameters explicitly
3. Use consistent error messages for validation failures

```rust
pub async fn get_task(
    State(registry): State<Arc<ServiceRegistry>>,
    current_user: CurrentUser,
    Path(id_str): Path<String>,
) -> Result<Json<TaskResponse>> {
    let task_id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid task ID format"))?;
    
    // Implementation
}
```

#### Query Parameters

1. Define explicit structs for query parameters
2. Use consistent naming for similar parameters across endpoints
3. Implement proper defaults for optional parameters

```rust
#[derive(Debug, Deserialize)]
pub struct TaskListParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
}
```

### Response Types

1. Name response DTOs consistently as `EntityResponse`
2. Include all necessary fields for client consumption
3. Use appropriate serialization attributes

```rust
#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub assigned_to: Option<String>,
}
```

### Request Types

1. Name creation requests as `CreateEntityRequest` 
2. Name update requests as `UpdateEntityRequest`
3. Use appropriate validation attributes

```rust
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub category_id: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub category_id: Option<String>,
    pub due_date: Option<String>,
}
```

## Error Handling

### Standard Error Types

Use the standard error types consistently:

1. `Error::validation_error()` - For invalid input formats or validation failures
2. `Error::not_found()` - When a requested resource doesn't exist
3. `Error::unauthorized()` - When authentication is missing or invalid
4. `Error::forbidden()` - When user doesn't have permission for the action
5. `Error::conflict()` - When there's a conflict (e.g. duplicate resource)
6. `Error::internal_server_error()` - For unexpected server-side errors

### Error Message Format

1. Use concise but descriptive messages
2. Be specific about the nature of the error
3. Don't expose sensitive details or implementation specifics

```rust
// Good
Error::validation_error("Invalid task ID format")

// Bad
Error::validation_error("ID 123e4567-e89b-12d3-a456-426614174000 failed UUID parsing")
```

### Error Mapping from Services

Map service errors to appropriate HTTP errors consistently:

```rust
.map_err(|e| match e.kind {
    TaskServiceErrorKind::NotFound => Error::not_found("Task not found"),
    TaskServiceErrorKind::ValidationError => Error::validation_error(e.message),
    TaskServiceErrorKind::PermissionDenied => Error::forbidden("You don't have permission to modify this task"),
    _ => Error::internal_server_error(format!("Task error: {}", e.message)),
})?;
```

## Routing Configuration

### Route Structure

1. Group related endpoints under the same resource path
2. Use plural nouns for resource collections
3. Follow a consistent URL structure:
   - Collection: `/api/resources`
   - Individual: `/api/resources/{id}`
   - Sub-resources: `/api/resources/{id}/sub-resources`

### Middleware Application

1. Apply authentication middleware at the route level
2. Use role-based middleware for sensitive operations
3. Configure consistent middleware chains

```rust
pub fn configure_routes(
    server: HttpServerBuilder,
    registry: Arc<ServiceRegistry>,
) -> HttpServerBuilder {
    server
        .route("/api/tasks", get(tasks::get_tasks)
            .layer(requires_auth(registry.clone())))
        .route("/api/tasks/{id}", get(tasks::get_task)
            .layer(requires_auth(registry.clone())))
        .route("/api/admin/tasks", post(tasks::create_task)
            .layer(requires_manager(registry.clone())))
}
```

## HTTP Methods

Use HTTP methods consistently:

1. `GET` - For retrieving resources
2. `POST` - For creating new resources
3. `PUT` - For full updates of existing resources
4. `PATCH` - For partial updates of existing resources
5. `DELETE` - For removing resources

## Versioning

1. Use URI versioning for major API versions (e.g., `/api/v1/tasks`)
2. Document version compatibility in API specifications
3. Maintain backwards compatibility within the same major version

## Documentation

1. Always include descriptive doc comments on public types and functions
2. Document request and response types thoroughly
3. Include example requests and responses in documentation

## Testing

1. Write comprehensive tests for each endpoint
2. Test happy paths, edge cases, and error scenarios
3. Include tests for authentication and authorization

---

These guidelines should be followed for all new API development and applied to existing APIs during maintenance and refactoring phases. 