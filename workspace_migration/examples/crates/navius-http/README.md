# Navius HTTP

HTTP server and client functionality for the Navius framework.

## Overview

`navius-http` provides HTTP server and client functionality for the Navius framework, built on top of Axum and Reqwest. It includes:

- HTTP server based on Axum with middleware support
- HTTP client with configurable options
- Common HTTP utilities and extensions
- Request and response handling
- Middleware implementations

## Features

- `server`: Enables HTTP server functionality (default)
- `client`: Enables HTTP client functionality (default)
- `test-utils`: Provides testing utilities for HTTP components

## Usage

Add the crate to your dependencies:

```toml
[dependencies]
navius-http = { version = "0.1.0" }
```

## Server Example

```rust
use navius_http::server::{RouterBuilder, HttpServer};

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a router
    let router = RouterBuilder::new()
        .add_route("/", get(|| async { "Hello, World!" }))
        .build();

    // Create and start the server
    let server = HttpServer::new()
        .with_router(router)
        .bind("127.0.0.1:3000")
        .await?;

    // Wait for the server to complete
    server.await?;
    
    Ok(())
}
```

## Client Example

```rust
use navius_http::client::HttpClient;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = HttpClient::new()
        .with_timeout(std::time::Duration::from_secs(30))
        .build();

    // Make a request
    let response = client.get("https://api.example.com/users")
        .send()
        .await?;
    
    // Parse the response
    let users: Vec<User> = response.json().await?;
    
    Ok(())
}
```

## License

Apache License 2.0 