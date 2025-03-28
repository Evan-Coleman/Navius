---
title: First Steps with Navius
description: Guide to creating your first Navius application
category: getting-started
tags:
  - tutorial
  - quickstart
  - firstapp
related:
  - installation.md
  - development-setup.md
  - ../guides/development/development-workflow.md
last_updated: March 23, 2025
version: 1.0
---

# First Steps with Navius

## Overview
This guide walks you through creating your first Navius application. You'll learn how to set up a basic API endpoint, understand the project structure, and run your application.

## Prerequisites
Before starting this guide, ensure you have:

- Completed the [Installation Guide](installation.md)
- Set up your development environment following the [Development Setup](development-setup.md) guide
- Basic knowledge of Rust programming language
- Familiarity with RESTful APIs

## Step-by-step Tutorial

### 1. Understanding the Project Structure

First, let's explore the project structure to understand how Navius is organized:

```
navius/
├── src/             # Application source code
│   ├── app/         # Application-specific code
│   ├── core/        # Core framework components
│   ├── lib.rs       # Library entry point
│   └── main.rs      # Application entry point
├── config/          # Configuration files
├── docs/            # Documentation
└── tests/           # Tests
```

The key directories for development are:

- `src/app/` - Where you'll add your application-specific code
- `src/core/` - Core framework components (don't modify directly)
- `config/` - Configuration files for your application

### 2. Creating Your First Endpoint

Let's create a simple "hello world" endpoint:

1. **Create a new module in the app directory**

   Create the file `src/app/hello/mod.rs`:

   ```rust
   use axum::{routing::get, Router};
   
   pub fn routes() -> Router {
       Router::new().route("/hello", get(hello_handler))
   }
   
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

2. **Register the module**

   Update `src/app/mod.rs` to include your new module:

   ```rust
   pub mod hello;
   // ... other existing modules
   
   // ... existing code
   ```

3. **Add the route to the application router**

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

### 3. Running Your Application

Now that you've created your first endpoint, let's run the application:

```bash
./run_dev.sh
```

This will start the server on http://localhost:3000.

### 4. Testing Your Endpoint

Test your new endpoint with:

```bash
curl http://localhost:3000/hello
```

You should see the response: `Hello, Navius!`

You can also run the unit test you created:

```bash
cargo test --package navius --lib -- app::hello::tests::test_hello_endpoint
```

### 5. Adding Configuration

Let's modify our endpoint to use configuration values:

1. **Update the configuration file**

   Add to `config/default.yaml`:

   ```yaml
   greeting:
     message: "Hello, Navius!"
     language: "en"
   ```

2. **Use the configuration in your handler**

   Update `src/app/hello/mod.rs`:

   ```rust
   use axum::{routing::get, Router, extract::State};
   use crate::core::config::AppConfig;
   
   pub fn routes() -> Router {
       Router::new().route("/hello", get(hello_handler))
   }
   
   async fn hello_handler(State(config): State<AppConfig>) -> String {
       config.get_string("greeting.message").unwrap_or_else(|_| "Hello, Navius!".to_string())
   }
   
   // ... existing test code
   ```

3. **Run the application with the updated configuration**

   ```bash
   ./run_dev.sh
   ```

## Next Steps

Now that you've created your first Navius application, here are some next steps to explore:

1. **Create a REST API** - Expand your application with CRUD operations
2. **Add Database Integration** - Connect to PostgreSQL using the database features
3. **Implement Authentication** - Add authentication to your API endpoints
4. **Explore Middleware** - Add request logging, error handling, and other middleware
5. **Write Tests** - Expand your test coverage with more unit and integration tests

## Related Documents

- [Development Workflow](../guides/development/development-workflow.md) - Understanding the development process
- [API Integration](../guides/features/api-integration.md) - Building API endpoints
- [Testing Guide](../guides/development/testing.md) - Writing tests for your application
- [Project Structure](../reference/architecture/project-structure.md) - Detailed project structure reference 