# Crate Enhancements for Example App Implementation

This document outlines the enhancements needed for the Navius crates to better support the example application implementation.

## Priority 1: Zero Boilerplate Initiative

The highest priority enhancement is to move complexity from user code into the Navius crates, following the "Zero Boilerplate" initiative. This initiative aims to **reduce infrastructure code in user applications by 80-90%** and provide a Spring Boot-like developer experience.

### Core Components to Move into Framework

The following components should be moved from user code into the Navius framework:

1. **web_plugin.rs** → navius-http crate
   - Users should not need to implement their own web server setup
   - All server configuration should have sensible defaults
   - Server lifecycle management should be automatic

2. **web_configurator.rs** → navius-config crate
   - Configuration loading should be automatic with convention-based paths
   - Environment variable overrides should be standardized
   - Router configuration should be simplified

3. **app.rs** → navius-core crate
   - Application bootstrapping should be automatic
   - Component registration should be handled via annotations
   - Plugin lifecycle should be managed by the framework

4. **main.rs** - Entry point should be minimal
   - Target reduction from 30+ lines to 3-5 lines
   - All boilerplate handled by macros

### Expected Outcome

**Current main.rs (30+ lines):**
```rust
mod routes;
// ... many imports ...

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let api_router = routes::api::create_router();
    
    let web_plugin = WebPlugin::new("web".into())
        .with_host("127.0.0.1".into())
        .with_port(8000)
        .with_router(Router::new().nest("/api/v1", api_router));
    
    App::builder()
        .with_plugin(web_plugin)
        .build()
        .run()
        .await?;
    
    Ok(())
}
```

**Target main.rs (3-5 lines):**
```rust
#[auto_config]
#[routes]
#[tokio::main]
async fn main() {
    App::new().run().await
}
```

### Implementation Approach

- Create procedural macros for common patterns:
  - `#[navius_app]` - For application bootstrapping and plugin registration
  - `#[auto_config]` - For automatic configuration loading
  - `#[routes]` - For route collection and registration
  - `#[get]`, `#[post]`, etc. - For route definitions
  - `#[inject]` - For dependency injection

- Implement convention-based configuration:
  - Automatic loading of configuration files from standard locations
  - Environment variable overrides with consistent naming
  - Sensible defaults for all settings

- Provide standardized route definition:
  - Declarative route annotations on handler functions
  - Automatic parameter extraction and validation
  - Built-in support for common response types

See the detailed task breakdown in [boilerplate-reduction-tasks.md](./boilerplate-reduction-tasks.md) and implementation plan in [zero-boilerplate-plan.md](./zero-boilerplate-plan.md).

## Priority 2: Improved Error Handling

Enhance error handling capabilities to provide:

- Standard error types for common scenarios
- Automatic error mapping between layers
- Consistent error response formatting
- Easy integration with logging and metrics

## Priority 3: Dependency Injection

Implement a dependency injection system that:

- Automatically resolves dependencies
- Supports different scopes (singleton, request, etc.)
- Allows for easy testing with mocks
- Integrates with the plugin system

## Priority 4: Configuration Management

Enhance configuration management to:

- Support multiple environment configurations
- Validate configuration at startup
- Provide helpful error messages for misconfiguration
- Support hot reloading where appropriate

## Priority 5: Testing Utilities

Develop testing utilities that make it easy to:

- Test handlers without a full server
- Mock dependencies
- Simulate requests and validate responses
- Test plugins in isolation

## Priority 6: Documentation and Examples

- Comprehensive API documentation
- Example implementations for common patterns
- Migration guides for existing applications
- Performance benchmarks and recommendations

## Progress Tracking: Zero Boilerplate Initiative

The Zero Boilerplate Initiative is currently in progress with approximately 35% completion. Teams have begun implementation work on all major components:

- **WebPlugin Abstraction (NC-11)**: 70% complete - Implementation moved to navius-http crate, with a complete WebPlugin implementation that integrates with the existing HttpServerConfig. Documentation, examples, and integration tests added.
- **WebConfigurator Abstraction (NC-12)**: 30% complete - Moving to navius-config crate
- **App Builder Simplification (NC-13)**: 15% complete - Moving to navius-core crate
- **Main Entry Point Simplification (NC-14)**: 15% complete - Creating proc-macros

For detailed status updates, see [crate-enhancements.md](../roadmap/sub-process/crate-enhancements.md) in the roadmap directory. 