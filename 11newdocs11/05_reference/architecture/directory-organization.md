---
title: Navius Directory Organization Reference
description: Detailed reference of the Navius framework directory structure and organization
category: reference
tags:
  - architecture
  - directory-structure
  - organization
related:
  - ../../guides/development/project-navigation.md
  - principles.md
  - ../standards/naming-conventions.md
last_updated: March 27, 2025
version: 1.0
---

# Navius Directory Organization Reference

## Overview
This reference document provides a detailed overview of the Navius framework's directory structure and organization. It serves as a comprehensive reference for developers to understand where different components are located within the project.

## Root Directory Structure

```
navius/
├── .github/            # GitHub workflows and templates
├── assets/             # Static assets for the project
├── bin/                # Binary executables
├── docs/               # Documentation
├── examples/           # Example applications
├── scripts/            # Utility scripts
├── src/                # Source code
├── tests/              # Integration tests
├── benches/            # Performance benchmarks
├── Cargo.toml          # Main package manifest
├── Cargo.lock          # Package dependency lock file
└── README.md           # Main project documentation
```

## Source Code Organization

The `src/` directory contains all Rust code for the Navius framework, organized by module:

```
src/
├── app/                # Application-specific code
│   ├── controllers/    # Request handlers
│   ├── models/         # Domain models
│   ├── services/       # Business logic
│   └── router.rs       # Application routing
├── core/               # Core framework components
│   ├── server.rs       # HTTP server implementation
│   ├── application.rs  # Application bootstrapping
│   ├── context.rs      # Request context management
│   ├── error.rs        # Error handling infrastructure
│   └── types.rs        # Common type definitions
├── config/             # Configuration management
│   ├── settings.rs     # Configuration settings
│   ├── environment.rs  # Environment configuration
│   └── loader.rs       # Configuration loading
├── db/                 # Database interactions
│   ├── database/
│   │   ├── connection.rs   # Database connection manager
│   │   ├── migrations/     # Schema migrations
│   │   └── models/         # Database models
│   ├── repositories/   # Data access
│   └── models/         # Database models
├── api/                # API definitions
│   ├── routes/         # API routes
│   ├── schemas/        # Request/response schemas
│   ├── documentation.rs # API documentation
│   └── validation.rs   # Request validation
├── middleware/         # HTTP middleware
│   ├── logger.rs       # Request logging
│   ├── cors.rs         # CORS handling
│   ├── authentication.rs # Authentication
│   ├── cache.rs        # Response caching
│   └── rate_limit.rs   # Rate limiting
├── auth/               # Authentication and authorization
│   ├── jwt.rs          # JWT token handling
│   ├── oauth.rs        # OAuth integration
│   ├── permissions.rs  # Permission management
│   └── providers/      # Authentication providers
├── utils/              # Utility functions
│   ├── string.rs       # String utilities
│   ├── time.rs         # DateTime utilities
│   ├── hash.rs         # Hashing functions
│   └── validator.rs    # Data validation
└── lib.rs              # Library entry point
```

## Test Directory Organization

The `tests/` directory contains integration tests organized by module:

```
tests/
├── api/                # API endpoint tests
│   ├── auth_tests.rs   # Authentication tests
│   └── user_tests.rs   # User API tests
├── db/                 # Database tests
│   └── migrations_tests.rs # Migration tests
├── common/             # Shared test utilities
│   └── test_utils.rs   # Common test helpers
└── integration_tests.rs # Main integration test module
```

## Binary Directory Organization

The `bin/` directory contains executable entry points:

```
bin/
├── server.rs           # Main application server
├── cli.rs              # Command-line interface
└── migrate.rs          # Database migration tool
```

## Scripts Directory Organization

The `scripts/` directory contains utility scripts:

```
scripts/
├── setup.sh            # Environment setup script
├── run_dev.sh          # Development server script
├── build.sh            # Production build script
├── test.sh             # Test runner script
└── add_api.sh          # API generation script
```

## Documentation Directory Organization

The `docs/` directory contains all project documentation:

```
docs/
├── getting-started/    # Onboarding guides
├── guides/             # Development guides
│   ├── development/    # Development workflow guides
│   ├── features/       # Feature implementation guides
│   └── deployment/     # Deployment guides
├── reference/          # Reference documentation
│   ├── api/            # API reference
│   ├── architecture/   # Architecture reference
│   └── standards/      # Code standards
├── roadmaps/           # Project roadmaps
├── contributing/       # Contribution guidelines
└── README.md           # Documentation index
```

## Build Artifacts

Build outputs are organized as follows:

```
target/
├── debug/              # Debug build artifacts
├── release/            # Release build artifacts
├── doc/                # Generated documentation
└── profiling/          # Profiling outputs
```

## Configuration Files

Configuration files are organized by environment:

```
config/
├── default.toml        # Default configuration
├── development.toml    # Development environment
├── test.toml           # Test environment
├── staging.toml        # Staging environment
└── production.toml     # Production environment
```

## File Naming Conventions

Navius follows these file naming conventions:

1. Source files: `snake_case.rs`
2. Directories: `snake_case/`
3. Test files: `module_name_tests.rs`
4. Binary files: `snake_case.rs`
5. Documentation files: `kebab-case.md`

## Related Documents

- [Project Navigation Guide](../../guides/development/project-navigation.md) - Guide to navigating the project structure
- [Architecture Principles](principles.md) - Core architectural principles
- [Naming Conventions](../standards/naming-conventions.md) - Naming conventions reference 