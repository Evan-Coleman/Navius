---
title: Navius Hello World Tutorial
description: A step-by-step guide to create your first Navius application
category: getting-started
tags:
  - tutorial
  - beginner
  - example
  - rest-api
  - getting-started
related:
  - installation.md
  - first-steps.md
  - development-setup.md
  - ../02_examples/rest-api-example.md
  - ../04_guides/dependency-injection.md
last_updated: March 27, 2025
version: 1.0
status: active
---

# Navius Hello World Tutorial

## Overview

This tutorial walks you through creating a simple "Hello World" REST API using the Navius framework. By the end, you'll have a functional application that demonstrates key Navius concepts including dependency injection, routing, and service architecture.

## Prerequisites

Before beginning this tutorial, ensure you have:

- Rust installed (1.70.0 or newer)
- Cargo installed
- Completed the [Installation Guide](installation.md)
- Optional: Completed the [Development Setup](development-setup.md)

## Quick Start

If you're familiar with Rust and just want the code, here's a quick overview of what we'll build:

```bash
# Create a new project
cargo new hello-navius
cd hello-navius

# Add dependencies to Cargo.toml
# [dependencies]
# navius = "0.1.0"
# tokio = { version = "1", features = ["full"] }
# axum = "0.6.0"
# serde = { version = "1.0", features = ["derive"] }

# Run the application
cargo run

# Test the API
curl http://localhost:3000/hello/World
# Result: {"message":"Hello, World"}
```

## Step-by-step Tutorial

### Step 1: Create a New Project

First, create a new Rust project using Cargo:

```bash
cargo new hello-navius
cd hello-navius
```

### Step 2: Add Dependencies

Edit your `Cargo.toml` file to add the necessary dependencies:

```toml
[package]
name = "hello-navius"
version = "0.1.0"
edition = "2021"

[dependencies]
navius = "0.1.0"
tokio = { version = "1", features = ["full"] }
axum = "0.6.0"
serde = { version = "1.0", features = ["derive"] }
```

These dependencies include:
- `navius`: The core Navius framework
- `tokio`: Asynchronous runtime for Rust
- `axum`: Web framework for building APIs
- `serde`: Serialization/deserialization library

### Step 3: Set Up the Main Service

Create a new file `src/hello_service.rs` that implements our core service:

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

This service:
- Uses an `Arc` (Atomic Reference Counting) to enable safe sharing across threads
- Stores a greeting message that can be customized
- Provides a method to generate personalized greetings

### Step 4: Implement a REST API Handler

Create a handler in `src/hello_handler.rs` to expose the service via a REST endpoint:

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

This handler:
- Uses Axum's path extraction to get the name parameter
- Accepts our `HelloService` as a dependency via Arc
- Returns a JSON response with the greeting message
- Uses Serde to serialize the response

### Step 5: Set Up Application State

Create `src/app_state.rs` to manage application state and dependencies:

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

The `AppState`:
- Acts as a container for all application services
- Initializes the `HelloService` with a default greeting
- Demonstrates basic dependency management

### Step 6: Configure Routing

Create `src/router.rs` to set up the API routes:

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

The router:
- Creates an instance of AppState
- Defines a GET route that accepts a name parameter
- Uses Axum's extension system to inject our HelloService into handlers

### Step 7: Configure the Main Application

Create the application entry point in `src/main.rs`:

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

The main function:
- Imports all our modules
- Uses the Tokio runtime for async execution
- Initializes the router
- Starts an HTTP server on localhost:3000

### Step 8: Run the Application

Run your application with Cargo:

```bash
cargo run
```

You should see output indicating the server has started:

```
Server starting on http://127.0.0.1:3000
```

### Step 9: Test the API

With the server running, test the API using curl or a web browser:

```bash
curl http://localhost:3000/hello/World
```

You should receive a JSON response:

```json
{"message":"Hello, World"}
```

Try different names to see personalized responses:

```bash
curl http://localhost:3000/hello/Navius
```

Response:

```json
{"message":"Hello, Navius"}
```

## Understanding the Code

### Project Structure

Our project follows a clean separation of concerns:

```
hello-navius/
├── Cargo.toml            # Project dependencies
└── src/
    ├── main.rs           # Application entry point
    ├── app_state.rs      # Application state management
    ├── hello_service.rs  # Business logic
    ├── hello_handler.rs  # API endpoints
    └── router.rs         # Route configuration
```

### Key Concepts

This simple example demonstrates several important Navius concepts:

1. **Service Pattern**: Business logic is encapsulated in the `HelloService`
2. **Dependency Injection**: Services are created in `AppState` and injected where needed
3. **Handler Pattern**: API endpoints are defined as handler functions
4. **Routing**: URL paths are mapped to handlers in the router

## Advanced Customization

### Adding Configuration

To make the greeting configurable, you could add a configuration file:

```rust
// src/config.rs
pub struct AppConfig {
    pub default_greeting: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_greeting: "Hello,".to_string(),
        }
    }
}
```

Then update `app_state.rs` to use this configuration:

```rust
// In app_state.rs
use crate::config::AppConfig;

pub struct AppState {
    pub hello_service: Arc<HelloService>,
    pub config: AppConfig,
}

impl AppState {
    pub fn new() -> Self {
        let config = AppConfig::default();
        let hello_service = HelloService::new(config.default_greeting.clone());
        
        Self { hello_service, config }
    }
}
```

### Adding Error Handling

For more robust error handling, you could update the service:

```rust
// Enhanced hello_service.rs
pub enum GreetingError {
    EmptyName,
}

impl HelloService {
    pub fn greet(&self, name: &str) -> Result<String, GreetingError> {
        if name.trim().is_empty() {
            return Err(GreetingError::EmptyName);
        }
        Ok(format!("{} {}", self.greeting, name))
    }
}
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| **Compilation Errors** | Ensure you have the correct versions of dependencies |
| **Server Won't Start** | Check if port 3000 is already in use |
| **404 Not Found** | Verify the URL path matches the route defined in `router.rs` |
| **Empty Response** | Check your handler logic and verify the service returns the expected data |

## Next Steps

This simple example demonstrates the basic structure of a Navius application. From here, you can:

- Add more advanced [routing patterns](../04_guides/routing.md)
- Implement proper [error handling](../04_guides/error-handling.md)
- Explore [database integration](../04_guides/database-integration.md)
- Add [authentication](../04_guides/authentication.md)
- Set up [comprehensive testing](../04_guides/testing.md)

For more sophisticated examples, check out the [Examples](../02_examples/README.md) section, particularly:

- [REST API Example](../02_examples/rest-api-example.md) for a more complete API
- [Dependency Injection Example](../02_examples/dependency-injection-example.md) for advanced DI patterns 