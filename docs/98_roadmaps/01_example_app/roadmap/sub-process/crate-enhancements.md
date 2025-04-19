# Crate Enhancements for Simmr - Social Cooking Platform

This document tracks the specific enhancements required in the Navius crate ecosystem to support the Simmr social cooking platform backend.

| Enhancement ID | Crate(s) Affected | Title                   | Description                                                                                                                               | Status      | Implementation Notes                                         |
| -------------- | ----------------- | ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------------------ |
| NC-1           | navius-core       | Plugin/DI System        | Initial DI and config loading via `ApplicationBuilder`. Basic error handling improvements. (Initial plugin concept superseded by direct DI). | Completed   | Implemented via `ApplicationBuilder` refactoring.            |
| NC-2           | navius-http       | Declarative Route Macros | Implement `#[route(path="...", method=..., ...)]` and `#[nest(...)]` macros for defining routes and applying middleware/auth declaratively.   | Completed   | Basic implementation functioning; API routes working with proper nesting. |
| NC-3           | navius-core       | Config Macros           | Implement `#[auto_config]` and `#[derive(Configurable)]` for automated configuration setup and struct mapping.                              | Not Started | Requires proc-macro development.                             |
| NC-4           | navius-di         | DI Improvements         | Implement `Component<T>` wrapper or similar extractor for simplified, type-safe component injection in handlers/services.                     | Not Started | Explore alternatives like direct `State` extraction vs. wrapper. |
| NC-5           | navius-db         | Repository Abstraction  | Define standard repository traits/macros for recipe and user data. Simplify common CRUD operations.                                       | Not Started | Consider base traits or codegen for repositories.          |
| NC-6           | navius-auth       | Auth Policy Integration | Standardize how `auth_policy` in `#[route]` maps to specific auth middleware/configurations for user authentication.                      | Not Started | Define policy registration/lookup mechanism.               |
| NC-7           | navius-media      | Media Processing        | Add support for recipe image upload, processing, and storage.                                                                           | Not Started | Implement streaming uploads, image optimization, and CDN integration. |
| NC-8           | navius-social     | Social Features         | Create new crate with common social media patterns: following, sharing, activity feeds.                                                  | Not Started | Build reusable components for social interactions.         |
| NC-9           | navius-search     | Search Functionality    | Implement search capabilities for recipes, ingredients, and users.                                                                       | Not Started | Consider integration with Elasticsearch or similar technology. |
| NC-10          | navius-metrics    | Usage Analytics         | Track platform usage patterns, popular recipes, and user engagement metrics.                                                             | Not Started | Design metrics collection that respects privacy concerns.  |
| NC-11          | navius-http       | WebPlugin Abstraction   | Move WebPlugin implementation from user code to navius-http crate. Simplify server setup and configuration for end users.                  | High Priority | Current implementation requires too much boilerplate in user code. Implementation work has begun.  |
| NC-12          | navius-config     | WebConfigurator Abstraction | Move WebConfigurator from user code to navius-config crate. Provide simple configuration loading with sensible defaults.                 | High Priority | Configuration should be handled transparently with minimal user code. |
| NC-13          | navius-core       | App Builder Simplification | Move App builder pattern implementation from user code to navius-core crate. Provide a simplified API for application setup.            | High Priority | Users shouldn't need to implement their own App builder. |
| NC-14          | navius-core/http  | Main Entry Point Simplification | Create macros and abstractions to simplify the main.rs entry point. Reduce boilerplate needed to start an application.           | High Priority | Target a minimal main.rs with just a few lines of code. |
| NC-15          | Framework-wide    | Zero Boilerplate Initiative | Comprehensive audit of all user code requirements. Move complexity into framework.                                                     | Highest Priority | No user should have to write infrastructure code. This is our current focus. |

---
*Updated at: May 30, 2025*

## Related Documents
- [Main Roadmap](../01-example-app.md)
- [Implementation Plan](../example-app-plan.md)

## Enhancement Process

For each enhancement:

1. **Problem Identification**
   - Document specific limitations encountered
   - Describe the use case that revealed the limitation
   - Note any workarounds implemented

2. **Enhancement Proposal**
   - Propose specific changes to the affected crate
   - Document the expected benefits
   - Consider backward compatibility

3. **Implementation**
   - Create a branch for the enhancement
   - Implement with TDD methodology
   - Add tests for new functionality
   - Update documentation

4. **Verification**
   - Test the enhancement in the Simmr backend
   - Verify no regressions in the crate
   - Document the updated usage

## Zero Boilerplate Initiative (ZBI)

The Zero Boilerplate Initiative aims to eliminate all unnecessary infrastructure code from user applications, making Navius a true "Spring-Boot for Rust" experience. **This is our highest priority enhancement**.

### Current Pain Points

After review of the current implementation, we've identified several files that should not be required in user code:

1. **`web_configurator.rs`** - Configuration loading should be handled by the framework
2. **`web_plugin.rs`** - Web server setup and routing should be abstracted 
3. **`app.rs`** - Application initialization and lifecycle should be simplified
4. **`main.rs`** - Entry point should be minimal with most functionality provided by macros

### Target Main.rs

Our goal is to simplify the main.rs entry point to something close to this level of simplicity:

```rust
#[auto_config]
#[routes]
#[tokio::main]
async fn main() {
    App::new()
        .run()
        .await;
}

#[get("/api/v1/hello")]
async fn hello_world() -> impl IntoResponse {
    "Hello, world!"
}
```

### Implementation Strategy

1. **Move Boilerplate to Crates** 
   - Extract common functionality from user code into appropriate crates
   - Provide sensible defaults for all configuration
   - Design flexible but powerful abstractions

2. **Create Declarative API**
   - Use attribute macros for routes, configuration, and DI
   - Hide implementation details behind clean interfaces
   - Automate wiring of components

3. **Measure Success**
   - Compare line counts in starter apps before and after
   - Aim for 80% reduction in infrastructure code
   - Ensure no loss of functionality or flexibility

## Implementation Plans for Zero Boilerplate Initiative

### NC-11: WebPlugin Abstraction (navius-http)

**Current Status**: In Progress
**Assigned To**: Team Alpha
**Implementation Timeline**:
- Phase 1 (Week 1): Extract WebPlugin from example codebase - DONE
- Phase 2 (Week 2): Create standardized API in navius-http - IN PROGRESS 
- Phase 3 (Week 3): Add configuration extension points
- Phase 4 (Week 4): Create documentation and examples

**Technical Approach**:
1. Copy the existing WebPlugin implementation to navius-http crate
2. Create a builder pattern for customization
3. Implement sensible defaults for all configuration options
4. Add hooks for custom middleware and error handlers

**Expected Outcome**:
```rust
// User code with new API
#[tokio::main]
async fn main() {
    App::new()
        .run_with_web(WebAppConfig::default()
            .with_port(8080)
            .with_host("0.0.0.0"))
        .await
}
```

### NC-12: WebConfigurator Abstraction (navius-config)

**Current Status**: Not Started
**Assigned To**: Team Beta
**Implementation Timeline**:
- Phase 1 (Week 1-2): Extract WebConfigurator from example codebase
- Phase 2 (Week 2-3): Create auto_config macro
- Phase 3 (Week 3-4): Implement configuration hierarchy and defaults
- Phase 4 (Week 4): Create documentation and examples

**Technical Approach**:
1. Move WebConfigurator implementation to navius-config
2. Create proc-macro for #[auto_config] attribute
3. Implement configuration source hierarchies (env vars, files, defaults)
4. Add strong typing for configuration with validation

**Expected Outcome**:
```rust
// User marks app as auto-configured
#[auto_config]
#[tokio::main]
async fn main() {
    // Configuration automatically loaded
    App::new().run().await
}

// Or explicit configuration
#[tokio::main]
async fn main() {
    let config = Config::default()
        .with_file("config.toml")
        .load()
        .unwrap();
    
    App::new()
        .with_config(config)
        .run()
        .await
}
```

### NC-13: App Builder Simplification (navius-core)

**Current Status**: Not Started
**Assigned To**: Team Gamma
**Implementation Timeline**:
- Phase 1 (Week 1): Extract App and AppBuilder from example codebase
- Phase 2 (Week 2): Create standardized APIs in navius-core
- Phase 3 (Week 3): Implement automatic component discovery
- Phase 4 (Week 4): Create documentation and examples

**Technical Approach**:
1. Move App and AppBuilder to navius-core crate
2. Simplify the builder interface with sensible defaults
3. Add auto-discovery for handlers and components
4. Create extension traits for specialized app configurations

**Expected Outcome**:
```rust
// Extremely simple app creation
async fn main() {
    App::new().run().await
}

// Or with customizations
async fn main() {
    App::new()
        .with_plugin(CustomPlugin::new())
        .with_component(MyService::new())
        .run()
        .await
}
```

### NC-14: Main Entry Point Simplification (navius-core/http)

**Current Status**: Not Started
**Assigned To**: Team Delta
**Implementation Timeline**:
- Phase 1 (Week 1-2): Design macro API
- Phase 2 (Week 2-3): Implement proc-macros for routes and configuration
- Phase 3 (Week 3-4): Create integration with App builder
- Phase 4 (Week 4): Create documentation and examples

**Technical Approach**:
1. Create #[routes] proc-macro to collect handler functions
2. Implement automatic route registration
3. Create integration with App builder
4. Implement integration tests for all combinations

**Expected Outcome**:
```rust
#[auto_config]
#[routes]
#[tokio::main]
async fn main() {
    App::new().run().await
}

#[get("/api/v1/hello")]
async fn hello_world() -> impl IntoResponse {
    "Hello, world!"
}

#[get("/api/v1/users/{id}")]
async fn get_user(Path(id): Path<String>) -> impl IntoResponse {
    format!("User ID: {}", id)
}
```

## Final Target Architecture

After implementing the Zero Boilerplate Initiative, we aim to achieve the following architecture:

1. **User Code**:
   - `main.rs` - Minimal entry point with macro annotations
   - `handlers/*.rs` - Route handlers with declarative annotations
   - `services/*.rs` - Business logic in services
   - `models/*.rs` - Domain model definitions

2. **Framework Code** (Moved from user space):
   - `WebConfigurator` -> navius-config crate
   - `WebPlugin` -> navius-http crate
   - `App`/`AppBuilder` -> navius-core crate
   - Route registration -> navius-http macros
   - Configuration loading -> navius-config macros

3. **User Experience**:
   - Write handlers with declarative annotations
   - Focus on business logic, not infrastructure
   - Minimal configuration required
   - Convention over configuration
   - Spring Boot-like simplicity

## Simmr Application Requirements

Based on the Simmr backend requirements, we've identified the following key enhancements needed:

### Required Macro Support
- `#[auto_config(WebConfigurator)]` - For automatic configuration loading
- `#[route]` with method parameters - For RESTful API routes
- `#[nest]` - For organizing routes by resource (recipes, users, etc.)
- `#[derive(Configurable)]` - For auto-configurable types

### Required Component Features
- User authentication and authorization
- Recipe data storage and retrieval
- Social interaction patterns (following, sharing, etc.)
- Media handling for recipe images
- Search functionality
- Analytics and metrics collection

## Detailed Crate Enhancement Tracking

### navius-core

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NC-1 | Plugin System | Completed | Implement plugin system for modular application setup | Implemented App and AppBuilder classes that leverage the existing navius-plugin Registry. Created SqlxPlugin and WebPlugin implementations. |
| NC-2 | Application Builder | Completed | Create App builder pattern for clean initialization | Supports add_plugin() and run() methods |
| NC-3 | Component Registration | In Progress | Implement component registration and retrieval system | Required for dependency injection |
| NC-4 | Error Handling | Completed | Implement standardized error handling for API responses | Consistent error pattern implemented |
| NC-5 | Repository Abstraction | Not Started | Define standard repository traits for recipe and user data | Will include specializations for social cooking context |
| NC-6 | Configuration System | Not Started | Implement hierarchical config with environment overrides | Support for complex Simmr configuration needs |
| NC-13 | App Builder Migration | **High Priority** | Move App builder from user code to framework | Users shouldn't implement this themselves |
| NC-14 | Entry Point Simplification | **High Priority** | Create macros to simplify main.rs | Minimize code in user application entry point |
| NC-15 | Zero Boilerplate Initiative | **Highest Priority** | Comprehensive audit and refactoring | Move infrastructure code to crates |

### navius-http

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NH-1 | Route Macros | Completed | Implement macros for route definition (#[route]) | Supports method specification and path parameters |
| NH-2 | Nested Routes | Completed | Support for nested route modules with #[nest] | Working implementation allows route organization by resource |
| NH-3 | Response Types | In Progress | Implement IntoResponse trait | Allow diverse return types from handlers |
| NH-4 | Path Parameters | Completed | Support path parameter extraction | Extract parameters from URL paths |
| NH-5 | WebPlugin | Completed | Create web server plugin | Integrates with App::new() builder |
| NH-6 | Media Uploads | Not Started | Support for multipart form handling | Required for recipe image uploads |
| NH-7 | Rate Limiting | Not Started | Implement rate limiting middleware | Protect Simmr API from abuse |
| NH-11 | WebPlugin Abstraction | **High Priority - In Progress** | Move WebPlugin to framework | Simplify server configuration |

### navius-config

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NC-12 | WebConfigurator Abstraction | **High Priority** | Move configuration logic to framework | Provide simple interface with sensible defaults |

### navius-db / navius-db-postgres

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| ND-1 | Recipe Repository | Not Started | Create specialized repository for recipes | Include support for ingredients and steps as nested data |
| ND-2 | User Repository | Not Started | Implement user profile storage | Support for profile pictures and preferences |
| ND-3 | Social Graph Storage | Not Started | Design optimal schema for following/followers | Consider performance implications of social graph queries |
| ND-4 | Query Optimization | Not Started | Optimize common social cooking queries | Focus on recipe discovery and filtering |
| ND-5 | Full-Text Search | Not Started | Implement recipe search functionality | Consider PostgreSQL full-text search capabilities |

### navius-auth

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NA-1 | JWT Support | In Progress | Implement JWT generation and validation | Required for user authentication |
| NA-2 | Claims Extraction | In Progress | Auto-extract claims from requests | Allow direct injection of Claims into handlers |
| NA-3 | Authentication Middleware | In Progress | Create authentication middleware | Should validate JWT tokens |
| NA-4 | Social Login | Not Started | Support for OAuth providers | Allow login with Google, Facebook, etc. |
| NA-5 | Permission System | Not Started | Role-based access control | Control access to recipes and social features |

### navius-social (New Crate)

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NS-1 | Following System | Not Started | Implement follower/following relationships | Core social graph functionality |
| NS-2 | Activity Feed | Not Started | Create activity feed generation | Aggregate and personalize user activities |
| NS-3 | Notifications | Not Started | Design notification system | Support multiple notification channels |
| NS-4 | Content Sharing | Not Started | Implement recipe sharing | Allow users to share and repost recipes |
| NS-5 | Comments & Ratings | Not Started | Create comment and rating system | Support for threaded comments and star ratings |

### navius-media (New Crate)

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NM-1 | Image Upload | Not Started | Implement secure image uploading | Support for recipe photos |
| NM-2 | Image Processing | Not Started | Create image optimization pipeline | Generate thumbnails and responsive sizes |
| NM-3 | Storage Integration | Not Started | Integrate with cloud storage | Support for S3 or similar services |
| NM-4 | CDN Support | Not Started | Configure CDN for media delivery | Improve image loading performance |
| NM-5 | Video Support | Not Started | (Future) Support for recipe videos | Allow short cooking demonstrations |

### navius-search (New Crate)

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NSE-1 | Recipe Search | Not Started | Implement recipe search functionality | Support for ingredient and title search |
| NSE-2 | User Search | Not Started | Create user discovery features | Find users by name, interests, etc. |
| NSE-3 | Tag System | Not Started | Implement tagging for recipes | Improve discoverability via tags |
| NSE-4 | Search Suggestions | Not Started | Create search autocomplete | Enhance user experience with smart suggestions |
| NSE-5 | Filter System | Not Started | Build advanced recipe filters | Filter by cuisine, ingredients, time, etc. |

## Enhancement Proposals

### Enhancement Proposal: Zero Boilerplate Initiative (ZBI)

**ID**: NC-15  
**Crate**: Framework-wide  
**Title**: Zero Boilerplate Initiative  
**Status**: In Progress - Highest Priority  

**Problem Statement**:  
The current Navius codebase requires users to implement too much infrastructure code in their applications. Files like `web_configurator.rs`, `web_plugin.rs`, and `app.rs` should be part of the framework, not user code. The `main.rs` entry point contains too much boilerplate that should be handled through macros and sensible defaults.

**Use Case**:  
Developers should be able to focus on business logic rather than infrastructure setup. A Spring Boot-like developer experience would allow developers to quickly build applications with minimal configuration.

**Proposed Solution**:  
1. Migrate infrastructure code from user applications to Navius crates:
   - Move WebConfigurator to navius-config
   - Move WebPlugin to navius-http
   - Move App builder to navius-core
   - Create macros to simplify main entry point

2. Implement attribute macros for declarative configuration:
   ```rust
   #[auto_config]
   #[routes]
   #[tokio::main]
   async fn main() {
       App::new().run().await;
   }
   ```

3. Simplify handler definitions with attribute macros:
   ```rust
   #[get("/hello")]
   async fn hello_world() -> impl IntoResponse {
       "Hello, world!"
   }
   ```

**Benefits**:  
- 80% reduction in infrastructure code
- Focus on business logic rather than setup
- True Spring Boot-like developer experience
- Faster onboarding for new developers
- Less error-prone application setup

**Backward Compatibility**:  
This refactoring will maintain backward compatibility by:
- Keeping the same underlying architecture
- Extracting current user code verbatim into crates
- Providing equivalent APIs with the same semantics

**Implementation Plan**:  
1. Extract WebConfigurator to navius-config crate
2. Move WebPlugin implementation to navius-http crate
3. Migrate App builder to navius-core
4. Create macros for main entry point simplification
5. Update documentation and examples
6. Refactor Simmr codebase to use new APIs

### Enhancement Proposal: WebConfigurator Abstraction

**ID**: NC-12  
**Crate**: navius-config  
**Title**: WebConfigurator Abstraction  
**Status**: Proposed - High Priority  

**Problem Statement**:  
Currently, users need to implement their own WebConfigurator in user code, handling configuration loading from multiple sources. This should be part of the framework with sensible defaults.

**Use Case**:  
Configuration loading is a cross-cutting concern that should be handled consistently across all Navius applications with minimal user intervention.

**Proposed Solution**:  
Create a standard WebConfigurator in the navius-config crate:

```rust
// In navius-config crate
pub struct WebConfigurator {
    config_prefix: String,
    config_dir: String,
}

impl WebConfigurator {
    pub fn new() -> Self { /* ... */ }
    pub fn with_prefix(self, prefix: impl Into<String>) -> Self { /* ... */ }
    pub fn with_config_dir(self, dir: impl Into<String>) -> Self { /* ... */ }
    pub fn load_config(&self) -> Result<Config> { /* ... */ }
}

// User code simply uses the macro:
#[auto_config]
async fn main() {
    // Config automatically loaded
}
```

**Benefits**:  
- Standardized configuration loading
- Sensible defaults for most applications
- Customization possible when needed
- Reduced boilerplate in user code

**Implementation Plan**:  
1. Extract current WebConfigurator implementation to navius-config
2. Add sensible defaults for all configuration parameters
3. Create auto_config macro to wire everything up
4. Update documentation with usage examples

### Enhancement Proposal: App Builder Simplification

**ID**: NC-13  
**Crate**: navius-core  
**Title**: App Builder Simplification  
**Status**: Proposed - High Priority  

**Problem Statement**:  
Currently, users need to implement their own App and AppBuilder classes in user code. This should be part of the framework with a clean, extensible API.

**Use Case**:  
Application initialization and lifecycle management is a framework concern and should be handled consistently with minimal user code.

**Proposed Solution**:  
Move the App and AppBuilder implementations to navius-core:

```rust
// In navius-core crate
pub struct App {
    registry: Registry,
    config: Config,
    // ...
}

impl App {
    pub fn new() -> Self { /* ... */ }
    pub fn with_plugin<P: Plugin>(self, plugin: P) -> Self { /* ... */ }
    pub fn with_component<C: Component>(self, component: C) -> Self { /* ... */ }
    pub fn run(self) -> impl Future<Output = Result<()>> { /* ... */ }
}

// User code simply creates and runs:
async fn main() {
    App::new()
        .with_plugin(WebPlugin::default())
        .run()
        .await
        .unwrap();
}
```

**Benefits**:  
- Standardized application initialization
- Consistent plugin and component lifecycle
- Clean, fluent API
- Reduced boilerplate in user code

**Implementation Plan**:  
1. Extract current App and AppBuilder implementations to navius-core
2. Add sensible defaults and extensibility points
3. Create integration points with other enhancements (auto-config, routes)
4. Update documentation with usage examples

### Enhancement Proposal: Main Entry Point Simplification

**ID**: NC-14  
**Crate**: navius-core/http  
**Title**: Main Entry Point Simplification  
**Status**: Proposed - High Priority  

**Problem Statement**:  
The main entry point in user applications contains too much boilerplate code for router setup, plugin configuration, and application initialization.

**Use Case**:  
Application entry points should be minimal, focusing only on essential customizations. Most of the setup should be handled by the framework.

**Proposed Solution**:  
Create attribute macros for simplified main entry points:

```rust
// User code with new API
#[auto_config]
#[routes]
#[tokio::main]
async fn main() {
    App::new().run().await
}

#[get("/api/v1/hello")]
async fn hello_world() -> impl IntoResponse {
    "Hello, world!"
}
```

The macros would:
1. Collect all handler functions with route annotations
2. Register them with the router
3. Configure the application with sensible defaults
4. Set up the web server

**Benefits**:  
- Minimal main entry point
- Declarative route definitions
- Focus on business logic
- Convention over configuration

**Implementation Plan**:  
1. Create proc-macros for route collection and registration
2. Implement automatic application setup
3. Integrate with WebPlugin and WebConfigurator
4. Create comprehensive documentation and examples

## Current Status
- Status: In Progress
- Progress: 20%
- Updated at: May 30, 2025