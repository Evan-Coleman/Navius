---
title: "Contribution Guide"
description: "Step-by-step guide for making contributions to the Navius project"
category: "Contributing"
tags: ["contributing", "development", "workflow", "guide", "pull request"]
last_updated: "April 5, 2025"
version: "1.0"
---

# Contributing Guide

## Overview

Thank you for your interest in contributing to the Navius project! This guide will walk you through the process of making contributions, from setting up your development environment to submitting your changes for review.

## Prerequisites

Before you begin, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Git](https://git-scm.com/downloads)
- A code editor (we recommend [VS Code](https://code.visualstudio.com/) with the Rust extension)
- [Docker](https://docs.docker.com/get-docker/) (for running integration tests)
- [PostgreSQL](https://www.postgresql.org/download/) (for local development)

## Getting Started

### 1. Fork the Repository

1. Visit the [Navius repository](https://github.com/example/navius)
2. Click the "Fork" button in the top-right corner
3. Clone your fork to your local machine:
   ```bash
   git clone https://github.com/YOUR_USERNAME/navius.git
   cd navius
   ```

### 2. Set Up the Development Environment

1. Add the original repository as an upstream remote:
   ```bash
   git remote add upstream https://github.com/example/navius.git
   ```

2. Install project dependencies:
   ```bash
   cargo build
   ```

3. Set up the database:
   ```bash
   ./scripts/setup_db.sh
   ```

4. Run the test suite to make sure everything is working:
   ```bash
   cargo test
   ```

## Making Changes

### 1. Create a Feature Branch

Always create a new branch for your changes:

```bash
git checkout -b feature/your-feature-name
```

Use a descriptive branch name that reflects the changes you're making.

### 2. Development Workflow

1. Make your changes in small, focused commits
2. Follow our [coding standards](./markdown-style-guide.md)
3. Include tests for your changes
4. Update documentation as needed

#### Code Style

- Run `cargo fmt` before committing to ensure your code follows our style guidelines
- Use `cargo clippy` to catch common mistakes and improve your code

#### Testing

- Write unit tests for all new functions
- Create integration tests for API endpoints
- Run tests with `cargo test` before submitting your changes

### 3. Keep Your Branch Updated

Regularly sync your branch with the upstream repository:

```bash
git fetch upstream
git rebase upstream/main
```

Resolve any conflicts that arise during the rebase.

## Submitting Your Contribution

### 1. Prepare Your Changes

Before submitting, make sure:

- All tests pass: `cargo test`
- Code passes linting: `cargo clippy`
- Your code is formatted: `cargo fmt`
- You've added/updated documentation

### 2. Create a Pull Request

1. Push your changes to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. Go to the [Navius repository](https://github.com/example/navius)

3. Click "Pull Requests" and then "New Pull Request"

4. Select your fork and the feature branch containing your changes

5. Provide a clear title and description for your pull request:
   - What changes does it introduce?
   - Why are these changes necessary?
   - How do these changes address the issue?
   - Any specific areas you'd like reviewers to focus on?

6. Link any related issues by including "Fixes #issue-number" or "Relates to #issue-number" in the description

### 3. Code Review Process

1. Wait for the CI/CD pipeline to complete
2. Address any feedback from reviewers
3. Make requested changes in new commits
4. Push the changes to the same branch
5. Mark resolved conversations as resolved

See our [Code Review Process](./code-review-process.md) for more details.

## Types of Contributions

### Bug Fixes

1. Check if the bug is already reported in the [issues](https://github.com/example/navius/issues)
2. If not, create a new issue describing the bug
3. Follow the steps above to submit a fix

### Features

1. For significant features, open an issue to discuss the proposal first
2. Once consensus is reached, implement the feature
3. Include comprehensive tests and documentation

### Documentation

1. For typos and minor corrections, you can edit directly on GitHub
2. For significant changes, follow the standard contribution process
3. Follow our [Documentation Standards](./documentation-standards.md)

## Local Development Tips

### Running the Application

```bash
cargo run
```

Visit `http://localhost:8080` to see the application running.

### Debugging

- Use `println!()` or the `log` crate for debugging
- For more advanced debugging, VS Code's Rust debugger works well

### Common Issues

- **Database connection errors**: Ensure PostgreSQL is running and credentials are correct
- **Compilation errors**: Run `cargo clean` followed by `cargo build`
- **Test failures**: Check for environment-specific issues like file permissions

## Contributor Expectations

- Follow our [Code of Conduct](./code-of-conduct.md)
- Be respectful and constructive in discussions
- Respond to feedback in a timely manner
- Help review other contributions when possible

## Recognition

Contributors are recognized in several ways:

- Added to the contributors list in the README
- Mentioned in release notes for significant contributions
- Potential for direct commit access after consistent quality contributions

## Related Resources

- [Code of Conduct](./code-of-conduct.md)
- [Coding Standards](./markdown-style-guide.md)
- [Documentation Standards](./documentation-standards.md)
- [Code Review Process](./code-review-process.md)
- [Project Roadmap](../98_roadmaps/project-roadmap.md)

Thank you for contributing to Navius! Your efforts help make this project better for everyone.
