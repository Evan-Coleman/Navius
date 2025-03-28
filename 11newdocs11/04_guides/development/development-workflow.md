---
title: Navius Development Workflow
description: Guide to the development process and workflow for Navius applications
category: guides
tags:
  - development
  - workflow
  - testing
  - debugging
related:
  - testing.md
  - project-navigation.md
  - ../../getting-started/development-setup.md
last_updated: March 23, 2025
version: 1.0
---

# Navius Development Workflow

## Overview
This guide explains the standard development workflow for working on Navius applications. It covers running the server, testing, code style, and debugging processes to help you develop efficiently.

## Prerequisites
Before following this workflow guide, ensure you have:

- Completed the [Installation Guide](../../getting-started/installation.md)
- Set up your development environment following the [Development Setup Guide](../../getting-started/development-setup.md)
- Basic knowledge of Rust and command-line tools

## Step-by-step Workflow

### 1. Running the Server

The Navius framework provides several ways to run the development server:

```bash
# Using the development script (recommended)
./run_dev.sh

# Regular run with Cargo
cargo run

# Development mode with auto-reload
cargo watch -x run
```

The development script (`run_dev.sh`) offers several options:

```bash
./run_dev.sh [OPTIONS]
```

Options include:
- `--skip-gen` - Skip API model generation
- `--release` - Build and run in release mode
- `--config-dir=DIR` - Use specified config directory (default: config)
- `--env=FILE` - Use specified .env file (default: .env)
- `--environment=ENV` - Use specified environment (default: development)
- `--port=PORT` - Specify server port (default: 3000)
- `--watch` - Restart server on file changes
- `--run-migrations` - Run database migrations before starting
- `--no-health-check` - Skip health check validation after startup

### 2. Testing Your Code

Navius emphasizes thorough testing. Use these commands for different testing scenarios:

```bash
# Run all tests
cargo test

# Run only library tests
cargo test --lib

# Run tests in a specific module
cargo test core::config

# Generate test coverage report (HTML)
cargo tarpaulin --out Html

# Generate JSON coverage report
cargo tarpaulin -o Json --output-file target/@navius-coverage.json
```

For test coverage, we aim for:
- Overall project: 80%+ code coverage
- Critical modules: 90%+ coverage
- Helper/utility functions: 70%+ coverage

### 3. Code Style and Formatting

Navius follows standard Rust code style enforced by rustfmt. Before committing your changes:

```bash
# Format your code
cargo fmt

# Check for warnings and errors
cargo clippy
```

Follow these code style principles:
- Use descriptive variable and function names
- Add documentation comments to public functions
- Follow the standard Rust naming conventions
- Use proper error handling rather than unwrap/expect in production code

### 4. Making Changes

When implementing new features or fixing bugs:

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Implement with tests**
   - Add tests for your changes
   - Ensure they cover both success and error paths

3. **Verify your changes locally**
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

4. **Commit your changes**
   ```bash
   git add .
   git commit -m "Description of your changes"
   ```

5. **Submit a pull request** (if working in a team)

### 5. Debugging

For effective debugging during development:

1. **Configure logging levels** in your `.env` file:
   ```
   LOG_LEVEL=debug
   LOG_FORMAT=text
   ```

2. **Use the Rust debugger**:
   - VS Code: Use the Rust Analyzer extension with the LLDB debugger
   - CLion: Use the built-in debugger for Rust

3. **Add tracing** with the tracing crate:
   ```rust
   use tracing::{debug, error, info, warn};
   
   info!("Processing request for {}", user_id);
   debug!("Request details: {:?}", request);
   ```

## Common Issues and Solutions

### Build Problems

If you encounter missing dependency errors or other build issues:

```bash
# Clean and rebuild
cargo clean
cargo build
```

### Test Failures

When tests fail due to issues with global state:
- Use mock implementations where possible
- Ensure tests clean up after themselves
- Consider using test-specific initialization

### Configuration Issues

If your application can't find configuration values:
- Check that your `.env` file exists and contains the required values
- Verify the config files in the `config/` directory
- Use the correct environment flag with the development script

## Next Steps

After understanding the basic development workflow, you might want to explore:

- [Project Navigation](project-navigation.md) - Understanding the project structure
- [Testing Guide](testing.md) - Writing comprehensive tests
- [API Integration](../features/api-integration.md) - Working with APIs
- [Authentication](../features/authentication.md) - Implementing authentication

## Related Documents

- [Development Setup](../../getting-started/development-setup.md) - Setting up your development environment
- [Project Structure](../../reference/architecture/project-structure.md) - Overview of the project's architecture
- [Testing Guide](testing.md) - In-depth guide to testing Navius applications 