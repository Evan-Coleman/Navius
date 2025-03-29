---
title: "Authorization Guide"
description: "Comprehensive guide for implementing role-based and attribute-based authorization in Navius applications"
category: "Guides"
tags: ["security", "authorization", "roles", "permissions", "access control", "RBAC", "ABAC"]
last_updated: "April 6, 2025"
version: "1.0"
---

# Authorization Guide

## Overview

This guide provides detailed instructions for implementing authorization in Navius applications. Authorization determines what actions authenticated users can perform and what resources they can access.

## Authorization Concepts

### Authentication vs. Authorization

- **Authentication** (covered in [Authentication Guide](./authentication-implementation.md)) verifies who the user is
- **Authorization** (covered in this guide) determines what the user can do

### Authorization Models

Navius supports multiple authorization models:

1. **Role-Based Access Control (RBAC)** - Permissions assigned to roles, which are assigned to users
2. **Attribute-Based Access Control (ABAC)** - Permissions based on user attributes, resource attributes, and context
3. **Resource-Based Access Control** - Permissions tied directly to resources

## Role-Based Access Control (RBAC)

### Core Components

RBAC consists of:

- **Users** - Individuals who need access to the system
- **Roles** - Named collections of permissions (e.g., Admin, Editor, Viewer)
- **Permissions** - Specific actions that can be performed (e.g., read, write, delete)

### Implementing RBAC in Navius

#### Configuration

Configure RBAC in your `config/default.yaml`:

```yaml
authorization:
  type: "rbac"
  default_role: "user"
  roles:
    - name: "admin"
      permissions: ["user:read", "user:write", "user:delete", "config:read", "config:write"]
    - name: "editor"
      permissions: ["user:read", "user:write", "config:read"]
    - name: "viewer"
      permissions: ["user:read", "config:read"]
```

#### Implementation

Create the authorization service:

```rust
use navius::auth::authorization::{AuthorizationService, RoleBasedAuthorization};

// Create the authorization service
let auth_service = RoleBasedAuthorization::from_config(&config)?;

// Check if a user has a permission
async fn can_perform_action(
    user_id: Uuid,
    permission: &str,
    auth_service: &impl AuthorizationService,
) -> Result<bool, Error> {
    let has_permission = auth_service.has_permission(user_id, permission).await?;
    Ok(has_permission)
}
```

#### Middleware

Implement authorization middleware:

```rust
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

// Define required permission for route
struct RequiredPermission(String);

// Authorization middleware
async fn authorize_middleware(
    State(state): State<AppState>,
    extensions: Extensions,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get user ID from authenticated session
    let user_id = extensions.get::<UserId>().ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Get required permission from route data
    let required_permission = req.extensions()
        .get::<RequiredPermission>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Check if user has the required permission
    let has_permission = state.auth_service
        .has_permission(user_id.0, &required_permission.0)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}

// Apply middleware to routes
let app = Router::new()
    .route("/users", get(get_users_handler))
    .layer(axum::Extension(RequiredPermission("user:read".to_string())))
    .route_layer(middleware::from_fn_with_state(app_state.clone(), authorize_middleware))
    .with_state(app_state);
```

#### Role Management Functions

```rust
// Assign a role to a user
async fn assign_role(
    user_id: Uuid,
    role: &str,
    auth_service: &impl AuthorizationService,
) -> Result<(), Error> {
    auth_service.assign_role(user_id, role).await?;
    Ok(())
}

// Remove a role from a user
async fn remove_role(
    user_id: Uuid,
    role: &str,
    auth_service: &impl AuthorizationService,
) -> Result<(), Error> {
    auth_service.remove_role(user_id, role).await?;
    Ok(())
}

// Get all roles for a user
async fn get_user_roles(
    user_id: Uuid,
    auth_service: &impl AuthorizationService,
) -> Result<Vec<String>, Error> {
    let roles = auth_service.get_roles(user_id).await?;
    Ok(roles)
}
```

## Attribute-Based Access Control (ABAC)

ABAC provides more fine-grained control by considering:

- **User attributes** - Properties of the user (role, department, location, clearance)
- **Resource attributes** - Properties of the resource being accessed (type, owner, classification)
- **Action attributes** - Properties of the action being performed (read, write, delete)
- **Context attributes** - Environmental factors (time, location, device)

### Implementing ABAC in Navius

#### Configuration

Configure ABAC in your `config/default.yaml`:

```yaml
authorization:
  type: "abac"
  policy_location: "./policies"
  default_deny: true
```

#### Creating Policies

Define ABAC policies in Rego (Open Policy Agent language):

```ruby
# policies/user_access.rego
package navius.authorization

default allow = false

# Allow users to read their own profile
allow {
  input.action == "read"
  input.resource.type == "user_profile"
  input.resource.id == input.user.id
}

# Allow users to read public documents
allow {
  input.action == "read"
  input.resource.type == "document"
  input.resource.visibility == "public"
}

# Allow admins to perform any action
allow {
  "admin" in input.user.roles
}

# Allow managers to access their team's data
allow {
  input.action == "read"
  input.resource.type == "team_data"
  input.resource.team_id == input.user.team_id
  input.user.role == "manager"
}
```

#### Implementation

Create the ABAC authorization service:

```rust
use navius::auth::authorization::{AuthorizationService, AbacAuthorization};

// Create the authorization service
let auth_service = AbacAuthorization::new(&config)?;

// Check if a user can perform an action on a resource
async fn can_access_resource(
    user: &User,
    action: &str,
    resource: &Resource,
    context: &Context,
    auth_service: &impl AuthorizationService,
) -> Result<bool, Error> {
    let access_request = AccessRequest {
        user,
        action,
        resource,
        context,
    };
    
    let allowed = auth_service.evaluate(access_request).await?;
    Ok(allowed)
}
```

#### ABAC Middleware

```rust
// ABAC authorization middleware
async fn abac_authorize_middleware(
    State(state): State<AppState>,
    extensions: Extensions,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get user from authenticated session
    let user = extensions.get::<User>().ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Get resource and action from request
    let resource = extract_resource_from_request(&req)?;
    let action = extract_action_from_method(req.method())?;
    
    // Create context from request
    let context = Context {
        time: chrono::Utc::now(),
        ip_address: req.remote_addr().map(|addr| addr.to_string()),
        user_agent: req.headers().get("User-Agent").map(|ua| ua.to_str().unwrap_or("").to_string()),
    };
    
    // Evaluate access request
    let access_request = AccessRequest {
        user,
        action: &action,
        resource: &resource,
        context: &context,
    };
    
    let allowed = state.auth_service
        .evaluate(access_request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !allowed {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}
```

## Resource-Based Access Control

In resource-based access control, permissions are directly tied to resources:

```rust
// Resource with access control
struct Document {
    id: Uuid,
    title: String,
    content: String,
    owner_id: Uuid,
    shared_with: Vec<Uuid>,
    public: bool,
}

impl Document {
    // Check if a user can access this document
    fn can_access(&self, user_id: Uuid) -> bool {
        self.public || self.owner_id == user_id || self.shared_with.contains(&user_id)
    }
    
    // Check if a user can edit this document
    fn can_edit(&self, user_id: Uuid) -> bool {
        self.owner_id == user_id || self.shared_with.contains(&user_id)
    }
    
    // Check if a user can delete this document
    fn can_delete(&self, user_id: Uuid) -> bool {
        self.owner_id == user_id
    }
}
```

## Declarative Authorization with Route Attributes

Navius provides a declarative approach to authorization using route attributes:

```rust
use navius::auth::authorization::{requires_permission, requires_role};

// Require a specific permission
#[get("/users")]
#[requires_permission("user:read")]
async fn get_users_handler() -> impl IntoResponse {
    // Handler implementation
}

// Require a specific role
#[post("/users")]
#[requires_role("admin")]
async fn create_user_handler() -> impl IntoResponse {
    // Handler implementation
}

// Multiple requirements
#[delete("/users/{id}")]
#[requires_permission("user:delete")]
#[requires_role("admin")]
async fn delete_user_handler() -> impl IntoResponse {
    // Handler implementation
}
```

## Implementing Permissions Checks in Services

### Service Layer Authorization

```rust
// Authorization in a service layer
struct UserService {
    repository: UserRepository,
    auth_service: Box<dyn AuthorizationService>,
}

impl UserService {
    // Create a new user (requires write permission)
    async fn create_user(&self, current_user_id: Uuid, new_user: UserCreate) -> Result<User, Error> {
        // Check if current user has permission
        let has_permission = self.auth_service
            .has_permission(current_user_id, "user:write")
            .await?;
        
        if !has_permission {
            return Err(Error::PermissionDenied);
        }
        
        // Proceed with creation
        let user = self.repository.create(new_user).await?;
        Ok(user)
    }
    
    // Get a user by ID (requires read permission)
    async fn get_user(&self, current_user_id: Uuid, user_id: Uuid) -> Result<User, Error> {
        // Check if current user has permission
        let has_permission = self.auth_service
            .has_permission(current_user_id, "user:read")
            .await?;
        
        if !has_permission {
            return Err(Error::PermissionDenied);
        }
        
        // Proceed with retrieval
        let user = self.repository.find_by_id(user_id).await?;
        Ok(user)
    }
}
```

## Dynamic Permissions

### Permission Delegation

```rust
// Delegate permissions temporarily
async fn delegate_permission(
    delegator_id: Uuid,
    delegatee_id: Uuid,
    permission: &str,
    duration: Duration,
    auth_service: &impl AuthorizationService,
) -> Result<(), Error> {
    // Check if delegator has the permission
    let has_permission = auth_service
        .has_permission(delegator_id, permission)
        .await?;
    
    if !has_permission {
        return Err(Error::PermissionDenied);
    }
    
    // Delegate the permission
    auth_service
        .delegate_permission(delegator_id, delegatee_id, permission, duration)
        .await?;
    
    Ok(())
}
```

### Conditional Permissions

```rust
// Permission based on resource ownership
async fn can_edit_document(
    user_id: Uuid,
    document_id: Uuid,
    document_service: &DocumentService,
    auth_service: &impl AuthorizationService,
) -> Result<bool, Error> {
    // Get the document
    let document = document_service.find_by_id(document_id).await?;
    
    // Check if user is the owner
    if document.owner_id == user_id {
        return Ok(true);
    }
    
    // Check if user has global edit permission
    let has_permission = auth_service
        .has_permission(user_id, "document:edit:any")
        .await?;
    
    Ok(has_permission)
}
```

## Hierarchical Role Structure

Navius supports hierarchical roles:

```yaml
authorization:
  type: "rbac"
  hierarchical: true
  roles:
    - name: "admin"
      inherits: ["editor"]
      permissions: ["user:delete", "config:write"]
    - name: "editor"
      inherits: ["viewer"]
      permissions: ["user:write"]
    - name: "viewer"
      permissions: ["user:read", "config:read"]
```

```rust
// Implementation with hierarchical roles
let auth_service = RoleBasedAuthorization::from_config(&config)?;

// Even though a user only has "admin" role, they'll have all permissions
// from admin, editor, and viewer roles due to the hierarchy
```

## Permission Management UI

### Role Management Component

```rust
// Handler for getting all roles
async fn get_roles_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<Role>>, StatusCode> {
    let roles = state.auth_service
        .get_all_roles()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(roles))
}

// Handler for creating a new role
async fn create_role_handler(
    State(state): State<AppState>,
    Json(payload): Json<RoleCreate>,
) -> Result<Json<Role>, StatusCode> {
    let role = state.auth_service
        .create_role(payload.name, payload.permissions)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(role))
}

// Handler for updating a role
async fn update_role_handler(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    Json(payload): Json<RoleUpdate>,
) -> Result<Json<Role>, StatusCode> {
    let role = state.auth_service
        .update_role(role_name, payload.permissions)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(role))
}
```

## Testing Authorization

### Unit Testing

```rust
#[tokio::test]
async fn test_rbac_authorization() {
    // Setup test authorization service
    let mut config = Config::default();
    config.set("authorization.type", "rbac").unwrap();
    config.set("authorization.roles", vec![
        Role { name: "admin".to_string(), permissions: vec!["user:read".to_string(), "user:write".to_string()] },
        Role { name: "viewer".to_string(), permissions: vec!["user:read".to_string()] },
    ]).unwrap();
    
    let auth_service = RoleBasedAuthorization::from_config(&config).unwrap();
    
    // Assign roles
    let admin_id = Uuid::new_v4();
    let viewer_id = Uuid::new_v4();
    
    auth_service.assign_role(admin_id, "admin").await.unwrap();
    auth_service.assign_role(viewer_id, "viewer").await.unwrap();
    
    // Test permissions
    assert!(auth_service.has_permission(admin_id, "user:read").await.unwrap());
    assert!(auth_service.has_permission(admin_id, "user:write").await.unwrap());
    assert!(auth_service.has_permission(viewer_id, "user:read").await.unwrap());
    assert!(!auth_service.has_permission(viewer_id, "user:write").await.unwrap());
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_protected_routes() {
    // Setup test app with authentication and authorization
    let app = test_app().await;
    
    // Login as admin
    let admin_token = app.login("admin", "password").await;
    
    // Login as viewer
    let viewer_token = app.login("viewer", "password").await;
    
    // Test admin access
    let response = app.get("/users")
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    
    let response = app.post("/users")
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&user_payload)
        .send()
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Test viewer access
    let response = app.get("/users")
        .header("Authorization", format!("Bearer {}", viewer_token))
        .send()
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    
    let response = app.post("/users")
        .header("Authorization", format!("Bearer {}", viewer_token))
        .json(&user_payload)
        .send()
        .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
```

## Best Practices

### Principle of Least Privilege

- Assign the minimum permissions necessary
- Regularly review and remove unnecessary permissions
- Use time-limited elevated privileges when needed

### Authorization Design

- Design permissions around business operations, not technical operations
- Group related permissions into roles
- Consider the user experience when defining permission granularity
- Document the authorization model

### Auditing and Logging

```rust
// Log permission checks
async fn log_permission_check(
    user_id: Uuid,
    permission: &str,
    allowed: bool,
    auth_service: &impl AuthorizationService,
) -> Result<(), Error> {
    let timestamp = chrono::Utc::now();
    
    let audit_log = AuditLog {
        timestamp,
        user_id,
        action: "permission_check".to_string(),
        resource: permission.to_string(),
        success: allowed,
    };
    
    auth_service.log_audit_event(audit_log).await?;
    Ok(())
}
```

## Troubleshooting

### Common Issues

1. **Missing Permissions**: Verify role assignments and permission inheritance
2. **Authorization Service Misconfiguration**: Check YAML configuration for typos
3. **Middleware Order**: Authentication middleware must run before authorization
4. **Cache Invalidation**: Permission changes may be cached; implement proper invalidation

### Debugging Authorization

Enable debug logging for authorization:

```rust
// Initialize logger with auth debug enabled
tracing_subscriber::fmt()
    .with_env_filter("navius::auth::authorization=debug")
    .init();
```

## Related Resources

- [Security Best Practices](./security-best-practices.md)
- [Authentication Implementation Guide](./authentication-implementation.md)
- [API Security Guide](./api-security.md)
- [Data Protection Guide](./data-protection.md)
- [OWASP Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html) 