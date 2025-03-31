# Basic Integration Example

This example demonstrates the integration of multiple Navius crates using the dependency injection system. It showcases how to build a simple HTTP server with authentication and health checking capabilities.

## Crates Used

- `navius-core`: For dependency injection, component registry, and configuration
- `navius-http`: For HTTP server, routing, and request/response handling
- `navius-auth`: For authentication and authorization interfaces

## Key Features Demonstrated

1. **Dependency Injection**:
   - Component registration with different scopes (singleton)
   - Type-safe dependency resolution
   - Lifecycle hooks for initialization and cleanup

2. **HTTP Server Configuration**:
   - Route configuration with closures
   - Request handling and response generation
   - Error handling and status codes

3. **Authentication**:
   - Simple token-based authentication
   - Protected routes with authentication middleware
   - User information retrieval

## Running the Example

```bash
cd workspace_migration/examples/integration/basic
cargo run
```

The server will start on `127.0.0.1:8080` with the following endpoints:

- `/health`: Returns the health status of the application
- `/api/protected`: A protected endpoint that requires authentication

To access the protected endpoint, use a request with an Authorization header:

```bash
curl -H "Authorization: Bearer user1" http://127.0.0.1:8080/api/protected
```

## Key Code Concepts

- **Application Builder**: Creates and configures the application with components
- **Component Registry**: Manages component lifecycle and dependencies
- **Server Configuration**: Sets up HTTP routes and handlers
- **Auth Provider**: Implements the authentication interface

This example demonstrates how to structure a real-world application using the Navius framework, emphasizing clean separation of concerns through dependency injection and modular design. 