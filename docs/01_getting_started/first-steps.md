---
title: First Steps with Navius
description: Guide to creating your first Navius application and understanding key concepts
category: getting-started
tags:
  - tutorial
  - quickstart
  - firstapp
  - endpoints
  - configuration
related:
  - installation.md
  - development-setup.md
  - hello-world.md
  - ../04_guides/development/development-workflow.md
  - ../02_examples/rest-api-example.md
last_updated: March 28, 2025
version: 1.1
status: active
---

# First Steps with Navius

## Overview

This guide walks you through creating your first Navius application. You'll learn how to set up a basic API endpoint, understand the project structure, configure the application, and run your first tests. By the end, you'll have a solid foundation for building more complex applications with Navius.

## Prerequisites

Before starting this guide, ensure you have:

- Completed the [Installation Guide](./)
- Set up your development environment following the [Development Setup](./) guide
- Basic knowledge of Rust programming language
- Familiarity with RESTful APIs
- A terminal or command prompt open in your Navius project directory

## Installation

To install the components needed for this guide:

```bash
# Clone the Navius repository
git clone https://github.com/your-organization/navius.git
cd navius

# Install dependencies
cargo build
```

For the complete installation process, refer to the [Installation Guide](./).

## Configuration

Configure the application with the following settings:

```yaml
# config/default.yaml - Base configuration file
server:
  port: 3000
  host: "127.0.0.1"
  timeout_seconds: 30
  
logging:
  level: "info"
  format: "json"
```

Key configuration options:
- Environment variables can override any configuration value
- `config/development.yaml` contains development-specific settings
- `config/production.yaml` contains production-specific settings
- Create a `.env` file for local environment variables

For more detailed configuration information, see the [Configuration Guide](./).

## Quick Start

For experienced developers who want to get started quickly:

```bash
# Clone the Navius repository (if not already done)
git clone https://github.com/your-organization/navius.git
cd navius

# Create a new module for your endpoint
mkdir -p src/app/hello
touch src/app/hello/mod.rs

# Add your endpoint code (see Section 2 below)
# Register your module in src/app/mod.rs
# Add your routes to src/app/router.rs

# Run the application
./run_dev.sh

# Test your endpoint
curl http://localhost:3000/hello
```

## 1. Understanding the Project Structure

Let's explore the Navius project structure to understand how the framework is organized:

```
navius/
├── src/                 # Application source code
│   ├── app/             # Application-specific code
│   │   ├── controllers/ # Request handlers
│   │   ├── models/      # Data models
│   │   ├── services/    # Business logic
│   │   ├── router.rs    # Route definitions
│   │   └── mod.rs       # Module exports
│   ├── core/            # Core framework components
│   │   ├── config/      # Configuration handling
│   │   ├── error/       # Error handling
│   │   ├── logging/     # Logging functionality
│   │   └── server/      # Server implementation
│   ├── lib.rs           # Library entry point
│   └── main.rs          # Application entry point
├── config/              # Configuration files
│   ├── default.yaml     # Default configuration
│   ├── development.yaml # Development environment config
│   └── production.yaml  # Production environment config
├── tests/               # Integration tests
├── docs/                # Documentation
└── .devtools/           # Development tools and scripts
```

### Key Directories

The most important directories for your development work are:

| Directory | Purpose |
|-----------|---------|
| `src/app/` | Where you'll add your application-specific code |
| `src/core/` | Core framework components (generally don't modify directly) |
| `config/` | Configuration files for your application |
| `tests/` | Integration tests for your application |

### Navius Architecture

Navius follows a layered architecture:

1. **Router Layer** - Defines HTTP routes and connects them to controllers
2. **Controller Layer** - Handles HTTP requests and responses
3. **Service Layer** - Contains business logic
4. **Repository Layer** - Interfaces with data storage

This separation of concerns makes your code more maintainable and testable.

## 2. Creating Your First Endpoint

Let's create a simple "hello world" API endpoint:

### Step 1: Create a New Module

Create a file at `src/app/hello/mod.rs` with the following content:

```rust
use axum::{routing::get, Router};

/// Returns the routes for the hello module
pub fn routes() -> Router {
    Router::new().route("/hello", get(hello_handler))
}

/// Handler for the hello endpoint
async fn hello_handler() -> String {
    "Hello, Navius!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_hello_endpoint() {
        // Arrange
        let app = routes();
        let request = Request::builder()
            .uri("/hello")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        
        // Act
        let response = app.oneshot(request).await.unwrap();
        
        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, "Hello, Navius!");
    }
}
```

This code:
- Creates a route that responds to GET requests at `/hello`
- Sets up a simple handler that returns a greeting message
- Includes a test to verify the endpoint works correctly

### Step 2: Register the Module

Update `src/app/mod.rs` to include your new module:

```rust
pub mod hello;
// ... other existing modules

// ... existing code
```

### Step 3: Add the Route to the Application Router

Update `src/app/router.rs` to include your hello routes:

```rust
use crate::app::hello;
// ... other existing imports

pub fn app_router() -> Router {
    Router::new()
        // ... existing routes
        .merge(hello::routes())
}
```

## 3. Running Your Application

Now that you've created your first endpoint, let's run the application:

```bash
./run_dev.sh
```

This will:
1. Compile your code
2. Start the server on http://localhost:3000
3. Enable hot reloading if you used the `--watch` option

You should see output similar to:

```
[INFO] Navius starting in development mode...
[INFO] Server listening on 127.0.0.1:3000
```

## 4. Testing Your Endpoint

### Manual Testing

You can test your new endpoint using `curl` from the command line:

```bash
curl http://localhost:3000/hello
```

You should see the response:
```
Hello, Navius!
```

Or using a browser, navigate to:
```
http://localhost:3000/hello
```

### Automated Testing

You can run the unit test you created:

```bash
cargo test --package navius --lib -- app::hello::tests::test_hello_endpoint
```

Or run all tests:

```bash
cargo test
```

## 5. Working with Configuration

Now, let's modify our endpoint to use configuration values, demonstrating how to work with Navius's configuration system.

### Step 1: Update the Configuration File

Edit `config/default.yaml` to add our greeting configuration:

```yaml
# ... existing configuration

# Custom greeting configuration
greeting:
  message: "Hello, Navius!"
  language: "en"
  options:
    - "Welcome"
    - "Greetings"
    - "Hey there"
```

### Step 2: Use the Configuration in Your Handler

Update `src/app/hello/mod.rs`:

```rust
use axum::{routing::get, Router, extract::State};
use crate::core::config::AppConfig;
use std::sync::Arc;

/// Routes for the hello module
pub fn routes() -> Router {
    Router::new()
        .route("/hello", get(hello_handler))
        .route("/hello/:name", get(hello_name_handler))
}

/// Handler for the basic hello endpoint
async fn hello_handler(State(config): State<Arc<AppConfig>>) -> String {
    config.get_string("greeting.message")
        .unwrap_or_else(|_| "Hello, Navius!".to_string())
}

/// Handler that greets a specific name
async fn hello_name_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
    State(config): State<Arc<AppConfig>>
) -> String {
    let greeting = config.get_string("greeting.message")
        .unwrap_or_else(|_| "Hello".to_string());
    
    format!("{}, {}!", greeting, name)
}

// ... existing test code
```

### Step 3: Run with the Updated Configuration

```bash
./run_dev.sh
```

Now test both endpoints:

```bash
curl http://localhost:3000/hello
curl http://localhost:3000/hello/Developer
```

## 6. Adding a JSON Response

Let's enhance our endpoint to return a JSON response, which is common in modern APIs.

### Step 1: Update Your Handler with JSON Support

Modify `src/app/hello/mod.rs`:

```rust
use axum::{routing::get, Router, extract::State, Json};
use serde::{Serialize, Deserialize};
use crate::core::config::AppConfig;
use std::sync::Arc;

/// Response model for greetings
#[derive(Serialize)]
struct GreetingResponse {
    message: String,
    timestamp: String,
}

/// Routes for the hello module
pub fn routes() -> Router {
    Router::new()
        .route("/hello", get(hello_handler))
        .route("/hello/:name", get(hello_name_handler))
        .route("/hello-json/:name", get(hello_json_handler))
}

/// Handler for the basic hello endpoint
async fn hello_handler(State(config): State<Arc<AppConfig>>) -> String {
    config.get_string("greeting.message")
        .unwrap_or_else(|_| "Hello, Navius!".to_string())
}

/// Handler that greets a specific name
async fn hello_name_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
    State(config): State<Arc<AppConfig>>
) -> String {
    let greeting = config.get_string("greeting.message")
        .unwrap_or_else(|_| "Hello".to_string());
    
    format!("{}, {}!", greeting, name)
}

/// Handler that returns a JSON greeting
async fn hello_json_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
    State(config): State<Arc<AppConfig>>
) -> Json<GreetingResponse> {
    let greeting = config.get_string("greeting.message")
        .unwrap_or_else(|_| "Hello".to_string());
    
    let now = chrono::Local::now().to_rfc3339();
    
    Json(GreetingResponse {
        message: format!("{}, {}!", greeting, name),
        timestamp: now,
    })
}

// ... existing test code
```

### Step 2: Test the JSON Endpoint

```bash
curl http://localhost:3000/hello-json/Developer
```

You should receive a JSON response:

```json
{
  "message": "Hello, Developer!",
  "timestamp": "2025-03-27T12:34:56.789-07:00"
}
```

## 7. Understanding Dependency Injection

Navius uses dependency injection to make services available to your handlers. Let's create a simple service and inject it.

### Step 1: Create a Greeting Service

Create a new file `src/app/hello/service.rs`:

```rust
/// Service for generating greetings
pub struct GreetingService {
    default_greeting: String,
}

impl GreetingService {
    /// Create a new GreetingService
    pub fn new(default_greeting: String) -> Self {
        Self { default_greeting }
    }
    
    /// Generate a greeting for the given name
    pub fn greet(&self, name: &str) -> String {
        format!("{}, {}!", self.default_greeting, name)
    }
    
    /// Get a formal greeting
    pub fn formal_greeting(&self, name: &str) -> String {
        format!("Greetings, {} - welcome to Navius!", name)
    }
}
```

### Step 2: Update Your Module to Use the Service

Modify `src/app/hello/mod.rs`:

```rust
mod service;

use service::GreetingService;
use axum::{routing::get, Router, extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

/// Create a router with the greeting service
pub fn routes() -> Router {
    let greeting_service = Arc::new(GreetingService::new("Hello".to_string()));
    
    Router::new()
        .route("/hello-service/:name", get(hello_service_handler))
        .layer(axum::extract::Extension(greeting_service))
        // ... other routes
}

/// Handler that uses the greeting service
async fn hello_service_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
    axum::extract::Extension(service): axum::extract::Extension<Arc<GreetingService>>
) -> String {
    service.greet(&name)
}

// ... other handlers and tests
```

## Key Concepts

### The Navius Way

Navius encourages certain patterns and practices:

1. **Modularity** - Organize code by feature in dedicated modules
2. **Separation of Concerns** - Keep routing, handlers, and business logic separate
3. **Configuration-Driven** - Use configuration files to control behavior
4. **Test-First Development** - Write tests alongside your code
5. **Dependency Injection** - Use DI for loose coupling and testability

### Common Components

These are components you'll work with frequently:

| Component | Purpose | Location |
|-----------|---------|----------|
| Router | Define HTTP routes | `src/app/router.rs` |
| Handlers | Process HTTP requests | Feature modules |
| Services | Implement business logic | Feature modules |
| Models | Define data structures | `src/app/models/` |
| Config | Application configuration | `config/` directory |

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Compilation errors | Check for typos and imports; make sure dependencies are declared |
| Route not found | Verify route is registered in the router with exact path |
| Configuration not loading | Check YAML syntax and path used in config.get_* calls |
| Test failures | Check test expectations match actual implementation |

## Next Steps

Now that you've created your first Navius application, here are some next steps to explore:

1. **Build a Complete API** - Expand your application with CRUD operations
2. **Add Database Integration** - Connect to PostgreSQL using the database features
3. **Implement Authentication** - Add authentication to secure your API endpoints
4. **Explore Middleware** - Add request logging, error handling, and other middleware
5. **Write More Tests** - Expand your test coverage with integration tests

## Related Documents

- [Hello World Tutorial](./) - A more focused tutorial on building a simple application
- [Development Setup](./) - Setting up your development environment
- [Development Workflow](./) - Understanding the development process
- [REST API Example](./) - Building a complete REST API
- [Testing Guide](../04_guides/development/testing.md) - Writing comprehensive tests 