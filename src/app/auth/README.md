# Application Authentication

This directory contains user-facing authentication functionality for the Navius application. Use this module to extend and customize the core authentication system for your specific needs.

## Usage

To use the authentication functionality in your application code:

```rust
use crate::app::auth;
use crate::config::get_config;

// Create a custom auth layer for a specific route
async fn setup_auth() {
    let config = get_config();
    let custom_auth = auth::client::create_custom_auth_layer(&config);
    
    // Use in a router
    let router = axum::Router::new()
        .route("/protected", get(protected_handler))
        .layer(custom_auth);
}

// Create an authenticated client for a downstream service
async fn call_downstream_service() -> Result<String> {
    let config = get_config();
    let client = auth::client::create_service_client(&config, "service1").await?;
    
    let response = client.get("https://service1.example.com/api/data")
        .send()
        .await?
        .text()
        .await?;
        
    Ok(response)
}
```

## Extending Authentication

1. Create custom authentication middleware for specific roles or claims:

```rust
// src/app/auth/middleware.rs
use crate::core::auth::{EntraAuthLayer, UserClaims};

pub fn require_admin_role() -> axum::middleware::from_fn(
    |request: Request, next: Next| async move {
        let user = request.extensions().get::<UserClaims>()
            .ok_or_else(|| AppError::unauthorized("No user claims found"))?;
            
        if !user.roles.contains("Admin") {
            return Err(AppError::forbidden("Admin role required"));
        }
        
        Ok(next.run(request).await)
    }
);
```

2. Create service-specific authentication utilities:

```rust
// src/app/auth/services.rs
pub async fn get_authenticated_resource(resource_id: &str) -> Result<Resource> {
    let config = get_config();
    let client = create_service_client(&config, "resource-service").await?;
    
    let resource: Resource = client.get(&format!("/api/resources/{}", resource_id))
        .send()
        .await?
        .json()
        .await?;
        
    Ok(resource)
}
```

## Best Practices

1. Keep authentication logic separate from business logic
2. Use role-based access control (RBAC) for authorization decisions
3. Create reusable authentication middleware for common patterns
4. Log authentication failures with appropriate detail
5. Test both successful and unsuccessful authentication scenarios
6. Use environment-specific configurations for auth settings
7. Never hardcode secrets in authentication code

## Core Authentication System

The core authentication system is provided by `crate::core::auth` and includes:

- Microsoft Entra ID integration
- JWT token validation
- Role-based access control
- Authentication middleware for Axum

Do not modify the core authentication system directly. Instead, use this directory to extend and customize authentication for your specific application needs. 