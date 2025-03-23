---
title: "Project Navigation Guide"
description: "Documentation about Project Navigation Guide"
category: guide
tags:
  - api
  - architecture
  - authentication
  - database
  - development
  - documentation
  - integration
last_updated: March 23, 2025
version: 1.0
---
# Project Navigation Guide

**Updated At:** March 22, 2025

This guide helps developers navigate the Navius codebase efficiently by providing navigation patterns, useful commands, and explanations of key architectural elements.

## Quick Navigation

### Key Entry Points

| Component | Entry Point File | Description |
|-----------|------------------|-------------|
| Application | `src/main.rs` | Main application entry point |
| Core Library | `src/lib.rs` | Core module exports and library entry point |
| API Routes | `src/core/router/app_router.rs` | Application routing definitions |
| User API | `src/app/router.rs` | User-defined API routes |
| Configuration | `src/core/config/mod.rs` | Configuration management |
| Error Handling | `src/core/error/mod.rs` | Error types and handling |

### Module Structure

Navius follows a modular architecture with clear separation of concerns:

```
src/
├── app/              # User-extensible application code
├── core/             # Core business logic and implementations
│   ├── api/          # API implementations
│   ├── auth/         # Authentication functionality
│   ├── cache/        # Cache management
│   ├── config/       # Configuration management
│   ├── database/     # Database access
│   ├── error/        # Error handling
│   ├── metrics/      # Metrics collection
│   ├── reliability/  # Circuit breakers, timeouts, retries
│   ├── repository/   # Data repositories
│   ├── router/       # Routing definitions
│   ├── services/     # Business services
│   └── utils/        # Utility functions
└── generated_apis.rs # Bridge to generated API code
```

## Navigation Patterns

### Following the Request Flow

1. **Start with routes**: Look at `src/core/router/app_router.rs` to find route definitions
2. **Examine handlers**: Follow the route to its handler in the appropriate API module
3. **Trace to services**: API handlers call service methods in `src/core/services/`
4. **Follow to repositories**: Services use repositories in `src/core/repository/`

### Understanding Configuration

1. **Default configuration**: Check `config/default.yaml` for base configuration
2. **Environment overrides**: Look at `config/development.yaml` or `config/production.yaml`
3. **Config implementation**: See `src/core/config/mod.rs` for how configuration is loaded

### Exploring Error Handling

1. **Error types**: Start with `src/core/error/error_types.rs` to understand error categories
2. **Result extensions**: Check `src/core/error/result_ext.rs` for utility methods
3. **Error handling**: See how errors are handled in API handlers

## Useful VS Code Commands

| Command | Shortcut | Description |
|---------|----------|-------------|
| Quick Open | Ctrl+P / Cmd+P | Quickly find and open files |
| Go to Symbol | Ctrl+Shift+O / Cmd+Shift+O | Find symbols within a file |
| Go to Definition | F12 | Jump to definition of a symbol |
| Find All References | Shift+F12 | Find all references to a symbol |
| Go Back | Alt+Left / Ctrl+- | Navigate back to previous location |
| Go Forward | Alt+Right / Ctrl+Shift+- | Navigate forward |
| Search Codebase | Ctrl+Shift+F / Cmd+Shift+F | Search across all files |
| Run Tasks | Ctrl+Shift+B / Cmd+Shift+B | Run build tasks |

## Common IDE Tasks

The VS Code configuration includes several predefined tasks:

1. `cargo build`: Build the project
2. `cargo test`: Run all tests
3. `cargo run`: Run the application
4. `cargo doc`: Generate documentation
5. `cargo clippy`: Run linting checks
6. `cargo fmt`: Format code
7. `regenerate API clients`: Regenerate API clients from OpenAPI specs
8. `run dev server`: Start the development server

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

## Further Resources

- [Project Structure](docs/architecture/project-structure-map.md): Detailed project structure map
- [Core Components](docs/architecture/core-components.md): Overview of core components
- [Contribution Guidelines](docs/contributing/): How to contribute to the project 

## Related Documents
- [Installation Guide](/docs/getting-started/installation.md) - How to install the application
- [Development Workflow](/docs/guides/development/development-workflow.md) - Development best practices

