---
title: "Basic Application Example"
description: "A minimalist Navius application demonstrating core concepts"
category: examples
tags:
  - examples
  - getting-started
  - basic
related:
  - examples/custom-service-example.md
  - getting-started/first-steps.md
last_updated: March 27, 2025
version: 1.0
---

# Basic Application Example

This example demonstrates a minimal Navius application, showing the essential components and patterns needed to get started.

## Project Structure

```
basic-app/
├── Cargo.toml
├── config/
│   └── default.yaml
└── src/
    ├── main.rs
    ├── app/
    │   ├── mod.rs
    │   ├── api/
    │   │   ├── mod.rs
    │   │   └── hello_handler.rs
    │   ├── models/
    │   │   ├── mod.rs
    │   │   └── greeting.rs
    │   └── services/
    │       ├── mod.rs
    │       └── greeting_service.rs
    └── core/
        ├── mod.rs
        ├── config.rs
        ├── error.rs
        └── router.rs
```

## Implementation

### Core Components

#### `main.rs`

```
use std::net::SocketAddr;
use navius::core::config::load_config;
use navius::core::error::AppError;
use navius::core::router::build_router;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = load_config()?;
    
    // Create application router
    let app = build_router(config.clone())?;
    
    // Extract server configuration
    let addr: SocketAddr = config.server.address.parse()
        .map_err(|e| AppError::configuration_error(format!("Invalid address: {}", e)))?;
    
    // Start server
    tracing::info!("Starting server on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| AppError::internal_server_error(format!("Server error: {}", e)))?;
    
    Ok(())
}
```

#### `core/config.rs`

```
use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub app_name: String,
}

pub fn load_config() -> Result<Arc<AppConfig>, ConfigError> {
    let config = Config::builder()
        .add_source(File::with_name("config/default"))
        .build()?;
    
    let app_config: AppConfig = config.try_deserialize()?;
    Ok(Arc::new(app_config))
}
```

#### `core/router.rs`

```
use crate::app::api::hello_handler;
use crate::core::config::AppConfig;
use crate::core::error::AppError;
use axum::{Router, routing::get};
use std::sync::Arc;

pub fn build_router(config: Arc<AppConfig>) -> Result<Router, AppError> {
    let router = Router::new()
        .route("/", get(hello_handler::hello))
        .route("/greeting/:name", get(hello_handler::greet_user))
        .with_state(config);
    
    Ok(router)
}
```

### Application Components

#### `app/api/hello_handler.rs`

```
use crate::app::models::greeting::Greeting;
use crate::core::config::AppConfig;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

pub async fn hello(State(config): State<Arc<AppConfig>>) -> Json<Greeting> {
    Json(Greeting {
        message: format!("Hello from {}!", config.app_name),
    })
}

pub async fn greet_user(
    State(config): State<Arc<AppConfig>>,
    Path(name): Path<String>,
) -> Json<Greeting> {
    Json(Greeting {
        message: format!("Hello, {}! Welcome to {}!", name, config.app_name),
    })
}
```

#### `app/models/greeting.rs`

```
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Greeting {
    pub message: String,
}
```

#### `app/services/greeting_service.rs`

```
use crate::app::models::greeting::Greeting;

pub struct GreetingService;

impl GreetingService {
    pub fn new() -> Self {
        Self
    }
    
    pub fn create_greeting(&self, name: &str, app_name: &str) -> Greeting {
        Greeting {
            message: format!("Hello, {}! Welcome to {}!", name, app_name),
        }
    }
    
    pub fn default_greeting(&self, app_name: &str) -> Greeting {
        Greeting {
            message: format!("Hello from {}!", app_name),
        }
    }
}
```

## Configuration

### `config/default.yaml`

```
server:
  address: "127.0.0.1"
  port: 3000

app_name: "Basic Navius App"
```

## Cargo.toml

```
[package]
name = "basic-navius-app"
version = "0.1.0"
edition = "2021"

[dependencies]
navius = { path = "../.." }
axum = "0.6.20"
tokio = { version = "1.32.0", features = ["full"] }
config = "0.13.3"
serde = { version = "1.0.188", features = ["derive"] }
tracing = "0.1.37"
tracing-subscriber = "0.3.17"
```

## Running the Example

1. Clone the Navius repository
2. Navigate to the `examples/basic-app` directory
3. Run the application:

```
cargo run
```

4. Test the endpoints:

```
# Get default greeting
curl http://localhost:3000/

# Get personalized greeting
curl http://localhost:3000/greeting/YourName
```

## Key Concepts Demonstrated

1. **Application Configuration**: Loading configuration from files
2. **Router Setup**: Creating routes with Axum
3. **State Management**: Sharing application state
4. **Request Handling**: Processing requests and returning responses
5. **Error Handling**: Standardized error types and conversion
6. **Project Structure**: Core vs. App layer separation

## Next Steps

After understanding this basic example, you can explore more advanced features:

- [Custom Service Example](custom-service-example.md): Learn how to create and register custom services
- [Dependency Injection Example](dependency-injection-example.md): Explore Navius' dependency injection system
- [Configuration Example](configuration-example.md): More advanced configuration scenarios
- [Error Handling Example](error-handling-example.md): Comprehensive error handling strategies

For a complete application with database access, authentication, and more, see the [Pet Store Example](pet-store-example.md). 