---
title: Navius CLI Reference
description: Comprehensive reference for the Navius command-line interface
category: getting-started
tags:
  - cli
  - reference
  - commands
  - development
  - tooling
related:
  - installation.md
  - development-setup.md
  - first-steps.md
last_updated: March 27, 2025
version: 1.0
status: active
---

# Navius CLI Reference

## Overview

This document provides a comprehensive reference for the Navius command-line interface (CLI), including all available commands, options, environment variables, and exit codes.

## Prerequisites

- Rust and Cargo installed
- Navius project cloned and dependencies installed (see [Installation Guide](./))

## Quick Start

Most commonly used commands:

```shell
# Run the application with default settings
cargo run

# Run in release mode
cargo run --release

# Run with specific features
cargo run --features "feature1,feature2"

# Run tests
cargo test
```

## Basic Commands

### Run Application

```bash
cargo run
```

Starts the application with default configuration.

### Run with Specific Feature Flags

```bash
cargo run --features "feature1,feature2"
```

Runs the application with specific features enabled.

### Development Mode

```bash
cargo run --features "dev"
```

Runs the application in development mode, which enables additional logging and development tooling.

## Configuration Commands

### Using Custom Configuration

```bash
cargo run -- --config-path=/path/to/config.yaml
```

Starts the application with a custom configuration file.

### Environment Override

```bash
ENV_VAR=value cargo run
```

Runs the application with environment variable overrides.

## Testing Commands

### Run All Tests

```bash
cargo test
```

Runs all tests in the application.

### Run Specific Tests

```bash
cargo test test_name
```

Runs tests matching the specified name.

### Run Tests with Coverage

```bash
cargo tarpaulin --out Html
```

Runs tests and generates a coverage report.

## Build Commands

### Debug Build

```bash
cargo build
```

Builds the application in debug mode.

### Release Build

```bash
cargo build --release
```

Builds the application in release mode, with optimizations enabled.

### Build Documentation

```bash
cargo doc --no-deps --open
```

Builds and opens the API documentation.

## Linting and Formatting

### Check Code Style

```bash
cargo clippy
```

Checks the code for style issues and common mistakes.

### Format Code

```bash
cargo fmt
```

Formats the code according to the Rust style guidelines.

## Database Commands

### Run Migrations

```bash
cargo run --bin migrate
```

Runs database migrations to set up or update the database schema.

### Reset Database

```bash
cargo run --bin reset-db
```

Resets the database (warning: this will delete all data).

## Advanced Commands

### Generate Offline SQLx Data

```bash
cargo sqlx prepare
```

Generates SQLx data for offline compilation.

### Analyze Binary Size

```bash
cargo bloat --release
```

Analyzes the binary size to identify large dependencies.

## Key Concepts

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Controls log level | `info` |
| `CONFIG_PATH` | Path to configuration file | `config/default.yaml` |
| `DATABASE_URL` | Database connection string | Configured in YAML |
| `PORT` | Server port | `3000` |
| `HOST` | Server host | `127.0.0.1` |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Database connection error |
| 4 | Permission error |

### Script Files

Navius provides several convenience scripts in the project root:

| Script | Purpose |
|--------|---------|
| `run_dev.sh` | Runs the application in development mode with hot reloading |
| `run_prod.sh` | Runs the application in production mode |
| `test.sh` | Runs tests with common options |
| `reset_db.sh` | Resets the database and reruns migrations |

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Command not found | Make sure you're in the project root directory |
| Permission denied | Check file permissions or use `sudo` if appropriate |
| Dependency errors | Run `cargo update` to update dependencies |
| Test failures | Check error messages and review related code |

## Next Steps

- Review the [Installation Guide](./) for setup instructions
- See [Development Setup](./) for creating a development environment
- Learn about basic concepts in [First Steps](first-steps.md) 