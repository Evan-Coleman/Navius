---
title: "Import Pattern Guidelines"
description: "Documentation about Import Pattern Guidelines"
category: reference
tags:
  - api
last_updated: March 23, 2025
version: 1.0
---
# Import Pattern Guidelines

This document outlines the import pattern conventions used in the Navius project after the project restructuring.

## Core Import Patterns

All imports from core modules should use the `crate::core::` prefix:

```rust
// CORRECT
use crate::core::error::{AppError, Result};
use crate::core::config::AppConfig;
use crate::core::utils::api_resource::ApiResource;

// INCORRECT
use crate::error::{AppError, Result};  // Missing core prefix
use crate::config::AppConfig;          // Missing core prefix
```

## App Import Patterns

When app modules need to import from other app modules, use the `crate::app::` prefix:

```rust
// CORRECT
use crate::app::repository::UserRepository;
use crate::app::services::UserService;

// INCORRECT
use crate::repository::UserRepository;  // Missing app prefix
use crate::services::UserService;       // Missing app prefix
```

## Importing Core Functionality in App Modules

When app modules need to import core functionality, use the `crate::core::` prefix:

```rust
// CORRECT
use crate::core::error::{AppError, Result};
use crate::core::config::AppConfig;

// INCORRECT
use crate::error::{AppError, Result};  // Should use core prefix
use crate::config::AppConfig;          // Should use core prefix
```

## Generated API Imports

For generated API code, use the `crate::generated_apis` prefix:

```rust
// CORRECT
use crate::generated_apis::petstore_api::models::Pet;

// INCORRECT
use crate::target::generated::petstore_api::models::Pet;  // Don't use target path directly
```

## Library Exports

In the root `lib.rs` file, expose only what is necessary for consumers:

```rust
// Public exports from core
pub use crate::core::router;
pub use crate::core::cache;
pub use crate::core::config;

// Public exports from app
pub use crate::app::api;
pub use crate::app::services;
```

## Automated Checking

We have implemented a script that can check and fix import patterns across the codebase:

```bash
./.devtools/scripts/fix-imports-naming.sh
```

This script automatically updates imports in core modules to use the `crate::core::` prefix and identifies potential issues in app modules.

## Benefits of Consistent Import Patterns

1. **Clarity** - Clearly distinguishes between core and app components
2. **Maintainability** - Makes code easier to maintain as the project evolves
3. **Refactoring** - Simplifies refactoring efforts
4. **Onboarding** - Makes it easier for new developers to understand the codebase structure 

## Related Documents
- [API Standards](../standards/api-standards.md) - API design guidelines
- [Error Handling](../standards/error-handling.md) - Error handling patterns

