# navius-auth-entra

Microsoft Entra ID authentication provider for the Navius framework.

## Overview

This crate provides an implementation of the `AuthProvider` trait from the `navius-auth` crate, allowing 
authentication and authorization using Microsoft Entra ID (formerly Azure AD) in Navius applications.

## Features

- JWT token validation with automatic JWKS key rotation
- Role-based authorization
- User profile extraction from token claims
- Configurable token validation parameters
- Flexible configuration options through builder pattern
- Comprehensive error handling

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
navius-auth-entra = { path = "../path/to/navius-auth-entra" }
```

## Basic Usage

```rust
use navius_auth::AuthProvider;
use navius_auth_entra::{EntraConfigBuilder, EntraProvider};

async fn authenticate_user(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create a provider with the builder pattern
    let provider = EntraProvider::builder()
        .client_id("your-client-id")
        .tenant_id("your-tenant-id")
        .build()?
        .new()?;
    
    // Authenticate the user token
    let auth_info = provider.authenticate(token).await?;
    
    println!("Authenticated user: {}", auth_info.user_id);
    println!("User roles: {:?}", auth_info.roles);
    
    Ok(())
}
```

## Configuration

The provider can be configured using the builder pattern:

```rust
let config = EntraConfigBuilder::new()
    .client_id("your-client-id")
    .tenant_id("your-tenant-id")
    .issuer("https://login.microsoftonline.com/your-tenant-id/v2.0")
    .jwks_uri("https://login.microsoftonline.com/your-tenant-id/discovery/v2.0/keys")
    .audience(vec!["api://your-app-id".to_string()])
    .jwks_cache_duration(std::time::Duration::from_secs(3600))
    .jwks_refresh_ahead_duration(std::time::Duration::from_secs(300))
    .clock_skew(std::time::Duration::from_secs(60))
    .build()?;
```

## Authorization

The provider implements the `authorize` method from the `AuthProvider` trait:

```rust
let auth_context = navius_auth::AuthorizationContext {
    user_id: "user123".to_string(),
    user_roles: vec!["User".to_string(), "Editor".to_string()],
    required_roles: vec!["Admin".to_string()],
    resource: Some("document/123".to_string()),
    action: Some("delete".to_string()),
};

let is_authorized = provider.authorize(&auth_context).await?;
if is_authorized {
    // User is authorized, proceed with the action
} else {
    // User is not authorized, deny access
}
```

## Examples

See the `examples` directory for more detailed examples.

## License

This project is licensed under the MIT License. 