---
title: Navius Documentation
description: Comprehensive documentation for the Navius framework
category: index
tags:
  - documentation
  - index
  - overview
related:
  - 01_getting_started/README.md
  - 04_guides/README.md
  - 05_reference/README.md
  - 03_contributing/README.md
  - 02_examples/README.md
last_updated: March 27, 2025
version: 1.0
---

<!-- Documentation Build Instructions -->
<!--
# Building Documentation

This documentation is built using [mdBook](https://rust-lang.github.io/mdBook/).

## Local Development

1. Install mdBook and the required plugins:
   ```bash
   cargo install mdbook --version 0.4.35
   cargo install mdbook-mermaid --version 0.12.6
   ```

2. Build the documentation:
   ```bash
   cd docs
   mdbook build
   ```

3. Serve the documentation locally:
   ```bash
   mdbook serve --open
   ```

## Deployment

The documentation is automatically deployed to GitLab Pages when changes are pushed to the main branch.
The CI/CD pipeline will build the documentation and deploy it to https://ecoleman2.gitlab.io/navius/.

-->

# Navius Documentation

Welcome to the Navius documentation. This guide will help you understand and work with the Navius framework effectively.

## Overview

Navius is a modern, secure, and performant web framework built with Rust and Axum. This documentation covers everything from getting started to advanced topics and best practices.

## Key Features

- **Security First**: Built-in security features and best practices
- **High Performance**: Optimized for speed and efficiency
- **Developer Experience**: Intuitive APIs and comprehensive documentation
- **Cloud Native**: Designed for modern cloud environments
- **Extensible**: Modular architecture for easy customization
- **Two-Tier Caching**: Fast in-memory cache with persistent Redis fallback
- **Server Customization**: Feature selection for optimized deployment
- **Generic Repositories**: Type-safe repository pattern for domain entities
- **Generic Logging Service**: Pluggable logging providers with structured logging

## Getting Help

- Join our [Community](03_contributing/code-of-conduct.md)
- Report issues on [GitLab](https://gitlab.com/ecoleman2/navius/) (primary repository)
- Check our mirror on [GitHub](https://github.com/Evan-Coleman/Navius)

## License

Navius is licensed under the [Apache License 2.0](99_misc/LICENSE.md) ([view on GitLab](https://gitlab.com/ecoleman2/navius/-/blob/master/LICENSE))

## 📚 Documentation Sections

### 🚀 Getting Started
Quick start guides to get up and running with Navius:

- [Installation](01_getting_started/installation.md) - How to install Navius
- [Development Setup](01_getting_started/development-setup.md) - Setting up your development environment
- [First Steps](01_getting_started/first-steps.md) - Getting started with Navius

### 📚 Examples
Practical code examples:

- [Overview](02_examples/README.md) - Introduction to examples
- [Spring Boot Comparison](02_examples/spring-boot-comparison.md) - Comparing with Spring Boot
- [Two-Tier Cache Implementation](02_examples/two-tier-cache-example.md) - Implementing two-tier caching
- [Server Customization System](02_examples/server-customization-example.md) - Using the feature system
- [Repository Pattern Example](02_examples/repository-pattern-example.md) - Implementing the generic repository pattern
- [Logging Service Example](02_examples/logging-service-example.md) - Using the generic logging service
- [Database Service Example](02_examples/database-service-example.md) - Working with the generic database service
- [Health Service Example](02_examples/health-service-example.md) - Creating custom health indicators
- [Cache Provider Example](02_examples/cache-provider-example.md) - Using the generic cache providers

### 🤝 Contributing
Guidelines for contributors:

- [Overview](03_contributing/README.md) - Introduction to contributing
- [Contributing Guide](03_contributing/contribution-guide.md) - How to contribute
- [Code of Conduct](03_contributing/code-of-conduct.md) - Community guidelines
- [Development Process](03_contributing/development-process.md) - Development workflow
- [Testing Guidelines](03_contributing/testing-guidelines.md) - Writing tests
- [Onboarding](03_contributing/onboarding.md) - Getting started as a contributor
- [IDE Setup](03_contributing/ide-setup.md) - Setting up your development environment
- [Testing Prompt](03_contributing/testing-prompt.md) - Testing guidelines
- [Test Implementation Template](03_contributing/test-implementation-template.md) - Templates for tests

### 🛠️ Guides
Practical guides for using Navius:

- [Overview](04_guides/README.md) - Introduction to Navius guides

- [Development](04_guides/development/README.md) - Development workflow and practices
  - [Development Workflow](04_guides/development/development-workflow.md) - Day-to-day development process
  - [Testing Guide](04_guides/development/testing-guide.md) - How to test Navius applications
  - [Debugging Guide](04_guides/development/debugging-guide.md) - Debugging your applications
  - [IDE Setup](04_guides/development/ide-setup.md) - Setting up your development environment
  - [Git Workflow](04_guides/development/git-workflow.md) - Version control practices

- [Features](04_guides/features/README.md) - Implementing specific features
  - [Authentication](04_guides/features/authentication.md) - Implementing authentication
  - [API Integration](04_guides/features/api-integration.md) - Integrating with external APIs
  - [PostgreSQL Integration](04_guides/features/postgresql-integration.md) - Working with PostgreSQL in features
  - [Redis Caching](04_guides/features/caching.md) - Implementing basic caching
  - [Server Customization CLI](04_guides/features/server-customization-cli.md) - Using the feature selection CLI
  - [WebSocket Support](04_guides/features/websocket-support.md) - Real-time communication

- [Deployment](04_guides/deployment/README.md) - Deploying Navius applications
  - [Production Deployment](04_guides/deployment/production-deployment.md) - Deploying to production
  - [Docker Deployment](04_guides/deployment/docker-deployment.md) - Working with Docker
  - [AWS Deployment](04_guides/deployment/aws-deployment.md) - Deploying to AWS
  - [Kubernetes Deployment](04_guides/deployment/kubernetes-deployment.md) - Deploying to Kubernetes

- [Caching Strategies](04_guides/caching-strategies.md) - Advanced caching with two-tier cache
- [PostgreSQL Integration](04_guides/postgresql-integration.md) - Comprehensive PostgreSQL integration
- [Application Structure](04_guides/application-structure.md) - App structure guide
- [Configuration](04_guides/configuration.md) - Configuration guide
- [Dependency Injection](04_guides/dependency-injection.md) - DI guide
- [Error Handling](04_guides/error-handling.md) - Error handling guide
- [Feature Selection](04_guides/feature-selection.md) - Feature selection guide
- [Service Registration](04_guides/service-registration.md) - Service registration guide
- [Testing](04_guides/testing.md) - Testing guide

### 📖 Reference
Technical reference documentation:

- [Overview](05_reference/README.md) - Introduction to reference documentation

- [API](05_reference/api/README.md) - API documentation
  - [API Resources](05_reference/api/api-resource.md) - Core API resources
  - [Authentication API](05_reference/api/authentication-api.md) - Authentication endpoints
  - [Database API](05_reference/api/database-api.md) - Database interaction APIs

- [Architecture](05_reference/architecture/README.md) - Architecture patterns and principles
  - [Principles](05_reference/architecture/principles.md) - Architectural principles
  - [Project Structure](05_reference/architecture/project-structure.md) - Project structure overview
  - [Project Structure Recommendations](05_reference/architecture/project-structure-recommendations.md) - Recommended structure
  - [Directory Organization](05_reference/architecture/directory-organization.md) - How directories are organized
  - [Component Architecture](05_reference/architecture/component-architecture.md) - Component design
  - [Design Principles](05_reference/architecture/design-principles.md) - Design principles
  - [Extension Points](05_reference/architecture/extension-points.md) - Extension points
  - [Module Dependencies](05_reference/architecture/module-dependencies.md) - Module dependencies
  - [Provider Architecture](05_reference/architecture/provider-architecture.md) - Provider architecture
  - [Service Architecture](05_reference/architecture/service-architecture.md) - Service architecture
  - [Spring Boot Migration](05_reference/architecture/spring-boot-migration.md) - Spring Boot migration

- [Auth](05_reference/auth/README.md) - Authentication documentation
  - [Error Handling](05_reference/auth/ERROR_HANDLING.md) - Auth error handling
  - [Auth Circuit Breaker](05_reference/auth/auth_circuit_breaker_operation.md) - Auth circuit breaker
  - [Auth Metrics](05_reference/auth/auth_metrics.md) - Auth metrics
  - [Auth Provider Implementation](05_reference/auth/auth_provider_implementation.md) - Auth provider implementation

- [Configuration](05_reference/configuration/README.md) - Configuration options and settings
  - [Environment Variables](05_reference/configuration/environment-variables.md) - Environment configuration
  - [Application Config](05_reference/configuration/application-config.md) - Application settings
  - [Cache Config](05_reference/configuration/cache-config.md) - Cache system configuration
  - [Feature Config](05_reference/configuration/feature-config.md) - Server customization configuration
  - [Logging Config](05_reference/configuration/logging-config.md) - Logging configuration
  - [Security Config](05_reference/configuration/security-config.md) - Security settings

- [Patterns](05_reference/patterns/README.md) - Common design patterns
  - [API Resource Pattern](05_reference/patterns/api-resource-pattern.md) - API design patterns
  - [Import Patterns](05_reference/patterns/import-patterns.md) - Module import patterns
  - [Caching Patterns](05_reference/patterns/caching-patterns.md) - Effective caching strategies
  - [Error Handling](05_reference/patterns/error-handling-patterns.md) - Error handling approaches
  - [Testing Patterns](05_reference/patterns/testing-patterns.md) - Testing best practices
  - [Repository Pattern](05_reference/patterns/repository-pattern.md) - Entity repository pattern
  - [Logging Service Pattern](05_reference/patterns/logging-service-pattern.md) - Generic logging service implementations

- [Standards](05_reference/standards/README.md) - Code and documentation standards
  - [Naming Conventions](05_reference/standards/naming-conventions.md) - Naming guidelines
  - [Code Style](05_reference/standards/code-style.md) - Code formatting standards
  - [Generated Code](05_reference/standards/generated-code-standards.md) - Generated code guidelines
  - [Security Standards](05_reference/standards/security-standards.md) - Security best practices
  - [Documentation Standards](05_reference/standards/documentation-standards.md) - Documentation guidelines
  - [Configuration Standards](05_reference/standards/configuration-standards.md) - Configuration standards
  - [Error Handling Standards](05_reference/standards/error-handling-standards.md) - Error handling standards
  - [Error Handling](05_reference/standards/error-handling.md) - Error handling guide

- [Generated](05_reference/generated/README.md) - Generated reference documentation
  - [API Index](05_reference/generated/api/index.md) - API index
  - [Configuration Index](05_reference/generated/config/index.md) - Configuration index
  - [Development Configuration](05_reference/generated/config/development.md) - Development configuration
  - [Production Configuration](05_reference/generated/config/production.md) - Production configuration
  - [Testing Configuration](05_reference/generated/config/testing.md) - Testing configuration
  - [Features Index](05_reference/generated/features/index.md) - Features index

### 🗺️ Roadmaps
Project roadmaps and future plans:

- [Overview](98_roadmaps/README.md) - Introduction to project roadmaps
- [Template for Updating](98_roadmaps/template-for-updating.md) - How to update roadmaps
- [Dependency Injection](98_roadmaps/01-dependency-injection.md) - DI implementation roadmap
- [Database Integration](98_roadmaps/02-database-integration.md) - Database features roadmap
- [Testing Framework](98_roadmaps/03-testing-framework.md) - Testing capabilities roadmap

### 🧩 Miscellaneous
Additional resources and documentation:

- [Feature System](99_misc/feature-system.md) - Overview of the feature system
- [Testing Guidance](99_misc/testing-guidance.md) - Additional testing guidance
- [Document Template](99_misc/document-template.md) - Documentation template
- [Migration Plan](99_misc/migration-plan.md) - Documentation migration plan

## 🔍 Documentation Search

Use the search functionality in the top bar to search through all documentation, or use your browser's search (Ctrl+F / Cmd+F) to search within the current page.

## 📝 Documentation Standards

All documentation follows these standards:

1. **Frontmatter**: Each document includes metadata in the YAML frontmatter
2. **Structure**: Clear headings and subheadings with logical progression
3. **Code Examples**: Practical examples with syntax highlighting
4. **Cross-referencing**: Links to related documentation
5. **Up-to-date**: Regular reviews and updates to ensure accuracy

## 🆘 Need Help?

If you can't find what you're looking for, please:

1. Check the [GitLab Issues](https://gitlab.com/ecoleman2/navius/-/issues) for known documentation issues
2. Open a new documentation issue if you find something missing or incorrect 