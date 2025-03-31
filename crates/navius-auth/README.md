# Navius Auth

Authentication and authorization framework for the Navius platform.

## Overview

Navius Auth provides a flexible authentication and authorization system that can be integrated with various authentication providers. It includes:

- Authentication providers (Basic, JWT)
- Authorization mechanisms based on roles and permissions
- Token management
- Built-in user identity handling
- Integration with HTTP servers through middleware

## Features

Navius Auth is feature-flagged to allow for flexible configuration:

- `basic` - Basic authentication provider with username/password support (enabled by default)
- `jwt` - JWT token authentication provider (enabled by default)
- `oauth` - OAuth2 authentication support for third-party providers
- `http` - HTTP integration features like middleware (enabled by default)
- `test-utils` - Testing utilities

## Usage

### Basic Authentication

The basic authentication provider uses username/password credentials for authentication:

```rust
use navius_auth::{
    basic::{BasicProvider, BasicProviderConfig, MockUser},
    AuthProvider, Credentials, Error,
};
use std::sync::Arc;

async fn authenticate_user() -> Result<(), Error> {
    // Create a basic auth provider with mock users for testing
    let config = BasicProviderConfig {
        secret_key: "example-secret-key".to_string(),
        token_expiry: 3600, // 1 hour
        hash_passwords: false, // For production use, set to true
        mock_users: vec![
            MockUser {
                id: "user-1".to_string(),
                username: "user".to_string(),
                password: "password".to_string(),
                display_name: Some("Regular User".to_string()),
                email: Some("user@example.com".to_string()),
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
            },
        ],
    };

    let provider = BasicProvider::new("basic-provider".to_string(), config);
    let provider_arc = Arc::new(provider) as Arc<dyn AuthProvider>;

    // Create credentials
    let credentials = Credentials {
        username: "user".to_string(),
        password: "password".to_string(),
    };

    // Authenticate
    let identity = provider_arc.authenticate(&credentials).await?;
    println!("Authentication successful for user: {}", identity.username);

    Ok(())
}
```

### JWT Authentication

JWT authentication provides token-based authentication:

```rust
use navius_auth::{
    jwt::{JWTProvider, JWTProviderConfig},
    AuthProvider, Credentials, Error, Subject,
};
use std::sync::Arc;

async fn jwt_authentication() -> Result<(), Error> {
    // Create JWT provider configuration
    let config = JWTProviderConfig {
        secret_key: "your-jwt-secret-key".to_string(),
        token_expiry: 3600, // 1 hour
        issuer: Some("navius-app".to_string()),
        audience: Some("users".to_string()),
    };

    let provider = JWTProvider::new("jwt-provider".to_string(), config);
    let provider_arc = Arc::new(provider) as Arc<dyn AuthProvider>;

    // Create a subject for token generation
    let subject = Subject {
        id: "user-123".to_string(),
        subject_type: "user".to_string(),
        name: "John Doe".to_string(),
        roles: vec![/* roles here */],
        attributes: None,
    };

    // Generate a token
    let token = provider_arc.create_token(&subject).await?;
    println!("Generated token: {}", token);

    // Validate the token
    let validated_subject = provider_arc.validate_token(&token).await?;
    println!("Token validation successful for: {}", validated_subject.name);

    Ok(())
}
```

### Authorization with Role-Based Access Control

The `AuthChecker` provides role-based access control:

```rust
use navius_auth::{
    AuthProvider, middleware::AuthChecker, Error,
};
use std::sync::Arc;

async fn check_authorization(
    provider: Arc<dyn AuthProvider>,
    token: &str
) -> Result<(), Error> {
    // Create an auth checker that requires the 'admin' role
    let admin_checker = AuthChecker::new(Arc::clone(&provider))
        .require_roles(vec!["admin".to_string()])
        .require_all_roles(true);

    // Check authorization
    match admin_checker.check_authorization(token).await {
        Ok(subject) => {
            println!("User is authorized with admin role: {}", subject.name);
            Ok(())
        },
        Err(e) => {
            println!("Authorization failed: {}", e);
            Err(e)
        }
    }
}
```

## HTTP Integration

When the `http` feature is enabled, Navius Auth provides middleware components for integrating with HTTP servers:

```rust
use navius_auth::{
    AuthProvider, middleware::{AuthLayer, AuthConfig},
};
use std::sync::Arc;

// Create authentication middleware
let auth_middleware = AuthLayer::with_config(
    provider_arc,
    AuthConfig {
        required: true,
        header_name: "Authorization".to_string(),
        token_prefix: "Bearer".to_string(),
        include_identity: true,
        exempt_paths: vec![
            "/api/login".to_string(),
            "/api/public".to_string(),
        ],
    }
);

// Configure with your HTTP server
// Example with Axum:
let app = Router::new()
    .route("/api/protected", get(protected_handler))
    .layer(auth_middleware);
```

## Error Handling

Navius Auth provides a comprehensive error system with appropriate status codes and error messages:

```rust
use navius_auth::Error;

// Create specific errors
let auth_error = Error::authentication_failed("Invalid credentials");
let token_error = Error::token_expired();
let authorization_error = Error::authorization_failed("Insufficient permissions");

// Check error types
if auth_error.is_authentication_error() {
    println!("Authentication error: {}", auth_error);
}

// Get HTTP status code
let status = auth_error.status_code(); // 401
```

## License

This crate is licensed under the terms of the MIT License or the Apache License 2.0, at your choice. 