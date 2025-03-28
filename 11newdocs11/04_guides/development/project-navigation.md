---
title: Navigating the Navius Project Structure
description: Guide to understanding and navigating the Navius project organization
category: guides
tags:
  - development
  - architecture
  - project-structure
  - organization
related:
  - development-workflow.md
  - testing.md
  - ../../reference/architecture/directory-organization.md
last_updated: March 27, 2025
version: 1.0
---

# Navigating the Navius Project Structure

## Overview
This guide explains the organization of the Navius project, helping developers navigate the codebase efficiently. Understanding the project structure is essential for making contributions, adding new features, and tracking down bugs.

## Prerequisites
Before using this guide, you should have:

- Basic understanding of Rust project structures
- Familiarity with package management in Rust (Cargo)
- Navius development environment set up

## Directory Structure

The Navius framework follows a modular directory structure designed to separate concerns and facilitate ease of development:

```
navius/
├── .github/            # GitHub workflows and templates
├── assets/             # Static assets for the project
├── bin/                # Binary executables
├── docs/               # Documentation
├── examples/           # Example applications
├── scripts/            # Utility scripts
├── src/                # Source code
│   ├── app/            # Application-specific code
│   ├── core/           # Core framework components
│   ├── config/         # Configuration management
│   ├── db/             # Database interactions
│   ├── api/            # API definitions
│   ├── middleware/     # HTTP middleware components
│   ├── auth/           # Authentication and authorization
│   ├── utils/          # Utility functions and helpers
│   └── lib.rs          # Library entry point
├── tests/              # Integration tests
├── benches/            # Performance benchmarks
├── Cargo.toml          # Main package manifest
├── Cargo.lock          # Package dependency lock file
└── README.md           # Main project documentation
```

Let's explore each of these directories in detail:

## Core Directories

### src/
The main source code directory contains all Rust code for the Navius framework.

#### src/app/
Contains application-specific code that defines the business logic of the framework:

- `controllers/`: Request handlers that process API requests
- `models/`: Data structures representing domain entities
- `services/`: Business logic implementation
- `router.rs`: Application routing configuration

#### src/core/
Contains the core framework components:

- `server.rs`: HTTP server implementation
- `application.rs`: Application bootstrapping and lifecycle
- `context.rs`: Request context management
- `error.rs`: Error handling infrastructure
- `types.rs`: Common type definitions

#### src/config/
Manages application configuration:

- `settings.rs`: Configuration settings structure
- `environment.rs`: Environment-specific configuration
- `loader.rs`: Configuration loading from files/env

#### src/db/
Database interaction layer:

- `connection.rs`: Database connection management
- `migrations/`: Schema migration files
- `repositories/`: Data access implementations
- `models/`: Database-specific data models

#### src/api/
API definition and documentation:

- `routes/`: API route definitions
- `schemas/`: API request/response schemas
- `documentation.rs`: API documentation generation
- `validation.rs`: Request validation logic

#### src/middleware/
HTTP middleware components:

- `logger.rs`: Request logging middleware
- `cors.rs`: Cross-Origin Resource Sharing
- `authentication.rs`: Authentication middleware
- `cache.rs`: Response caching middleware
- `rate_limit.rs`: Rate limiting implementation

#### src/auth/
Authentication and authorization:

- `jwt.rs`: JWT token handling
- `oauth.rs`: OAuth integration
- `permissions.rs`: Permission management
- `providers/`: Authentication providers

#### src/utils/
Utility functions and helpers:

- `string.rs`: String manipulation utilities
- `time.rs`: DateTime handling utilities
- `hash.rs`: Hashing functionality
- `validator.rs`: Data validation helpers

### tests/
Contains integration tests organized by module:

- `api/`: API endpoint tests
- `db/`: Database integration tests
- `auth/`: Authentication tests
- `common/`: Shared test utilities

### bin/
Contains binary executable entry points:

- `server.rs`: Main application server
- `cli.rs`: Command-line interface utility
- `migrate.rs`: Database migration tool

## Supporting Directories

### docs/
Project documentation:

- `getting-started/`: Onboarding guides
- `guides/`: Development guides
- `reference/`: API and architecture reference
- `contributing/`: Contribution guidelines

### scripts/
Utility scripts for development and deployment:

- `setup.sh`: Environment setup script
- `run_dev.sh`: Development server script
- `build.sh`: Production build script
- `test.sh`: Test runner script

### examples/
Example applications demonstrating Navius usage:

- `basic/`: Basic API example
- `auth/`: Authentication example
- `websocket/`: WebSocket implementation
- `database/`: Database interaction example

## Key Files

### Cargo.toml
The main package manifest containing:

- Package metadata
- Dependencies
- Build configuration
- Feature flags

### lib.rs
The library entry point that:

- Defines module structure
- Exposes public API
- Sets up global state

## Finding Your Way Around

### Where to Add New Features

When adding new features to Navius, consider these guidelines:

1. **New API endpoints**: Add to `src/api/routes/` and register in `src/app/router.rs`
2. **New models**: Add to `src/app/models/` and corresponding repository in `src/db/repositories/`
3. **Business logic**: Add to `src/app/services/`
4. **Utility functions**: Add to appropriate module in `src/utils/`
5. **Database migrations**: Add to `src/db/migrations/`

### Where to Fix Bugs

When fixing bugs, focus on these areas based on the issue type:

1. **API issues**: Check `src/api/routes/` and `src/app/controllers/`
2. **Database issues**: Look in `src/db/repositories/` and `src/db/models/`
3. **Authentication issues**: Examine `src/auth/` and `src/middleware/authentication.rs`
4. **Configuration issues**: Review `src/config/`

### Debugging Locations

Common locations for adding debugging code:

1. **Request lifecycle**: `src/core/server.rs` and `src/middleware/logger.rs`
2. **Database operations**: `src/db/connection.rs` and specific repositories
3. **API request handling**: Relevant controller in `src/app/controllers/`

## Architecture Highlights

### Dependency Injection

Navius uses a dependency injection pattern to make components testable:

```rust
// Service definition
pub struct UserService<R: UserRepository> {
    repository: R,
}

// Service implementation
impl<R: UserRepository> UserService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    
    pub async fn get_user(&self, id: Uuid) -> Result<User, Error> {
        self.repository.find_by_id(id).await
    }
}
```

### Middleware Pipeline

HTTP requests flow through a middleware pipeline defined in `src/core/server.rs`:

```rust
// Building the middleware pipeline
let app = Router::new()
    .route("/health", get(health_handler))
    .nest("/api", api_router)
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())
    .layer(AuthenticationLayer::new())
    .layer(RequestIdLayer::new());
```

### Error Handling

Errors are handled using a centralized error type defined in `src/core/error.rs`:

```rust
pub enum AppError {
    NotFound,
    Unauthorized,
    BadRequest(String),
    Internal(anyhow::Error),
    ValidationError(Vec<ValidationError>),
    DatabaseError(DatabaseError),
}
```

## Related Documents

- [Development Workflow](development-workflow.md) - Development process overview
- [Testing Guide](testing.md) - How to test different parts of the application
- [Architecture Principles](../../reference/architecture/principles.md) - Core architectural principles
- [API Design](../features/api-design.md) - Guide to designing APIs in Navius 