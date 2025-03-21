# Core Authentication Module

This module provides the core authentication functionality for the application, with a focus on Microsoft Entra ID (formerly Azure AD) integration. The implementation is designed to be used by the application code but should not be modified directly.

## Key Components

### EntraAuthLayer

A middleware layer for validating incoming bearer tokens. This middleware can be used to protect API routes with different access levels:

- Read-only access
- Full access
- Custom role requirements

Example usage can be found in `src/app/router.rs`.

### EntraTokenClient

A client for acquiring tokens for downstream service calls. This client handles:

- Token acquisition using client credentials flow
- Token caching to reduce authentication overhead
- Creating HTTP clients with pre-configured auth headers

## How to Extend or Customize

To customize authentication for your application:

1. **DO NOT** modify files in `src/core/auth` directly
2. Create your extensions and customizations in `src/app/auth`
3. Use the core components as building blocks for your custom authentication logic

See `src/app/auth/mod.rs` for examples of how to create custom authentication components based on the core functionality.

## Authentication Flow

1. Requests come in to the application
2. The router applies the appropriate EntraAuthLayer based on the route
3. The middleware validates the bearer token against Entra ID
4. If valid, the request continues; if invalid, a 401 Unauthorized response is returned

For outgoing requests to other services:

1. The application uses the EntraTokenClient to get a token
2. The token is cached to avoid unnecessary requests
3. The token is added to the Authorization header of outgoing requests 