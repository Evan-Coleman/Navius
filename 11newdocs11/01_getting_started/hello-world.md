# Hello World Example

This guide walks through creating a simple "Hello World" application using Navius, demonstrating the basic framework features and patterns.

## Prerequisites

Before beginning, ensure you have:
- Rust installed (1.65 or newer)
- Cargo installed
- Completed the [Installation](installation.md) guide

## Step 1: Create a New Project

First, create a new Rust project using Cargo:

```bash
cargo new hello-navius
cd hello-navius
```

## Step 2: Add Dependencies

Add Navius dependencies to your `Cargo.toml`:

```toml
[dependencies]
navius = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Step 3: Set Up the Main Service

Create a simple service that will provide the "Hello World" functionality:

```rust
// src/hello_service.rs
use std::sync::Arc;

pub struct HelloService {
    greeting: String,
}

impl HelloService {
    pub fn new(greeting: String) -> Arc<Self> {
        Arc::new(Self { greeting })
    }
    
    pub fn greet(&self, name: &str) -> String {
        format!("{} {}", self.greeting, name)
    }
}
```

## Step 4: Implement a Simple API Handler

Create a handler to expose the service via a REST API:

```rust
// src/hello_handler.rs
use axum::{extract::Path, response::Json};
use serde::Serialize;
use std::sync::Arc;

use crate::hello_service::HelloService;

#[derive(Serialize)]
pub struct GreetingResponse {
    message: String,
}

pub async fn greet_handler(
    Path(name): Path<String>,
    service: Arc<HelloService>,
) -> Json<GreetingResponse> {
    let message = service.greet(&name);
    Json(GreetingResponse { message })
}
```

## Step 5: Set Up Application State

Configure the application state to provide dependency injection:

```rust
// src/app_state.rs
use std::sync::Arc;

use crate::hello_service::HelloService;

pub struct AppState {
    pub hello_service: Arc<HelloService>,
}

impl AppState {
    pub fn new() -> Self {
        let hello_service = HelloService::new("Hello,".to_string());
        
        Self { hello_service }
    }
}
```

## Step 6: Configure Routing

Create the router with the hello endpoint:

```rust
// src/router.rs
use axum::{
    routing::get,
    Router,
    Extension,
};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::hello_handler::greet_handler;

pub fn app_router() -> Router {
    let app_state = Arc::new(AppState::new());
    
    Router::new()
        .route("/hello/:name", get(greet_handler))
        .layer(Extension(app_state.hello_service.clone()))
}
```

## Step 7: Configure the Main Application

Create the main application entry point:

```rust
// src/main.rs
mod app_state;
mod hello_handler;
mod hello_service;
mod router;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Initialize router
    let app = router::app_router();
    
    // Set up the address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    
    // Start the server
    println!("Server starting on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## Step 8: Run the Application

Run your application with cargo:

```bash
cargo run
```

## Step 9: Test the API

Once the application is running, you can test it using curl:

```bash
curl http://localhost:3000/hello/World
```

You should receive a response like:

```json
{"message":"Hello, World"}
```

## What's Next

This simple example demonstrates the basic structure of a Navius application. From here, you can:

- Explore more complex [service registration](../guides/service-registration.md)
- Learn about [configuration](../guides/configuration.md) options
- Add [error handling](../guides/error-handling.md)
- Implement [dependency injection](../guides/dependency-injection.md) patterns
- Set up [testing](../guides/testing.md) for your services

For more sophisticated examples, check out the [Examples](../examples/README.md) section. 