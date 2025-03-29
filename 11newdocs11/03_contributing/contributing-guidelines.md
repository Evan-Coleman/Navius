---
title: "Contributing Guidelines"
description: "Guidelines for contributing to the Navius project"
category: "Contributing"
tags: ["contributing", "guidelines", "development", "documentation", "pull-requests"]
last_updated: "March 31, 2025"
version: "1.0"
---

# Contributing Guidelines

Thank you for your interest in contributing to the Navius project! This document outlines the process and guidelines for contributing to both code and documentation.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Documentation Guidelines](#documentation-guidelines)
- [Testing Requirements](#testing-requirements)
- [Issue Reporting](#issue-reporting)
- [Community Resources](#community-resources)

## Code of Conduct

The Navius project adheres to a Code of Conduct that all contributors are expected to follow. Please read [the full text](./) to understand what actions will and will not be tolerated.

## Getting Started

1. **Fork the repository** to your GitHub account
2. **Clone your fork** to your local machine
3. **Set up the development environment** following instructions in [development-setup.md](./)
4. **Create a new branch** for your changes

```bash
git checkout -b feature/your-feature-name
```

## Development Workflow

1. **Make changes** on your feature branch
2. **Write or update tests** for your changes
3. **Ensure all tests pass** by running `cargo test`
4. **Commit your changes** with a descriptive commit message
5. **Push your branch** to your fork
6. **Create a pull request** to the main repository

## Pull Request Process

1. **Fill out the PR template** completely
2. **Link related issues** in the PR description
3. **Pass all CI checks**
4. **Request review** from appropriate maintainers
5. **Address review feedback**
6. **Wait for approval** before merging

All pull requests require at least one approval from a project maintainer before merging.

## Coding Standards

The Navius project follows strict coding standards to maintain consistency and quality:

- **Follow the Rust Style Guide** as outlined in [code-style-guide.md](./)
- **Document all public APIs** with rustdoc comments
- **Write clear commit messages** following conventional commits
- **Maintain backward compatibility** where possible

## Documentation Guidelines

Documentation is a critical part of the Navius project:

- **Follow the Markdown Style Guide** in [markdown-style-guide.md](./)
- **Update documentation** when you change functionality
- **Include code examples** for API documentation
- **Use correct frontmatter** for all documentation files
- **Use proper code blocks** with language identifiers at the beginning only (not at the end)
- **Check links** to ensure they point to correct targets

### Documentation Code Block Format

When adding code examples in documentation, always use the correct markdown format:

✅ CORRECT:
````markdown
```rust
fn example() {
    println!("This is correct!");
}
```
````

❌ INCORRECT:
````markdown
```rust
fn example() {
    println!("This is incorrect!");
}
```rust
````

## Testing Requirements

All code contributions must include appropriate tests:

- **Write unit tests** for new functionality
- **Write integration tests** for API endpoints
- **Maintain or improve code coverage**
- **Test edge cases and error conditions**

Run tests locally before submitting:

```bash
cargo test
```

## Issue Reporting

When reporting issues:

1. **Use the issue template**
2. **Include reproduction steps**
3. **List relevant environment information**
4. **Attach logs** or screenshots if applicable
5. **Suggest a fix** if possible

## Community Resources

- **Project Chat:** [Discord Server](https://discord.gg/navius)
- **Mailing List:** contributors@navius.example.com
- **Office Hours:** Every Wednesday at 14:00 UTC

## Related Documents

- [Code Style Guide](./)
- [Markdown Style Guide](./)
- [Development Setup](./)
- [Architecture Overview](./) 
