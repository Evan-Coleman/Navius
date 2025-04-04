# Example App Implementation Plan

## Application Overview

The example application will be a simple but comprehensive task management system that demonstrates all key capabilities of the Navius crate ecosystem. It will allow users to:

1. Create and manage tasks with deadlines
2. Organize tasks in categories/projects
3. Assign tasks to users
4. Track task completion status
5. Generate reports and metrics

## Technical Stack

- **Framework**: Axum web framework
- **Database**: PostgreSQL via navius-db-postgres
- **Authentication**: Microsoft Entra via navius-auth-entra
- **Caching**: Redis via navius-cache
- **API Documentation**: OpenAPI/Swagger
- **Background Jobs**: navius-job
- **Messaging**: navius-messaging
- **Metrics**: Prometheus via navius-metrics-prometheus
- **Testing**: navius-test and navius-test-utils

## Application Structure

The example app will follow a clean, declarative style based on this target main.rs example:

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

// Additional endpoints for authentication, task management, etc.
```

## Implementation Details

### Phase 1: Project Setup and Core Framework

#### Step 1.1: Basic Project Structure
- Set up project with cargo new
- Configure Cargo.toml with initial dependencies
- Set up directory structure

#### Step 1.2: Core Framework Components
- Implement App builder pattern
- Create Plugin trait and registration mechanism
- Implement basic application lifecycle management
- Create route collection and registration system

#### Step 1.3: Route Macro Implementation
- Create proc-macro crate for route macros
- Implement #[routes], #[get], and #[post] macros
- Add support for nested routes with #[nest]
- Implement path parameter extraction

#### Step 1.4: Configuration System
- Implement #[auto_config] macro
- Create Configurable derive macro
- Build configuration loading system
- Add support for configuration prefixes

### Phase 2: Dependency Injection and Components

#### Step 2.1: Dependency Injection System
- Implement Component<T> wrapper
- Create service registration mechanism
- Build dependency resolution system
- Implement parameter extraction for handlers

#### Step 2.2: Core Plugins
- Create WebPlugin for HTTP server
- Implement SqlxPlugin for database access
- Build basic plugin dependencies management

#### Step 2.3: JWT Authentication
- Implement JWT generation and validation
- Create authentication middleware
- Add Claims extraction for handlers

### Phase 3: Task Management Domain

#### Step 3.1: Task Domain Model
- Implement Task entity and validation
- Create Category/Project domain model
- Implement User reference model

#### Step 3.2: Task Services
- Implement TaskService with dependency injection
- Create CategoryService with business logic
- Build UserService for reference data

#### Step 3.3: Task Repositories
- Implement database schema with migrations
- Create Repository interfaces
- Implement PostgreSQL repositories

### Phase 4: API Development

#### Step 4.1: Task API Endpoints
- Create task management endpoints
- Implement category management endpoints
- Build user reference endpoints

#### Step 4.2: API Security
- Implement authentication for endpoints
- Add authorization checks
- Configure security headers and CORS

#### Step 4.3: API Documentation
- Implement OpenAPI documentation
- Create API usage examples
- Add request/response schemas

### Phase 5: Advanced Features

#### Step 5.1: Background Jobs
- Implement task notification jobs
- Create scheduled task processing
- Configure job monitoring

#### Step 5.2: Event System
- Implement domain events
- Create event handlers
- Build event-based workflows

#### Step 5.3: Metrics
- Implement performance metrics
- Add health checks
- Create monitoring dashboards

### Phase 6: Testing and Documentation

#### Step 6.1: Testing
- Complete test coverage
- Implement integration tests
- Add performance tests

#### Step 6.2: Documentation
- Complete API documentation
- Create usage guides
- Document architecture decisions

## Implementation Strategy

Our approach will be to work backward from the target main.rs example:

1. **Identify Core Components**
   - Determine what framework features are needed
   - Map out dependencies between components
   - Create minimal implementations first

2. **Build Foundation First**
   - Start with App builder and Plugin system
   - Implement basic route macros
   - Create minimal working example

3. **Add Features Incrementally**
   - Build out each feature using TDD
   - Focus on one component at a time
   - Ensure everything works together

4. **Enhance as Needed**
   - Identify limitations in Navius crates
   - Implement enhancements with TDD
   - Document improvements

## Navius Crate Enhancement Focus Areas

The implementation will require several enhancements to Navius crates:

1. **navius-core**
   - Plugin system implementation
   - Application builder pattern
   - Component registration system

2. **navius-http**
   - Route macro development
   - Nested route support
   - Path parameter extraction

3. **navius-di** (new crate)
   - Dependency injection framework
   - Component wrapper
   - Parameter extraction system

4. **navius-config** (new crate)
   - Configuration system
   - auto_config macro
   - Configurable derive macro

These enhancements will be tracked in the [Crate Enhancements](./sub-process/crate-enhancements.md) document.

## Success Criteria

The example app implementation will be considered successful when:

1. All features are implemented and working correctly
2. Test coverage exceeds 90%
3. No errors or warnings in cargo build
4. All identified crate enhancements are implemented
5. Comprehensive documentation is available
6. The app demonstrates the clean, declarative style shown in the target example

## Current Status
- Status: Not Started
- Progress: 0%
- Updated at: May 30, 2024 