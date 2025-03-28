---
title: "Navius Development Guide"
description: "# Using the development script (recommended)"
category: guide
tags:
  - api
  - architecture
  - development
  - documentation
  - integration
  - performance
  - testing
last_updated: March 23, 2025
version: 1.0
---
# Navius Development Guide

This document provides instructions for setting up your development environment and contributing to the Navius project.

## 🚀 Quick Setup

The easiest way to set up your development environment is to use the provided setup script:

```sh
./scripts/setup.sh
```

This will:
- Install Rust if it's not already installed
- Install development tools (cargo-watch, cargo-tarpaulin)
- Create a sample .env file if one doesn't exist
- Build the project and run tests to verify everything works

### Manual Setup

If you prefer to set up manually:

1. Install Rust: https://www.rust-lang.org/tools/install
2. Install development tools:
   ```sh
   cargo install cargo-watch
   cargo install cargo-tarpaulin
   ```
3. Create a `.env` file with the necessary configuration

## 💻 Development Workflow

### Running the Server

```sh
# Using the development script (recommended)
./run_dev.sh

# Regular run
cargo run

# Development mode with auto-reload
cargo watch -x run
```

### Running Tests

```sh
# Run all tests
cargo test

# Run only library tests
cargo test --lib

# Run tests in a specific module
cargo test core::config

# Generate test coverage report
cargo tarpaulin --out Html

# Generate JSON coverage report (for integration with other tools)
cargo tarpaulin -o Json --output-file target/@navius-coverage.json
```

## 📐 Code Style and Formatting

Navius follows the standard Rust code style enforced by rustfmt. Before committing, make sure to format your code:

```sh
cargo fmt
```

And check for warnings and errors with Clippy:

```sh
cargo clippy
```

## 🔄 Continuous Integration

The project uses GitHub Actions and GitLab CI for continuous integration. The workflow includes:
- Running tests
- Checking code style with rustfmt
- Linting with Clippy
- Generating code coverage reports

## 📁 Project Structure

Navius follows a modular architecture inspired by enterprise Java frameworks but with Rust's performance and safety:

- `src/` - Source code
  - `core/` - Framework core functionality (auth, config, metrics, etc.)
  - `api/` - API endpoints
  - `services/` - Business logic 
  - `repository/` - Data access layer
  - `models/` - Domain models
  - `app/` - Application-specific code
- `tests/` - Integration tests
- `scripts/` - Utility scripts
- `docs/` - Documentation
- `config/` - Configuration files

## 🧩 Adding New Features

1. Create a new branch from `main`
2. Implement the feature with tests
3. Ensure all tests pass and the code follows the style guidelines
4. Submit a pull request

## 🐛 Debugging

For detailed logging during development, set the following in your `.env` file:

```
LOG_LEVEL=debug
LOG_FORMAT=text
```

## 🔍 Common Issues

### Missing Dependencies

If you encounter "missing dependency" errors, try:

```sh
cargo clean
cargo build
```

### Test Failures

When tests fail due to issues with the global state (like metrics recorder):
- Use mock implementations where possible
- Ensure tests clean up after themselves
- Consider using test-specific initialization functions

## 💡 Architecture Principles

Navius follows these key architectural principles:

1. **Separation of Concerns**: Clear boundaries between layers
2. **Type Safety**: Leverage Rust's type system to catch errors at compile time
3. **Async First**: Built for asynchronous processing from the ground up
4. **Explicit Over Implicit**: Favor explicit code over "magic" behavior
5. **Testing First**: Every component designed with testability in mind

## 🤝 Need Help?

If you need assistance with development:
- Check the existing documentation
- Review the example code in `examples/`
- Reach out to the project maintainers
- Open an issue on GitHub/GitLab 

## Related Documents
- [Installation Guide](../../01_getting_started/installation.md) - How to install the application
- [Development Workflow](development-workflow.md) - Development best practices

