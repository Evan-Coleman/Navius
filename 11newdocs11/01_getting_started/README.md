---
title: Getting Started with Navius
description: "Complete introduction and quick start guides for the Navius framework, including installation, setup, and building your first application"
category: getting-started
tags:
  - introduction
  - installation
  - setup
  - quickstart
  - development
  - tutorial
related:
  - installation.md
  - development-setup.md
  - first-steps.md
  - hello-world.md
  - ../04_guides/development/development-workflow.md
  - ../05_reference/architecture/principles.md
last_updated: March 27, 2025
version: 1.1
status: active
---

# Getting Started with Navius

## Overview

Welcome to Navius! This section provides everything you need to start building high-performance, maintainable applications with the Navius framework. Whether you're new to Rust or an experienced developer, these guides will help you quickly set up your environment and build your first application.

Navius is a modern, opinionated web framework for Rust that combines the performance benefits of Rust with the developer experience of frameworks like Spring Boot. It provides built-in support for dependency injection, configuration management, API development, and more.

## Quick Navigation

- [Installation Guide](installation.md) - Set up Navius and its dependencies
- [Development Setup](development-setup.md) - Configure your development environment
- [First Steps](first-steps.md) - Create your first Navius application
- [Hello World Tutorial](hello-world.md) - Build a simple REST API

## Getting Started in 5 Minutes

For experienced developers who want to dive right in:

```bash
# Install Navius (requires Rust 1.70+)
git clone https://github.com/your-organization/navius.git
cd navius

# Build the framework
cargo build

# Run the development server
./run_dev.sh

# Create a new project (optional)
cargo new --bin my-navius-app
cd my-navius-app

# Add Navius dependency to Cargo.toml
# [dependencies]
# navius = { path = "../navius" }
# tokio = { version = "1", features = ["full"] }
# axum = "0.6"
```

## Prerequisites

Before you begin with Navius, ensure you have:

- **Rust** (version 1.70.0 or later)
  - Install from [rust-lang.org](https://www.rust-lang.org/tools/install)
  - Verify with `rustc --version`

- **Development Environment**
  - A code editor or IDE (VS Code or JetBrains CLion recommended)
  - Git for version control
  - Terminal/command-line access

- **Recommended Knowledge**
  - Basic Rust programming concepts
  - Familiarity with web development concepts (HTTP, REST, APIs)
  - Understanding of asynchronous programming principles

## Installation Options

Navius offers multiple installation methods to fit your workflow:

### Option 1: Using Cargo (Simplest)

```bash
cargo install navius
```

This installs the Navius CLI tool, allowing you to create and manage Navius projects.

### Option 2: From Source (Recommended for Development)

```bash
git clone https://github.com/your-organization/navius.git
cd navius
cargo install --path .
```

This approach gives you access to the latest features and allows you to contribute to the framework.

### Option 3: As a Dependency in Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
navius = "0.1.0"
tokio = { version = "1", features = ["full"] }
axum = "0.6.0"
```

## Recommended Development Setup

For the best development experience, we recommend:

### 1. Development Tools

- **VS Code** with these extensions:
  - rust-analyzer
  - Even Better TOML
  - crates
  - LLDB Debugger

- **Terminal Tools**:
  - `cargo-watch` for auto-reloading (`cargo install cargo-watch`)
  - `cargo-expand` for macro debugging (`cargo install cargo-expand`)
  - `cargo-edit` for dependency management (`cargo install cargo-edit`)

### 2. Environment Setup

- **Docker** for containerized development (databases, Redis, etc.)
- **Git** with pre-commit hooks (as described in [Development Setup](development-setup.md))
- **Environment Configuration** (custom `.env` files for different environments)

See the [Development Setup](development-setup.md) guide for detailed instructions.

## Learning Path

We recommend following this path to learn Navius effectively:

### 1. Basic Concepts (Start Here)

- Complete the [Installation Guide](installation.md)
- Set up your environment with [Development Setup](development-setup.md)
- Build your first app with [First Steps](first-steps.md)
- Try the [Hello World Tutorial](hello-world.md)

### 2. Core Framework Concepts

- Learn about dependency injection and service architecture
- Understand configuration management
- Explore routing and middleware
- Master error handling and logging

### 3. Advanced Topics

- Database integration with SQLx or Diesel
- Authentication and authorization
- Testing strategies
- Deployment considerations

## Common Tasks

Here's a quick reference for common Navius development tasks:

### Create a New Navius Application

```bash
# With Navius CLI
navius new my-project
cd my-project

# Or manually with Cargo
cargo new --bin my-project
cd my-project
# Then add Navius to dependencies
```

### Run Your Navius Application

```bash
# Using the development script
./run_dev.sh

# With hot reloading
./run_dev.sh --watch

# Manually with cargo
cargo run
```

### Test Your Application

```bash
# Run all tests
cargo test

# Run specific tests
cargo test --package navius --lib -- app::hello::tests

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Build for Production

```bash
# Build optimized binary
cargo build --release

# Run in production mode
./run_prod.sh
```

## Navius Framework Structure

Understanding the framework structure helps navigate the documentation:

```
navius/
├── src/
│   ├── app/       # Your application code goes here
│   ├── core/      # Framework core components
│   ├── lib.rs     # Library definition
│   └── main.rs    # Entry point
├── config/        # Configuration files
├── tests/         # Integration tests
└── docs/          # Documentation
```

## Key Concepts

Navius is built around these core principles:

1. **Modularity**: Components are organized into cohesive modules
2. **Dependency Injection**: Services are registered and injected where needed
3. **Configuration-Driven**: Application behavior is controlled via configuration
4. **Convention over Configuration**: Sensible defaults with flexibility to override
5. **Testability**: First-class support for testing at all levels

## Troubleshooting

If you encounter issues during setup:

| Issue | Solution |
|-------|----------|
| **Build failures** | Ensure you have the correct Rust version and dependencies |
| **Missing libraries** | Check OS-specific requirements in the [Installation Guide](installation.md) |
| **Configuration errors** | Verify your config files match the expected format |
| **Runtime errors** | Check logs and ensure all required services are running |

## Support Resources

Need help with Navius?

- **Documentation**: Comprehensive guides in this documentation site
- **Community**: Join our [Discord community](https://discord.gg/navius)
- **GitHub Issues**: Report bugs or suggest features on our [repository](https://github.com/your-organization/navius)
- **Stack Overflow**: Ask questions with the `navius` tag

## Contributing

We welcome contributions to Navius! Here's how to get involved:

1. Read our [Contributing Guidelines](../03_contributing/README.md)
2. Set up your development environment
3. Pick an issue from our tracker or propose a new feature
4. Submit a pull request with your changes

## Next Steps

Ready to explore more?

- [Examples](../02_examples/README.md) - See Navius in action with practical examples
- [Guides](../04_guides/README.md) - In-depth guides on specific features
- [Reference](../05_reference/README.md) - Detailed API and architecture reference
- [Roadmap](../98_roadmaps/README.md) - See what's coming in future releases 