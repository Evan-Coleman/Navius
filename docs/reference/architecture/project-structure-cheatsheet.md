---
title: "Navius Project Structure Cheatsheet"
description: "# Run the server"
category: guide
tags:
  - api
  - architecture
  - authentication
  - caching
  - database
  - development
  - documentation
  - integration
  - testing
last_updated: March 23, 2025
version: 1.0
---
# Navius Project Structure Cheatsheet

**Updated:** March 24, 2025

This cheatsheet provides a quick reference for navigating and extending the Navius project structure.

## Directory Structure

```
navius/
├── src/
│   ├── app/                # User-extensible application code
│   │   ├── api/            # User-defined API endpoints
│   │   ├── services/       # User-defined services
│   │   └── router.rs       # User-defined routes
│   ├── core/               # Framework core functionality (don't modify)
│   │   ├── api/            # Core API implementations
│   │   ├── auth/           # Authentication & authorization
│   │   ├── cache/          # Caching infrastructure
│   │   ├── config/         # Configuration management
│   │   ├── database/       # Database connections & transactions
│   │   ├── error/          # Error handling
│   │   ├── handlers/       # Core request handlers
│   │   ├── metrics/        # Metrics collection
│   │   ├── models/         # Core data models
│   │   ├── reliability/    # Circuit breakers, retries, etc.
│   │   ├── repository/     # Data access patterns
│   │   ├── router/         # Routing infrastructure
│   │   ├── services/       # Core business logic
│   │   └── utils/          # Utility functions
│   ├── cache/              # Cache wrappers (user-facing)
│   └── config/             # Configuration wrappers (user-facing)
├── tests/                  # Test suite
└── docs/                   # Documentation
    ├── architecture/       # Architecture documentation
    ├── guides/             # User guides
    └── roadmaps/           # Development roadmaps
```

## Quick Navigation Guide

### Adding New Features

| If you need to... | Go to... | Notes |
|-------------------|----------|-------|
| Add a new API endpoint | `src/app/api/` | Create a new module with your endpoint handlers |
| Add a new service | `src/app/services/` | Implement business logic for your feature |
| Define new routes | `src/app/router.rs` | Add your routes to the user routes section |
| Create a new model | `src/core/models/extensions.rs` | Add user models as extensions |
| Add a database repository | `src/app/repository/` | Create new repository for your data access |

### Understanding Core Components

| Component | Location | Primary Responsibility |
|-----------|----------|------------------------|
| Router | `src/core/router/` | HTTP route registration and handling |
| Auth | `src/core/auth/` | Authentication and authorization |
| Cache | `src/core/cache/` | Caching infrastructure |
| Config | `src/core/config/` | Configuration management |
| Error | `src/core/error/` | Error handling framework |
| Database | `src/core/database/` | Database connectivity |

## Common Extension Patterns

### 1. Creating a New API Endpoint

```rust
// src/app/api/your_feature.rs
use axum::{
    Json,
    extract::{Path, State},
};
use std::sync::Arc;
use crate::core::{
    error::Result,
    router::AppState,
};

pub async fn get_your_feature(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<YourModel>> {
    // Implementation
}

// Add to src/app/api/mod.rs
pub mod your_feature;

// Add to src/app/router.rs in create_user_routes function
.route("/your-feature/:id", get(api::your_feature::get_your_feature))
```

### 2. Adding a New Service

```rust
// src/app/services/your_service.rs
use crate::core::{
    error::Result,
    models::YourModel,
};

pub async fn get_your_data(id: &str) -> Result<YourModel> {
    // Implementation
}

// Add to src/app/services/mod.rs
pub mod your_service;
```

### 3. Working with AppState

```rust
// Access AppState in your handlers
use crate::core::router::AppState;
use std::sync::Arc;
use axum::extract::State;

pub async fn your_handler(
    State(state): State<Arc<AppState>>,
) -> Result<()> {
    // Access components like:
    // state.client (HTTP client)
    // state.config (Application config)
    // state.cache_registry (Cache)
    // state.db_pool (Database)
}
```

## Testing Guidelines

| Test Type | Location | Example |
|-----------|----------|---------|
| Unit Tests | Same file as implementation | `#[cfg(test)] mod tests { ... }` |
| Integration Tests | `tests/` directory | `tests/api_integration_test.rs` |
| API Tests | `tests/api/` directory | `tests/api/your_feature_test.rs` |

## Documentation References

| Documentation | Location | Purpose |
|---------------|----------|---------|
| Architecture Diagrams | `docs/architecture/diagrams/` | Visualize component relationships |
| API Reference | `docs/reference/api/` | API endpoint documentation |
| User Guides | `docs/guides/` | How-to guides for developers |
| Roadmaps | `docs/roadmaps/` | Planned features and improvements |

## Common Commands

```bash
# Run the server
cargo run

# Run tests
cargo test

# Run specific tests
cargo test your_test_name

# Generate documentation
cargo doc --open

# Check for lints
cargo clippy
```

## Getting Help

- Check the architecture diagrams in `docs/architecture/diagrams/`
- Read the specific component documentation in `docs/reference/`
- Review example implementations in `src/app/api/examples/` 

## Related Documents
- [Installation Guide](/docs/getting-started/installation.md) - How to install the application
- [Development Workflow](/docs/guides/development/development-workflow.md) - Development best practices

