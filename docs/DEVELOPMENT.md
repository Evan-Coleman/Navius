# Development Guide

This document provides instructions for setting up your development environment and contributing to the Rust Backend project.

## Setup

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

## Development Workflow

### Running the Server

```sh
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
```

## Code Style and Formatting

We follow the standard Rust code style enforced by rustfmt. Before committing, make sure to format your code:

```sh
cargo fmt
```

And check for warnings and errors with Clippy:

```sh
cargo clippy
```

## Continuous Integration

The project uses GitHub Actions for CI/CD. The workflow includes:
- Running tests
- Checking code style with rustfmt
- Linting with Clippy
- Generating code coverage reports

## Project Structure

- `src/` - Source code
  - `core/` - Core functionality (auth, config, metrics, etc.)
  - `api/` - API endpoints
  - `services/` - Business logic
  - `repository/` - Data access
  - `models/` - Domain models
- `tests/` - Integration tests
- `scripts/` - Utility scripts
- `docs/` - Documentation

## Adding New Features

1. Create a new branch from `main`
2. Implement the feature with tests
3. Ensure all tests pass and the code follows the style guidelines
4. Submit a pull request

## Debugging

For detailed logging during development, set the following in your `.env` file:

```
LOG_LEVEL=debug
LOG_FORMAT=text
```

## Common Issues

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

## Need Help?

If you need assistance with development, check the existing documentation or reach out to the project maintainers. 