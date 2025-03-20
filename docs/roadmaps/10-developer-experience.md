# Developer Experience Roadmap

## Overview
A pragmatic approach to developer experience for our Rust Axum backend, focusing on the essential capabilities needed for efficient development, debugging, and testing in a local environment that mirrors our production stack.

## Current State
Our application needs fundamental developer experience improvements to accelerate development cycles, streamline testing, and ensure consistent development across environments.

## Target State
A practical developer experience featuring:
- Efficient local development workflow that mirrors production
- Essential debugging and observability capabilities 
- Security-focused testing tools
- Just enough documentation to onboard developers quickly

## Implementation Progress Tracking

### Phase 1: Development Environment
1. **Local Development Setup**
   - [ ] Create Docker Compose configuration for local Redis, Postgres services
   - [ ] Implement environment-based configuration loading 
   - [ ] Build service mocks/emulators for local development
   - [ ] Implement unified startup script for one-command setup
   
   *Updated at: Not started*

2. **Rapid Iteration Tools**
   - [ ] Implement file watching with cargo-watch integration
   - [ ] Create development mode with enhanced error messages
   - [ ] Add hot reload capabilities for configuration changes
   - [ ] Implement test data seeding for development database
   
   *Updated at: Not started*

3. **Development Testing Tools**
   - [ ] Create testing utilities for API endpoints
   - [ ] Implement security headers validation in development mode
   - [ ] Add permission testing utilities
   - [ ] Create data sanitization verification tools
   
   *Updated at: Not started*

### Phase 2: Debugging and Observability
1. **Request Debugging**
   - [ ] Implement structured request/response logging
   - [ ] Create request tracing with context propagation
   - [ ] Add performance timing annotations
   - [ ] Implement correlation ID tracking
   
   *Updated at: Not started*

2. **Error Handling**
   - [ ] Create developer-friendly error messages in development mode
   - [ ] Implement error cataloging with troubleshooting guidance
   - [ ] Add contextual error information support
   - [ ] Build security-safe error reporting
   
   *Updated at: Not started*

3. **Database Tools**
   - [ ] Implement migration tooling for Postgres
   - [ ] Create database reset/seed commands for testing
   - [ ] Add query logging in development mode
   - [ ] Implement transaction debugging helpers
   
   *Updated at: Not started*

### Phase 3: Documentation and Examples
1. **Documentation**
   - [ ] Build essential API documentation with OpenAPI
   - [ ] Create getting started guide for new developers
   - [ ] Document security practices and requirements
   - [ ] Add development environment setup guide
   
   *Updated at: Not started*

2. **Patterns and Examples**
   - [ ] Document recommended Axum implementation patterns
   - [ ] Create example handlers for common use cases
   - [ ] Add reference implementations for Redis and Postgres interaction
   - [ ] Document service patterns and best practices
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 20, 2024
- **Next Milestone**: Local Development Setup

## Success Criteria
- Developers can run the complete system locally with one command
- Code changes are reflected quickly during development
- Error messages provide actionable guidance
- Development issues can be diagnosed efficiently
- Security testing can be performed effectively in development
- New developers can be onboarded in less than one day

## Implementation Notes

### Example Implementation: Local Development Environment

```rust
use std::env;
use axum::{
    routing::get,
    Router, 
    extract::State,
    middleware::{self, Next},
    response::Response,
};
use tokio::fs::File;
use notify::{Watcher, RecursiveMode};
use serde::Deserialize;

// Development environment configuration
#[derive(Deserialize, Clone)]
pub struct DevConfig {
    // Local development overrides
    pub development_mode: bool,
    pub enhanced_logging: bool,
    pub reload_templates: bool,
    pub postgres_connection: String,
    pub redis_connection: String,
    pub mock_services: Vec<String>,
}

impl DevConfig {
    pub fn from_env() -> Self {
        // Load from dev.env file or environment
        dotenv::from_filename("dev.env").ok();
        
        Self {
            development_mode: env::var("DEV_MODE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enhanced_logging: env::var("DEV_ENHANCED_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            reload_templates: env::var("DEV_RELOAD_TEMPLATES")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            postgres_connection: env::var("DEV_POSTGRES_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/app_dev".to_string()),
            redis_connection: env::var("DEV_REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            mock_services: env::var("DEV_MOCK_SERVICES")
                .unwrap_or_else(|_| "external_api,payment_service".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }
}

// Development environment setup middleware
async fn dev_middleware(
    State(dev_config): State<DevConfig>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    // Add development-only headers
    if dev_config.enhanced_logging {
        response.headers_mut().insert(
            "X-Dev-Mode", 
            "true".parse().unwrap()
        );
    }
    
    response
}

// Main development server setup
pub async fn setup_development_server() -> Router {
    // Load development configuration
    let dev_config = DevConfig::from_env();
    
    // Set up file watcher for hot reloading
    if dev_config.reload_templates {
        let mut watcher = notify::recommended_watcher(|res| {
            match res {
                Ok(event) => println!("File changed: {:?}", event),
                Err(e) => println!("Watch error: {:?}", e),
            }
        }).unwrap();
        
        watcher.watch("./templates", RecursiveMode::Recursive).unwrap();
    }
    
    // Set up database with development schema
    let db_pool = setup_database(&dev_config.postgres_connection).await;
    
    // Set up Redis connection
    let redis_client = setup_redis(&dev_config.redis_connection).await;
    
    // Set up mock services if configured
    let mock_services = if !dev_config.mock_services.is_empty() {
        setup_mock_services(&dev_config.mock_services).await
    } else {
        setup_real_services().await
    };
    
    // Configure the application with development-specific middleware
    Router::new()
        .route("/dev/status", get(dev_status_handler))
        .route("/dev/seed-data", get(seed_test_data))
        .route("/dev/reset-db", get(reset_database))
        .nest("/api", 
            build_api_router()
                .layer(middleware::from_fn_with_state(
                    dev_config.clone(),
                    dev_middleware,
                ))
        )
        .with_state(dev_config)
        .with_state(db_pool)
        .with_state(redis_client)
        .with_state(mock_services)
}

// Development status handler
async fn dev_status_handler(
    State(dev_config): State<DevConfig>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "dev_mode": true,
        "enhanced_logging": dev_config.enhanced_logging,
        "mock_services": dev_config.mock_services,
        "connections": {
            "postgres": dev_config.postgres_connection,
            "redis": dev_config.redis_connection,
        }
    }))
}

// Database seeding utility
async fn seed_test_data() -> &'static str {
    // Insert test data for development
    "Database seeded with test data"
}

// Database reset utility
async fn reset_database() -> &'static str {
    // Reset database to clean state
    "Database reset to initial state"
}
```

This roadmap prioritizes a pragmatic developer experience that:

1. **Supports rapid development**: Fast feedback loops with file watching and hot reloading for configuration changes

2. **Mirrors production**: Local environment closely matches production deployment with Redis and Postgres

3. **Prioritizes security**: Built-in tools for testing security features and configurations

4. **Enables debugging**: Enhanced logging, request tracing, and database tools make debugging straightforward

The implementation focuses on making the developer experience smooth and efficient, independent of the specific deployment environment.

## References
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [cargo-watch](https://crates.io/crates/cargo-watch)
- [Docker Compose](https://docs.docker.com/compose/)
- [OpenAPI Documentation with Axum](https://github.com/juhaku/utoipa) 