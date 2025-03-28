---
title: Getting Started with Navius
description: "Quick start guides for getting up and running with Navius, including installation, development setup, and first steps in building applications"
category: getting-started
tags:
  - getting-started
  - installation
  - setup
  - quickstart
  - development
  - tutorial
related:
  - ../guides/development/README.md
  - ../reference/architecture/principles.md
  - ../guides/features/README.md
last_updated: March 23, 2025
version: 1.0
---

# Getting Started with Navius

Welcome to Navius! This section will help you get up and running quickly with the Navius framework. Follow these guides in sequence to set up your development environment and build your first Navius application.

## Quick Start

1. [Installation](installation.md) - Install Navius and its dependencies
2. [Development Setup](development-setup.md) - Set up your development environment
3. [First Steps](first-steps.md) - Create your first Navius application

## Prerequisites

Before you begin, ensure you have:

- Rust installed (1.75.0 or later)
- A code editor (VS Code recommended)
- Basic knowledge of:
  - Rust programming language
  - Web development concepts
  - Command line usage

## Installation Options

Choose your installation method:

### Using Cargo
```bash
cargo install navius
```

### From Source
```bash
git clone https://github.com/navius/navius.git
cd navius
cargo install --path .
```

## Development Environment

We recommend:

1. **IDE Setup**
   - Visual Studio Code with rust-analyzer
   - Recommended extensions listed in [Development Setup](development-setup.md)

2. **Tools**
   - Git for version control
   - Docker for containerization
   - PostgreSQL for database
   - Redis for caching

## Learning Path

After completing the getting started guides, we recommend:

1. **Core Concepts**
   - Review [Architecture Principles](../reference/architecture/principles.md)
   - Understand [Project Structure](../reference/architecture/project-structure-recommendations.md)

2. **Essential Features**
   - Implement [Database Access](../guides/features/postgresql-integration.md)
   - Add [Authentication](../guides/features/authentication.md)
   - Set up [Caching](../guides/features/caching.md)

3. **Development Practices**
   - Follow [Development Workflow](../guides/development/development-workflow.md)
   - Learn [Testing Practices](../guides/development/testing-guide.md)

## Common Tasks

Quick reference for common tasks:

### Create a New Project
```bash
navius new my-project
cd my-project
cargo run
```

### Run Development Server
```bash
./run_dev.sh
```

### Run Tests
```bash
cargo test
```

### Generate Documentation
```bash
cargo doc --no-deps --open
```

## Next Steps

After completing these getting started guides:

1. Explore [Feature Guides](../guides/features/README.md) for implementing specific functionality
2. Review [Development Guides](../guides/development/README.md) for best practices
3. Consult [Reference Documentation](../reference/README.md) for detailed specifications
4. Check [Deployment Guides](../guides/deployment/README.md) for production deployment

## Need Help?

If you encounter issues while getting started:

1. Check the troubleshooting section in each guide
2. Review our [Common Issues](../reference/troubleshooting/common-issues.md) documentation
3. Join our [Discord Community](https://discord.gg/navius) for real-time help
4. Open an issue on our [GitHub repository](https://github.com/navius/navius)

## Contributing

We welcome contributions! If you find issues or have suggestions:

1. Read our [Contributing Guide](../contributing/contribution-guide.md)
2. Follow our [Code of Conduct](../contributing/code-of-conduct.md)
3. Submit issues or pull requests on [GitHub](https://github.com/navius/navius) 