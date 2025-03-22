# Navius Project Structure Map

**Updated At:** March 25, 2024  

This document provides a comprehensive map of the Navius project structure to help developers navigate the codebase more efficiently.

## Directory Structure Overview

```
navius/
├── .devtools/              # Development tools and configurations
│   ├── coverage/           # Test coverage tools and reports
│   ├── github/             # GitHub-specific configurations
│   ├── gitlab/             # GitLab-specific configurations (excluding CI)
│   └── scripts/            # Development and build scripts
├── config/                 # Application configuration files
│   ├── default.yaml        # Default configuration
│   ├── development.yaml    # Development environment configuration
│   ├── production.yaml     # Production environment configuration
│   ├── api_registry.json   # API generation registry
│   └── swagger/            # API definitions in OpenAPI format
├── docs/                   # Project documentation
│   ├── architecture/       # Architecture documentation
│   ├── contributing/       # Contribution guidelines
│   ├── guides/             # User and developer guides
│   ├── reference/          # API and technical reference
│   └── roadmaps/           # Development roadmaps
├── migrations/             # Database migration files
├── src/                    # Source code
│   ├── app/                # User-extensible application code
│   ├── cache/              # Cache implementations (wrappers)
│   ├── config/             # Configuration implementations (wrappers)
│   ├── core/               # Core business logic and implementations
│   │   ├── api/            # API implementations
│   │   ├── auth/           # Authentication functionality
│   │   ├── cache/          # Cache management
│   │   ├── config/         # Configuration management
│   │   ├── database/       # Database access
│   │   ├── error/          # Error handling
│   │   ├── metrics/        # Metrics collection
│   │   ├── reliability/    # Circuit breakers, timeouts, retries
│   │   ├── repository/     # Data repositories
│   │   ├── router/         # Routing definitions
│   │   ├── services/       # Business services
│   │   └── utils/          # Utility functions
│   └── generated_apis.rs   # Bridge to generated API code
├── target/                 # Build artifacts
│   └── generated/          # Generated API code
├── tests/                  # Test suite
│   ├── integration/        # Integration tests
│   └── common/             # Common test utilities
├── .env                    # Environment variables (for development)
├── .gitlab-ci.yml          # GitLab CI/CD configuration
├── build.rs               # Build script
├── Cargo.toml             # Rust dependencies and project configuration
└── README.md              # Project overview
```

## Core Components and Their Responsibilities

### 1. Core Module Structure (`src/core/`)

The core module contains the central business logic and implementations:

- **api**: Contains the API endpoints and handlers
- **auth**: Authentication and authorization functionality
- **cache**: Cache management and provider integration
- **config**: Configuration management and parsing
- **database**: Database connections and query execution
- **error**: Error types and handling utilities
- **metrics**: Metrics collection and reporting
- **reliability**: Circuit breakers, rate limiting, and retry logic
- **repository**: Data access layer
- **router**: API route definitions
- **services**: Business service implementations
- **utils**: Shared utility functions

### 2. User-Facing Components (`src/app/`)

User-extensible scaffolding that allows developers to extend the application:

- **router.rs**: User-defined routes and endpoints
- **auth_config.rs**: Authentication configuration
- **api**: User-defined API endpoints
- **services**: User-defined service implementations

### 3. Generated Code (`target/generated/`)

Auto-generated API clients and models:

- **[api_name]_api/**: Generated API client code for each API
- **openapi/**: OpenAPI schemas and configurations

## Major Dependencies and Integrations

- **Axum**: Web framework
- **Tokio**: Asynchronous runtime
- **SQLx**: Database access
- **Redis**: Cache provider
- **AWS**: Cloud services integration

## Key Design Patterns

1. **Clean Architecture**: Separation of concerns with core business logic isolated
2. **Repository Pattern**: Data access abstraction
3. **Dependency Injection**: Through function parameters and context
4. **Circuit Breaker Pattern**: For resilient external service calls
5. **Middleware Pattern**: For cross-cutting concerns

## Common Development Workflows

### 1. Adding a New API Endpoint

1. Define the route in `src/app/router.rs`
2. Implement the handler in `src/app/api/`
3. Add any needed services in `src/app/services/`
4. Add tests in `tests/integration/`

### 2. Working with Generated API Clients

1. Update API definitions in `config/swagger/`
2. Run `.devtools/scripts/regenerate_api.sh` or `cargo build` (automatic generation)
3. Import the generated code through `src/generated_apis.rs`

### 3. Updating Configuration

1. Modify the appropriate YAML file in `config/`
2. Access the configuration through the `config::get_config()` function

## Common File Locations

| What You're Looking For | Where to Find It |
|-------------------------|------------------|
| Main entry point | `src/main.rs` |
| Core application logic | `src/lib.rs` |
| API routes | `src/core/router/`, `src/app/router.rs` |
| Error handling | `src/core/error/` |
| Database access | `src/core/database/` |
| Authentication | `src/core/auth/` |
| Generated API clients | `target/generated/` |
| Configuration | `config/` |
| Scripts | `.devtools/scripts/` |
| Tests | `tests/` |

## Module Dependencies

Key dependencies between modules:

- **api** → depends on → **services**, **repository**, **error**
- **services** → depends on → **repository**, **error**
- **repository** → depends on → **database**, **error**
- **router** → depends on → **api**, **auth**
- **auth** → depends on → **error**, **config**

## Navigation Tips

1. Use the module declarations in `src/lib.rs` to understand the module hierarchy
2. Leveraging the repository pattern helps to understand data flow
3. Follow API routes to find handler implementations
4. Use `src/generated_apis.rs` as the entry point for working with external APIs 