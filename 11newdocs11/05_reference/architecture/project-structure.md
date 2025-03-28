---
title: "Navius Project Structure"
description: "Documentation about Navius Project Structure"
category: architecture
tags:
  - api
  - architecture
  - authentication
  - aws
  - caching
  - database
  - development
  - documentation
  - integration
  - redis
  - testing
last_updated: March 23, 2025
version: 1.0
---
# Navius Project Structure

**Updated At:** March 23, 2025

This document provides a comprehensive guide to the Navius project structure, helping developers understand how the codebase is organized and how different components work together.

## Directory Structure Overview

```
navius/
├── .devtools/              # Development tools and configurations
│   ├── coverage/           # Test coverage tools and reports
│   ├── github/             # GitHub-specific configurations
│   ├── gitlab/             # GitLab-specific configurations (excluding CI)
│   ├── ide/                # IDE configurations (VS Code, IntelliJ, etc.)
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
│   │   ├── api/            # User-defined API endpoints
│   │   ├── services/       # User-defined services
│   │   └── router.rs       # User-defined routes
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
├── build.rs                # Build script
├── Cargo.toml              # Rust dependencies and project configuration
└── README.md               # Project overview
```

## Version Control Strategy

Navius uses a dual VCS approach:

- **GitLab (Primary)**: Business operations, CI/CD, issue tracking, code review workflow
- **GitHub (Secondary)**: Public visibility, community engagement, documentation accessibility

### Repository Sync Strategy

Repositories are synchronized using GitLab's mirroring feature:
- Automatic one-way sync from GitLab → GitHub after successful builds
- Production code and releases are pushed to GitHub only after validation

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
- **api/**: User-defined API endpoints
- **services/**: User-defined service implementations

### 3. Generated Code (`target/generated/`)

Auto-generated API clients and models:

- **[api_name]_api/**: Generated API client code for each API
- **openapi/**: OpenAPI schemas and configurations

## Module Organization and Dependencies

Navius follows a modular architecture with clean separation of concerns:

1. **HTTP Layer** (API): Defines REST endpoints, handles HTTP requests/responses
2. **Business Logic** (Services): Implements core application functionality
3. **Data Access** (Repository): Manages data persistence and retrieval
4. **Domain Model** (Models): Defines data structures used across the application
5. **Infrastructure** (Core): Provides framework capabilities like auth, caching, etc.

### Dependencies Between Modules

The dependencies between modules follow a clean architecture approach:

```
API → Services → Repository → Database
        ↓
      Models
        ↑
       Core
```

Specific component dependencies:
- **api** → depends on → **services**, **repository**, **error**
- **services** → depends on → **repository**, **error**
- **repository** → depends on → **database**, **error**
- **router** → depends on → **api**, **auth**
- **auth** → depends on → **error**, **config**

## Major Dependencies and Integrations

- **Axum**: Web framework
- **Tokio**: Asynchronous runtime
- **SQLx**: Database access
- **Redis**: Cache provider
- **AWS**: Cloud services integration
- **Microsoft Entra**: Authentication platform

## Key Design Patterns

1. **Clean Architecture**: Separation of concerns with core business logic isolated
2. **Repository Pattern**: Data access abstraction
3. **Dependency Injection**: Through function parameters and context
4. **Circuit Breaker Pattern**: For resilient external service calls
5. **Middleware Pattern**: For cross-cutting concerns

## Route Groups

- `/` - Public routes, no authentication required
- `/read` - Read-only authenticated routes
- `/full` - Full access authenticated routes
- `/actuator` - System monitoring and health checks

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

## Testing Structure

Each component type has its own testing approach:

- **Services**: Unit tests focus on business logic
- **Repositories**: Integration tests focus on data access
- **API Endpoints**: End-to-end tests focus on HTTP interactions
- **Models**: Property-based tests focus on invariants

## Navigation Tools

To help with navigating the codebase, several tools are available:

1. **Documentation**:
   - See `docs/guides/project-navigation.md` for detailed navigation guidance
   - Check `docs/architecture/module-dependencies.md` for visualizations of module dependencies

2. **Helper Scripts**:
   - Use `.devtools/scripts/navigate.sh` to find code components
   - Use `.devtools/scripts/verify-structure.sh` to validate project structure

3. **IDE Configuration**:
   - VS Code configuration is available in `.devtools/ide/vscode/`
   - Use the provided launch configurations for debugging

## Further Resources

- **Developer Onboarding**: See `docs/contributing/onboarding.md`
- **IDE Setup**: See `docs/contributing/ide-setup.md`
- **Module Dependencies**: See `docs/architecture/module-dependencies.md` for visualizations 

## Related Documents
- [Module Dependencies](module-dependencies.md) - Dependencies between modules

