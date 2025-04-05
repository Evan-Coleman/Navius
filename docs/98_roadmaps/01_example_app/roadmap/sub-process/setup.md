# Example App Setup Process

## Overview
This document outlines the process of setting up the initial project structure for the example app. The setup phase focuses on creating a solid foundation for the rest of the implementation using TDD.

## Related Documents
- [Main Roadmap](../01-example-app.md)
- [Implementation Plan](../example-app-plan.md)

## Steps

### Step 1: Create Project Structure

#### Task 1.1: Initialize Project
- [ ] Create a new Rust project in the examples directory
- [ ] Initialize git repository
- [ ] Set up .gitignore file

#### Task 1.2: Configure Cargo.toml
- [ ] Add basic metadata (name, version, authors, etc.)
- [ ] Add dependencies:
  - [ ] navius-core
  - [ ] navius-http
  - [ ] navius-auth
  - [ ] navius-db
  - [ ] navius-cache
  - [ ] axum
  - [ ] tokio
  - [ ] serde
  - [ ] Development dependencies (test, clippy, etc.)

#### Task 1.3: Set Up Directory Structure
- [ ] Create src directory structure:
  - [ ] src/main.rs - Entry point
  - [ ] src/lib.rs - Library exports
  - [ ] src/core/ - Core domain logic
  - [ ] src/api/ - API endpoints
  - [ ] src/data/ - Data access
  - [ ] src/config/ - Configuration
  - [ ] src/error/ - Error handling

### Step 2: Configure Development Environment

#### Task 2.1: Set Up Configuration
- [ ] Create .env.example template
- [ ] Configure environment variables

#### Task 2.2: Docker Setup
- [ ] Create docker-compose.yml for dependencies
- [ ] Add PostgreSQL service
- [ ] Add Redis service
- [ ] Configure network and volumes

#### Task 2.3: Database Setup
- [ ] Create initial migration structure
- [ ] Set up connection configuration

### Step 3: Establish Test Infrastructure

#### Task 3.1: Configure Testing Framework
- [ ] Set up test utilities
- [ ] Configure test database connection
- [ ] Implement test helpers

#### Task 3.2: First Test
- [ ] Implement health check test using TDD
- [ ] Verify cargo build passes without errors/warnings
- [ ] Document testing approach

#### Task 3.3: CI Setup
- [ ] Configure basic CI workflow
- [ ] Set up test runner
- [ ] Configure code quality checks

### Step 4: Implement Main Application Entry Point

#### Task 4.1: Main Application Structure
- [x] Implement the main.rs file based on the provided example
- [x] Set up basic application structure with plugins:
  - [x] SqlxPlugin for database connections
  - [x] WebPlugin for web server functionality
- [x] Implement App::new() builder pattern for app initialization

#### Task 4.2: Route Implementation
- [ ] Implement route registration using macros
- [ ] Create basic endpoints:
  - [ ] Hello world endpoint
  - [ ] Path parameter example endpoint
  - [ ] Login endpoint with JWT authentication
  - [ ] Protected user info endpoint
  - [ ] SQL query endpoints

#### Task 4.3: Application Configuration
- [ ] Implement configuration system
- [ ] Create custom configuration structure
- [ ] Set up automatic configuration loading and injection
- [ ] Implement the #[auto_config] macro functionality

## Target Main Application Example

The example app should support a clean, declarative API similar to this example:

```rust
mod jwt;

use axum::http::StatusCode;
use jwt::Claims;
use serde::Deserialize;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await;

    tracing::info!("Server Shutdown")
}

#[routes]
#[get("/")]
#[get("/hello_world")]
async fn hello_world() -> impl IntoResponse {
    "hello world"
}

#[route("/hello/{name}", method = "GET", method = "POST")]
async fn hello(Path(name): Path<String>) -> impl IntoResponse {
    format!("hello {name}")
}

#[derive(Deserialize)]
struct LoginCredentials {
    username: String,
    password: String,
}

#[post("/login")]
async fn login(Json(credentials): Json<LoginCredentials>) -> Result<impl IntoResponse> {
    let LoginCredentials { username, password } = credentials;
    if username == "root" && password == "correct_password" {
        let mock_user_id = 1000;
        let jwt_token = jwt::encode(Claims::new(mock_user_id))?;
        Ok((StatusCode::OK, jwt_token))
    } else {
        Ok((
            StatusCode::BAD_REQUEST,
            format!("{username} login failed: username or password are incorrect"),
        ))
    }
}

#[derive(Configurable, Deserialize)]
#[config_prefix = "custom"]
struct CustomConfig {
    user_info_detail: String,
}

#[get("/user-info")]
async fn protected_user_info(
    claims: Claims,
    Config(conf): Config<CustomConfig>,
) -> impl IntoResponse {
    let user_id = claims.uid;
    format!("get user info of id#{}: {}", user_id, conf.user_info_detail)
}

#[nest("/sql")]
mod sql {
    use anyhow::Context;
    use std::ops::Deref;

    #[get("/version")]
    pub async fn sqlx_request_handler(Component(pool): Component<ConnectPool>) -> Result<String> {
        let version = sqlx::query("select version() as version")
            .fetch_one(&pool)
            .await
            .context("sqlx query failed")?
            .get("version");
        Ok(version)
    }

    #[get("/now")]
    pub async fn sqlx_time_handler(pool: Component<ConnectPool>) -> Result<String> {
        let time = sqlx::query("select DATE_FORMAT(now(),'%Y-%m-%d %H:%i:%s') as time")
            .fetch_one(pool.deref())
            .await
            .context("sqlx query failed")?
            .get("time");
        Ok(time)
    }
}
```

## TDD Process for Setup

For each task in the setup phase:

1. **Write Tests First**
   - Write tests that validate the expected functionality
   - Start with basic health check and configuration validation tests

2. **Implement Minimal Functionality**
   - Create just enough code to make tests pass
   - Focus on clean interfaces for later extension

3. **Verify**
   - Run `cargo build` to ensure no errors/warnings
   - Run tests with `cargo test` to ensure all tests pass

4. **Document**
   - Add inline documentation (comments, docstrings)
   - Update progress in this document

## Navius Crate Considerations

During the setup phase, document any initial observations about the Navius crates:

- Ease of integration
- Documentation completeness
- Initial friction points
- Potential enhancement areas

## Progress Tracking

| Task | Status | Notes |
|------|--------|-------|
| 1.1 Initialize Project | Completed | Project structure created |
| 1.2 Configure Cargo.toml | Completed | Added dependencies and package info |
| 1.3 Set Up Directory Structure | Completed | Created src directory with proper module structure |
| 2.1 Set Up Configuration | Not Started | |
| 2.2 Docker Setup | Not Started | |
| 2.3 Database Setup | Not Started | |
| 3.1 Configure Testing Framework | Not Started | |
| 3.2 First Test | Completed | Initial tests for App and Plugins |
| 3.3 CI Setup | Not Started | |
| 4.1 Main Application Structure | Completed | Created App builder pattern with plugin support |
| 4.2 Route Implementation | Not Started | |
| 4.3 Application Configuration | Not Started | |

## Current Status
- Status: In Progress
- Progress: 25%
- Updated at: May 30, 2024 