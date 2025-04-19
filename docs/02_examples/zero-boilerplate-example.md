---
title: "Zero Boilerplate Example"
description: "Guide to using Navius's zero-boilerplate functionality for rapid application development"
category: examples
tags:
  - zero-boilerplate
  - macros
  - routing
  - web
related:
  - ../03_tutorials/getting-started.md
  - ../05_reference/api/router-api.md
last_updated: May 30, 2024
---

# Zero Boilerplate Example

This guide demonstrates how to use Navius's zero-boilerplate functionality to rapidly build web applications with minimal code.

## Overview

The Zero Boilerplate Initiative aims to reduce user code by 90%+ for common application setups. This is achieved through:

1. **Attribute Macros**: Annotate your code with attributes for routing, dependency injection, etc.
2. **Smart Defaults**: Convention-based configuration that works out-of-the-box
3. **Auto-discovery**: Automatic component and route detection
4. **Declarative Patterns**: Focus on what your app should do, not how

## Getting Started

First, add the required dependencies to your `Cargo.toml`:

```toml
[dependencies]
navius-core = "0.1.0"
navius-http = { version = "0.1.0", features = ["server"] }
navius-macros = "0.1.0"
axum = "0.6.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Creating a Minimal Application

Here's a complete zero-boilerplate application in just a few lines of code:

```rust
use navius_macros::navius_app;

#[navius_app(name = "minimal-app")]
async fn main() {
    // That's it! Your app is ready to run.
}
```

This will create a web application with default configuration and start a server listening on `127.0.0.1:3000`.

## Adding Routes

To add routes to your application, use the `route` and `nest` macros:

```rust
use axum::Json;
use navius_macros::{navius_app, nest, route};
use serde_json::json;

#[nest(prefix = "/api/v1")]
mod api {
    use super::*;

    #[route(path = "/hello", method = "GET")]
    async fn hello() -> Json<serde_json::Value> {
        Json(json!({ "message": "Hello, world!" }))
    }
}

#[navius_app(
    name = "routes-example",
    routes = [self::api]
)]
async fn main() {
    // Your app now has an /api/v1/hello endpoint
}
```

## Advanced Routing Features

### Multiple HTTP Methods

You can handle multiple HTTP methods on a single route:

```rust
#[route(path = "/users", method = ["GET", "POST"])]
async fn users(
    // Optional payload for POST requests
    payload: Option<Json<CreateUser>>,
) -> impl IntoResponse {
    if let Some(Json(user_data)) = payload {
        // Handle POST request
        let user = User {
            id: 1337,
            username: user_data.username,
        };
        return (StatusCode::CREATED, Json(user)).into_response();
    }

    // Handle GET request
    let users = vec![
        User { id: 1, username: "alice".to_string() },
        User { id: 2, username: "bob".to_string() },
    ];
    Json(users).into_response()
}
```

### Wildcard Path Matching

You can use wildcards to match any path segment:

```rust
#[route(path = "/assets/*path", method = "GET")]
async fn serve_asset(Path(path): Path<String>) -> impl IntoResponse {
    format!("Serving asset from path: {}", path)
}
```

### Path Parameters in Nested Routes

You can capture path parameters from nested routes:

```rust
#[nest(prefix = "/admin")]
mod admin {
    use super::*;
    
    #[route(path = "/metrics/:type", method = "GET")]
    async fn metrics(Path(params): Path<HashMap<String, String>>) -> impl IntoResponse {
        let metric_type = params.get("type").unwrap_or(&"default".to_string());
        format!("Admin metrics for type: {}", metric_type)
    }
}
```

## Complete Example with All Features

Here's a comprehensive example demonstrating all the features:

```rust
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use navius_http::WebPlugin;
use navius_macros::{navius_app, nest, route};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// API module with annotated route handlers
#[nest(prefix = "/api/v1")]
mod api {
    use super::*;

    #[route(path = "/hello", method = "GET")]
    async fn hello() -> Json<serde_json::Value> {
        Json(json!({ "message": "Hello from zero-boilerplate API!" }))
    }

    #[route(path = "/users/:id", method = "GET")]
    async fn get_user(Path(id): Path<String>) -> Json<serde_json::Value> {
        Json(json!({
            "id": id,
            "name": "Sample User",
            "email": "user@example.com"
        }))
    }

    #[route(path = "/users", method = ["GET", "POST"])]
    async fn users(
        // Only used for POST requests
        // For GET requests, this will be ignored
        payload: Option<Json<CreateUser>>,
    ) -> impl IntoResponse {
        if let Some(Json(user_data)) = payload {
            // Create a new user (POST)
            let user = User {
                id: 1337,
                username: user_data.username,
            };
            return (StatusCode::CREATED, Json(user)).into_response();
        }

        // Return user list (GET)
        let users = vec![
            User { id: 1, username: "alice".to_string() },
            User { id: 2, username: "bob".to_string() },
        ];
        Json(users).into_response()
    }

    #[route(path = "/context", method = "GET")]
    async fn context_info(State(state): State<navius_http::AppState>) -> Json<serde_json::Value> {
        Json(json!({
            "components": state.app.registry().component_types().len(),
            "app_id": "zero-boilerplate-example"
        }))
    }
    
    // Example of wildcard path handling
    #[route(path = "/assets/*path", method = "GET")]
    async fn serve_asset(Path(path): Path<String>) -> impl IntoResponse {
        format!("Would serve asset: {}", path)
    }
}

/// Admin module with annotated route handlers
#[nest(prefix = "/admin")]
mod admin {
    use super::*;

    #[route(path = "/status", method = "GET")]
    async fn status() -> Json<serde_json::Value> {
        Json(json!({
            "status": "OK",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime": "10m"
        }))
    }
    
    // Capture path parameters in nested routes
    #[route(path = "/metrics/:type", method = "GET")]
    async fn metrics(Path(params): Path<HashMap<String, String>>) -> impl IntoResponse {
        let metric_type = params.get("type").unwrap_or(&"default".to_string());
        format!("Admin metrics for type: {}", metric_type)
    }
}

// Data structures for request and response
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

/// Zero-boilerplate application main function
#[navius_app(
    name = "zero-boilerplate-example",
    routes = [self::api, self::admin],
    plugins = [WebPlugin]
)]
async fn main() {
    // This is the only code you need to write - everything else is handled by the macro
    tracing::info!("Zero-boilerplate application started!");
    tracing::info!("Application ready to serve requests!");
}
```

## Understanding the Macros

### The `navius_app` Macro

The `navius_app` macro is the entry point for your application:

```rust
#[navius_app(
    name = "app-name",             // Optional app name
    routes = [mod1, mod2],         // Optional route modules
    plugins = [Plugin1, Plugin2],  // Optional plugins
    configurator = MyConfigurator  // Optional custom configurator
)]
async fn main() {
    // Your code here
}
```

### The `nest` Macro

The `nest` macro groups related routes under a common prefix:

```rust
#[nest(prefix = "/api/v1")]
mod api {
    // Route handlers go here
}
```

### The `route` Macro

The `route` macro marks functions as HTTP handlers:

```rust
#[route(path = "/users/:id", method = "GET")]
async fn get_user(Path(id): Path<String>) -> Json<serde_json::Value> {
    // Handler implementation
}

// Multiple methods
#[route(path = "/users", method = ["GET", "POST"])]
async fn users() -> &'static str {
    // Handles both GET and POST
}
```

## Configuration

By default, the application will use the following configuration sources (in order of precedence):

1. Environment variables (e.g., `NAVIUS_WEB_PORT=8080`)
2. Configuration files:
   - `./config.toml`
   - `./config/config.toml`
   - `~/.config/navius/config.toml`
   - `/etc/navius/config.toml`

Common configuration options:

```toml
# config.toml
[app]
name = "my-app"
environment = "development" # development, testing, production

[web]
host = "127.0.0.1"
port = 3000

[logging]
level = "info"
```

## Custom Initialization

You can add custom initialization code in the main function:

```rust
#[navius_app(name = "custom-init-example")]
async fn main() {
    // Initialize external services
    let db = initialize_database().await;
    
    // Set up custom state
    let state = MyState::new(db);
    
    // Register event handlers
    register_shutdown_hook();
}
```

## Plugins

Add plugins to extend functionality:

```rust
#[navius_app(
    name = "plugins-example",
    plugins = [WebPlugin, MetricsPlugin, AuthPlugin]
)]
async fn main() {
    // Application now has web, metrics, and auth functionality
}
```

## Comparison with Manual Approach

Here's a comparison of the zero-boilerplate approach versus manual configuration:

### Zero-Boilerplate (3 lines)

```rust
#[navius_app(name = "my-app", routes = [self::api])]
async fn main() {
}
```

### Manual Approach (30+ lines)

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = ConfigurationBuilder::new().build()?;
    
    // Create router
    let router = Router::new();
    
    // Add API routes
    let api_router = Router::new()
        .route("/hello", get(hello))
        .route("/users/:id", get(get_user));
    
    // Nest routers
    let router = router.nest("/api/v1", api_router);
    
    // Configure web server
    let host = config.get_string("web.host").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = config.get_number("web.port").unwrap_or(3000) as u16;
    
    // Create web plugin
    let web_plugin = WebPlugin::new()
        .with_host(host)
        .with_port(port)
        .with_router(router);
    
    // Build and run application
    let app = App::new("my-app")
        .with_plugin(web_plugin)
        .run()
        .await?;
    
    Ok(())
}
```

## Benefits

- **90% Less Code**: Eliminate boilerplate infrastructure code
- **Convention over Configuration**: Sensible defaults that work out-of-the-box  
- **Type Safety**: Full compile-time checking of routes and handlers
- **Modularity**: Organize routes into logical groups
- **Extensibility**: Easy to add custom behavior and plugins

## Advanced Topics

### Error Handling with IntoResponse

```rust
#[route(path = "/error-example", method = "GET")]
async fn error_example() -> Result<impl IntoResponse, impl IntoResponse> {
    if some_condition() {
        return Err((StatusCode::NOT_FOUND, "Resource not found").into_response());
    }
    
    Ok(Json(json!({ "status": "success" })))
}
```

### Working with Serialization

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    username: String,
    email: String,
}

#[route(path = "/users", method = "POST")]
async fn create_user(Json(payload): Json<User>) -> impl IntoResponse {
    // Create user logic here
    (StatusCode::CREATED, Json(payload))
}
```

### Stateful Applications

```rust
#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
    cache: Arc<Cache>,
}

#[navius_app(
    name = "stateful-app",
    routes = [api, admin],
)]
async fn main() {
    // Create and configure state
    let db_pool = PgPool::connect("postgres://...").await?;
    let cache = Arc::new(Cache::new());
    
    // State will be automatically passed to handlers that need it
    let state = AppState { db_pool, cache };
}
```

## Conclusion

The Zero Boilerplate Initiative dramatically reduces the amount of code needed to build and maintain Navius applications. By using declarative macros and smart conventions, you can focus on building your application's unique features rather than writing infrastructure code.

For more information, see the [Zero Boilerplate Plan](../98_roadmaps/01_example_app/implementation/zero-boilerplate-plan.md) and [Router API Reference](../05_reference/api/router-api.md). 