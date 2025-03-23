# Developer Experience Roadmap

## Overview
A pragmatic approach to developer experience for Navius, focusing on the essential capabilities needed for efficient development, debugging, and testing in a local environment that mirrors our production stack.

## Current State
- Visual Studio Code configuration completed with enhanced settings
- Basic development environment setup needed
- Testing infrastructure in place with 35% coverage
- Need improved debugging and observability tools

## Target State
A practical developer experience featuring:
- Efficient local development workflow that mirrors production
- Essential debugging and observability capabilities 
- Security-focused testing tools
- Just enough documentation to onboard developers quickly

## Implementation Progress Tracking

### Phase 1: Development Environment
1. **Local Development Setup**
   - [ ] Create Docker Compose configuration for local services:
     - [ ] PostgreSQL container with proper initialization
     - [ ] Redis container with persistence
     - [ ] Mock AWS services with LocalStack
     - [ ] Health check endpoints for all services
   - [ ] Implement environment-based configuration loading:
     - [ ] Development environment overrides
     - [ ] Local secrets management
     - [ ] Service connection strings
     - [ ] Feature flags
   - [ ] Build service mocks/emulators:
     - [ ] Mock authentication service
     - [ ] Mock external APIs
     - [ ] Mock AWS services
     - [ ] Mock payment providers
   - [ ] Create unified startup script:
     - [ ] Service dependency checks
     - [ ] Database migrations
     - [ ] Configuration validation
     - [ ] Health verification
   
   *Updated at: Not started*

2. **Rapid Iteration Tools**
   - [ ] Implement file watching with cargo-watch:
     - [ ] Source code recompilation
     - [ ] Test execution
     - [ ] Linting
     - [ ] Documentation generation
   - [ ] Create development mode:
     - [ ] Enhanced error messages
     - [ ] Request/response logging
     - [ ] Performance metrics
     - [ ] Stack traces
   - [ ] Add hot reload capabilities:
     - [ ] Configuration reloading
     - [ ] Template recompilation
     - [ ] Static asset serving
     - [ ] Route updates
   - [ ] Implement test data seeding:
     - [ ] Development database setup
     - [ ] Test data generation
     - [ ] Data reset functionality
     - [ ] Fixture management
   
   *Updated at: Not started*

3. **Development Testing Tools**
   - [ ] Create testing utilities:
     - [ ] API endpoint testing helpers
     - [ ] Request builders
     - [ ] Response validators
     - [ ] Test data generators
   - [ ] Implement security validation:
     - [ ] Header validation
     - [ ] Authentication testing
     - [ ] Authorization checks
     - [ ] Input sanitization
   - [ ] Add permission testing:
     - [ ] Role-based access control
     - [ ] Scope validation
     - [ ] Token verification
     - [ ] Policy enforcement
   - [ ] Create data validation:
     - [ ] Schema validation
     - [ ] Data sanitization
     - [ ] Format verification
     - [ ] Constraint checking
   
   *Updated at: Not started*

4. **IDE Configuration and Documentation**
   - [x] Create Visual Studio Code configuration:
     - [x] Recommended extensions
     - [x] Workspace settings
     - [x] Debug configurations
     - [x] Task definitions
   - [x] Set up Rust Analyzer settings:
     - [x] Inlay hints
     - [x] Code completion
     - [x] Type information
     - [x] Documentation
   - [x] Configure code navigation:
     - [x] Symbol search
     - [x] Go to definition
     - [x] Find references
     - [x] Outline view
   - [x] Document IDE setup:
     - [x] Installation guide
     - [x] Extension setup
     - [x] Configuration options
     - [x] Troubleshooting
   
   *Updated at: April 24, 2024 - Completed VS Code configuration with enhanced settings for Rust development including customized file nesting, project-specific theming, todo tracking, and spell checking with domain-specific terms.*

### Phase 2: Debugging and Observability
1. **Request Debugging**
   - [ ] Implement structured logging:
     - [ ] Request/response logging
     - [ ] Error tracking
     - [ ] Performance metrics
     - [ ] Security events
   - [ ] Create request tracing:
     - [ ] Correlation IDs
     - [ ] Span tracking
     - [ ] Service dependencies
     - [ ] Error context
   - [ ] Add timing annotations:
     - [ ] Request duration
     - [ ] Database queries
     - [ ] External calls
     - [ ] Cache operations
   - [ ] Implement correlation:
     - [ ] Request chaining
     - [ ] Error correlation
     - [ ] User session tracking
     - [ ] Service dependencies
   
   *Updated at: Not started*

2. **Error Handling**
   - [ ] Create error messages:
     - [ ] Detailed error context
     - [ ] Stack traces
     - [ ] Cause chain
     - [ ] Recovery suggestions
   - [ ] Implement error catalog:
     - [ ] Error categories
     - [ ] Error codes
     - [ ] Documentation
     - [ ] Examples
   - [ ] Add error context:
     - [ ] Request information
     - [ ] User context
     - [ ] System state
     - [ ] Dependencies
   - [ ] Build error reporting:
     - [ ] Error aggregation
     - [ ] Alert generation
     - [ ] Error patterns
     - [ ] Impact analysis
   
   *Updated at: Not started*

3. **Database Tools**
   - [ ] Implement migrations:
     - [ ] Version control
     - [ ] Rollback support
     - [ ] Data preservation
     - [ ] Schema validation
   - [ ] Create database tools:
     - [ ] Schema reset
     - [ ] Data seeding
     - [ ] Backup/restore
     - [ ] Query analysis
   - [ ] Add query logging:
     - [ ] Performance metrics
     - [ ] Query plans
     - [ ] Lock analysis
     - [ ] Index usage
   - [ ] Implement debugging:
     - [ ] Transaction tracking
     - [ ] Deadlock detection
     - [ ] Connection monitoring
     - [ ] Cache analysis
   
   *Updated at: Not started*

### Phase 3: Documentation and Examples
1. **Documentation**
   - [ ] Build API documentation:
     - [ ] OpenAPI specification
     - [ ] Request/response examples
     - [ ] Authentication guide
     - [ ] Error handling
   - [ ] Create getting started:
     - [ ] Installation guide
     - [ ] Configuration guide
     - [ ] Development setup
     - [ ] First application
   - [ ] Document security:
     - [ ] Authentication setup
     - [ ] Authorization guide
     - [ ] Security best practices
     - [ ] Vulnerability handling
   - [ ] Add environment setup:
     - [ ] Prerequisites
     - [ ] Installation steps
     - [ ] Configuration guide
     - [ ] Troubleshooting
   
   *Updated at: Not started*

2. **Patterns and Examples**
   - [ ] Document Axum patterns:
     - [ ] Route organization
     - [ ] Middleware usage
     - [ ] Error handling
     - [ ] State management
   - [ ] Create example handlers:
     - [ ] CRUD operations
     - [ ] Authentication
     - [ ] File uploads
     - [ ] WebSocket handling
   - [ ] Add implementation examples:
     - [ ] Database access
     - [ ] Cache usage
     - [ ] External APIs
     - [ ] Background tasks
   - [ ] Document best practices:
     - [ ] Code organization
     - [ ] Error handling
     - [ ] Testing strategies
     - [ ] Performance tips
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 10% complete
- **Last Updated**: April 24, 2024
- **Next Milestone**: Local Development Environment

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
    "Test data seeded"
}

// Database reset utility
async fn reset_database() -> &'static str {
    // Reset database to clean state
    "Database reset complete"
}
```

### Docker Compose Configuration
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:14
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: app_dev
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:6
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

  localstack:
    image: localstack/localstack
    ports:
      - "4566:4566"
    environment:
      - SERVICES=s3,dynamodb,sqs
      - DEFAULT_REGION=us-west-2
      - AWS_ACCESS_KEY_ID=test
      - AWS_SECRET_ACCESS_KEY=test
    volumes:
      - localstack_data:/tmp/localstack
      - ./localstack:/docker-entrypoint-initaws.d

volumes:
  postgres_data:
  redis_data:
  localstack_data:
```

## References
- [Rust Development Tools](https://www.rust-lang.org/tools)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [LocalStack Documentation](https://docs.localstack.cloud/overview/)
- [VS Code Rust Extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Axum Documentation](https://docs.rs/axum/latest/axum/) 