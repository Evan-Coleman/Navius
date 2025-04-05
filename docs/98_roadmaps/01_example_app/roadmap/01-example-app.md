# 01 - Example App Roadmap

## Overview
This roadmap details the process of building an example application using the Navius crate ecosystem. The app will be developed using Test-Driven Development (TDD) methodology, with incremental feature implementation and continuous verification.

## Timeline and Milestones

### Phase 1: Project Setup (Week 1)
- [x] Create example app project structure
- [x] Configure Cargo.toml with initial dependencies
- [ ] Set up basic project documentation
- [x] Establish test infrastructure
- [ ] Verify initial build with no errors/warnings

### Phase 2: Core Application Components (Weeks 2-3)
- [ ] Design core domain model using TDD
- [ ] Implement service layer with dependency injection
- [ ] Set up configuration management
- [ ] Implement error handling strategy
- [ ] Add logging and observability
- [x] Enhance navius-core crate as needed

### Phase 3: Data Layer (Weeks 4-5)
- [  ] Implement database integration using navius-db
- [  ] Create repository pattern implementation
- [  ] Add caching mechanism using navius-cache
- [  ] Implement data validation
- [  ] Enhance navius-db and navius-cache crates as needed

### Phase 4: API Development (Weeks 6-7)
- [  ] Design RESTful API using TDD
- [  ] Implement authentication using navius-auth
- [  ] Create API endpoints with proper error handling
- [  ] Implement request validation
- [  ] Add rate limiting and security features
- [  ] Enhance navius-http and navius-auth crates as needed

### Phase 5: Advanced Features (Weeks 8-9)
- [  ] Implement background job processing using navius-job
- [  ] Add event handling using navius-event
- [  ] Implement messaging with navius-messaging
- [  ] Set up metrics with navius-metrics
- [  ] Enhance related navius crates as needed

### Phase 6: Testing and Documentation (Weeks 10-11)
- [  ] Ensure comprehensive test coverage
- [  ] Document architectural decisions
- [  ] Create usage examples
- [  ] Polish and finalize application
- [  ] Prepare summary of enhancements to navius crates

## Target Example Application Style

The example app aims to achieve a clean, declarative style using attribute macros, as shown in the following `main.rs` excerpt:

```rust
mod jwt;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};
use jwt::Claims;
use serde::Deserialize;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .run()
        .await;

    tracing::info!("Server Shutdown")
}

// Example using the unified route macro
#[route(path: "/hello/{name}", method: ["GET", "POST"])]
async fn hello(Path(name): Path<String>) -> impl IntoResponse {
    format!("hello {name}")
}

// Example with authentication policy
#[route(path: "/auth/hello/{name}", method: ["GET", "POST"], auth_policy: "basic_required")]
async fn auth_hello(Path(name): Path<String>) -> impl IntoResponse {
    format!("Authenticated hello {name}")
}

// Example using module-level nesting
#[nest("/api/v1", middleware = [request_logging])]
mod api_v1 {
    use super::*;

    #[route(path: "/status", method: "GET")]
    async fn status() -> impl IntoResponse {
        "OK"
    }
    // ... other v1 routes
}

// Additional example endpoints for login, user info, and database queries using #[route]
```

To support this style, we'll need to develop several enhancements to the Navius crates, including:

1.  **Route Macros (`#[route]`, `#[nest]`)** - For declarative route definition (NC-2).
2.  **Configuration System Macros (`#[auto_config]`, `#[derive(Configurable)]`)** - For automatic configuration loading and component setup (NC-3).
3.  **Dependency Injection Improvements (`Component<T>`)** - For simplified component injection (NC-4).
    *Note: Plugin System (NC-1) was initially listed but direct ApplicationBuilder usage is preferred for this example's core setup.* 

See the [Crate Enhancements](./sub-process/crate-enhancements.md) document for detailed tracking of these enhancements.

## Development Approach

For each feature, the following TDD approach will be strictly followed:

1. **Write Tests First**
   - Write unit tests that define the expected behavior
   - Ensure tests fail initially (Red phase)

2. **Implement Functionality**
   - Implement minimal code to make tests pass (Green phase)
   - Run cargo build to ensure no errors or warnings

3. **Refactor**
   - Improve code while maintaining test coverage
   - Consider design patterns and best practices

4. **Verify**
   - Run cargo build to ensure no errors or warnings
   - Run all tests to ensure nothing breaks
   - Check test coverage metrics

5. **Document**
   - Add code documentation
   - Update roadmap progress
   - Note any issues with navius crates

**Note on Crate Usage:** A key principle is to **maximize the use of existing Navius crate functionality** (e.g., `navius-core` for config/logging, `navius-auth` for JWT, `navius-db` for database, `navius-http` for server/middleware). Custom implementations will be avoided unless functionality is missing from the core crates. The example app should showcase idiomatic usage with minimal boilerplate.

## Navius Crate Enhancement Strategy

Throughout development, we'll identify areas where the Navius crates need enhancement:

1. **Problem Identification**
   - Document specific limitations encountered
   - Track workarounds implemented

2. **Solution Design**
   - Propose enhancements to the affected crates
   - Design improvements that maintain compatibility

3. **Implementation**
   - Make necessary changes to crates
   - Add tests for new functionality
   - Update documentation

4. **Verification**
   - Ensure example app works with enhancements
   - Verify no regressions in crate functionality

## Key Performance Indicators (KPIs)

The following metrics will be tracked throughout development:

- **Test Coverage**: Target of 90%+ for all code
- **Build Quality**: Zero errors/warnings in cargo build
- **Crate Enhancements**: Number of improvements made to navius crates
- **Documentation Quality**: Comprehensive documentation for all features

## Current Progress
- Status: In Progress
- Progress: 4%
- Updated at: May 31, 2024 