# Navius HTTP

Navius HTTP provides HTTP server and client functionality for the Navius framework.

## Features

- HTTP server based on [Axum](https://github.com/tokio-rs/axum)
- HTTP client based on [Reqwest](https://github.com/seanmonstar/reqwest)
- Common middleware for CORS, logging, request ID, and timeout
- WebPlugin for easily integrating web functionality into Navius applications

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
navius-http = { version = "0.1.0", features = ["server", "client"] }
```

### Zero-Boilerplate Web Server with WebPlugin

The WebPlugin provides a simplified way to add HTTP server capabilities to your Navius application with minimal boilerplate:

```rust
use navius_http::WebPlugin;
use axum::{Router, routing::get};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create API router
    let api_router = Router::new()
        .route("/hello", get(|| async { "Hello, world!" }));
    
    // Create a web plugin with custom settings
    let web_plugin = WebPlugin::new()
        .with_host("127.0.0.1")
        .with_port(8080)
        .with_router(Router::new().nest("/api/v1", api_router));
    
    // Build and run your application with the plugin
    navius_core::app::App::new()
        .with_plugin(web_plugin)
        .run()
        .await
}
```

### Automatic Route Discovery

For even more Zero Boilerplate development, you can use the automatic route discovery feature to define routes using simple attributes:

```rust
use navius_http::{WebPlugin, RouteDiscoveryConfig, RouteRegistry};
use navius_macros::{route, nest, navius_router};
use axum::Json;
use serde_json::json;

// Define an API module with routes
#[nest(prefix = "/api/v1")]
mod api {
    use super::*;

    #[route(path = "/hello", method = "GET")]
    async fn hello_world() -> Json<serde_json::Value> {
        Json(json!({ "message": "Hello, World!" }))
    }

    #[route(path = "/items/:id", method = ["GET", "PUT", "DELETE"])]
    async fn item_operations(axum::extract::Path(id): axum::extract::Path<String>) -> String {
        format!("Operating on item: {}", id)
    }
}

// Discover and register all routes
#[navius_router(modules = [self::api])]
fn register_routes(registry: &mut RouteRegistry) {
    // The macro will generate code to register all discovered routes
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create a route registry and register all routes
    let mut registry = RouteRegistry::new();
    register_routes(&mut registry);
    
    // Create a web plugin with the discovered routes
    let web_plugin = WebPlugin::new()
        .with_host("127.0.0.1")
        .with_port(8080)
        .with_router(registry.build_router());
    
    // Build and run your application with the plugin
    navius_core::app::App::new()
        .with_plugin(web_plugin)
        .run()
        .await
}

### Starting a Server Manually

You can also use the lower-level HTTP server API:

```rust
use navius_http::server::{HttpServerConfig};
use axum::{Router, routing::get};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a router
    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }));
    
    // Configure server
    let server = HttpServerConfig::new()
        .with_host_and_port("127.0.0.1", 8080);
    
    // Bind and serve
    let listener = server.bind_listener().await?;
    
    println!("Server started at http://{}", listener.local_addr()?);
    
    axum::serve(listener, router.into_make_service())
        .await?;
    
    Ok(())
}
```

### Making HTTP Requests

```rust
use navius_http::client::HttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = HttpClient::new()
        .with_base_url("https://api.example.com")
        .with_timeout(std::time::Duration::from_secs(30))
        .build()?;
    
    // Make a GET request
    let response = client.get("/users").send().await?;
    
    // Parse JSON response
    let users: Vec<User> = response.json().await?;
    
    println!("Found {} users", users.len());
    
    Ok(())
}
```

## Middleware

Navius HTTP includes several middleware components:

### CORS

```rust
use navius_http::middleware::{CorsConfig, cors_layer};
use axum::Router;

let cors = CorsConfig::new()
    .allow_origin("https://example.com")
    .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
    .build();

let app = Router::new()
    // ... routes ...
    .layer(cors_layer(cors));
```

### Logging

```rust
use navius_http::middleware::logging_layer;
use axum::Router;

let app = Router::new()
    // ... routes ...
    .layer(logging_layer());
```

### Request ID

```rust
use navius_http::middleware::request_id_layer;
use axum::Router;

let app = Router::new()
    // ... routes ...
    .layer(request_id_layer());
```

### Timeout

```rust
use navius_http::middleware::timeout_layer_with_duration;
use axum::Router;
use std::time::Duration;

let app = Router::new()
    // ... routes ...
    .layer(timeout_layer_with_duration(Duration::from_secs(30)));
```

## License

Copyright © Navius Technologies, Inc. All rights reserved. 