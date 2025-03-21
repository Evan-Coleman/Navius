# Application Auth Customization

This directory is where you can extend and customize the authentication and authorization functionality for your application. The core authentication logic is in `src/core/auth` and should not be modified directly.

## How to Use This Directory

### Customizing Authentication Requirements

You can create custom authentication layers with specific role requirements:

```rust
// Example: Custom authentication layer that requires specific roles
pub fn require_admin_role(config: &AppConfig) -> EntraAuthLayer {
    let required_roles = vec!["Admin".to_string()];
    
    EntraAuthLayer::new(
        &config.auth.entra.tenant_id,
        &config.auth.entra.audience,
        &config.auth.entra.issuer,
        required_roles,
    )
}
```

### Authentication for Downstream Services

You can create helpers for authenticating to downstream services:

```rust
// Example: Get client for a specific API
pub async fn get_api_client(config: &AppConfig) -> Result<reqwest::Client, String> {
    let token_client = EntraTokenClient::from_config(config);
    token_client.create_client("api://your-api-id/.default").await
}
```

### Using in Routes

Apply your custom authentication layers to routes in `src/app/router.rs`:

```rust
let admin_routes = Router::new()
    .route("/admin/users", get(admin_handlers::list_users))
    .layer(auth::require_admin_role(&state.config));
```

## Tips

- Keep authentication logic centralized in this directory
- Use the core components (EntraAuthLayer, EntraTokenClient) as building blocks
- Don't directly modify the core authentication implementation 