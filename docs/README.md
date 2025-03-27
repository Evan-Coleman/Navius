---
title: Navius Documentation
description: Comprehensive documentation for the Navius framework
category: index
tags:
  - documentation
  - index
  - overview
related:
  - getting-started/README.md
  - guides/README.md
  - reference/README.md
  - architecture/README.md
  - roadmaps/README.md
  - contributing/README.md
last_updated: March 26, 2025
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

- Join our [Community](contributing/code-of-conduct.md)
- Report issues on [GitLab](https://gitlab.com/ecoleman2/navius/) (primary repository)
- Check our mirror on [GitHub](https://github.com/Evan-Coleman/Navius)

## License

Navius is licensed under the [Apache License 2.0](LICENSE.md) ([view on GitLab](https://gitlab.com/ecoleman2/navius/-/blob/master/LICENSE))

## 📚 Documentation Sections

### 🚀 Getting Started
Quick start guides to get up and running with Navius:

- [Installation](getting-started/installation.md) - How to install Navius
- [Development Setup](getting-started/development-setup.md) - Setting up your development environment
- [First Steps](getting-started/first-steps.md) - Getting started with Navius

### 🏛️ Architecture
Explore the architectural foundations of Navius:

- [Overview](architecture/README.md) - Introduction to Navius architecture
- [Project Structure](architecture/project-structure.md) - Overview of the project organization
- [Module Dependencies](architecture/module-dependencies.md) - How modules interact with each other
- [Spring Boot Migration](architecture/spring-boot-migration.md) - Guide for migrating from Spring Boot

### 🛠️ Guides
Practical guides for using Navius:

- [Overview](guides/README.md) - Introduction to Navius guides

- [Development](guides/development/README.md) - Development workflow and practices
  - [Development Workflow](guides/development/development-workflow.md) - Day-to-day development process
  - [Testing Guide](guides/development/testing-guide.md) - How to test Navius applications
  - [Debugging Guide](guides/development/debugging-guide.md) - Debugging your applications
  - [IDE Setup](guides/development/ide-setup.md) - Setting up your development environment
  - [Git Workflow](guides/development/git-workflow.md) - Version control practices

- [Features](guides/features/README.md) - Implementing specific features
  - [Authentication](guides/features/authentication.md) - Implementing authentication
  - [API Integration](guides/features/api-integration.md) - Integrating with external APIs
  - [PostgreSQL Integration](guides/features/postgresql-integration.md) - Working with PostgreSQL in features
  - [Redis Caching](guides/features/caching.md) - Implementing basic caching
  - [Caching Strategies](guides/caching-strategies.md) - Advanced caching with two-tier cache
  - [Server Customization CLI](guides/features/server-customization-cli.md) - Using the feature selection CLI
  - [WebSocket Support](guides/features/websocket-support.md) - Real-time communication

- [Deployment](guides/deployment/README.md) - Deploying Navius applications
  - [Production Deployment](guides/deployment/production-deployment.md) - Deploying to production
  - [Docker Deployment](guides/deployment/docker-deployment.md) - Working with Docker
  - [AWS Deployment](guides/deployment/aws-deployment.md) - Deploying to AWS
  - [Kubernetes Deployment](guides/deployment/kubernetes-deployment.md) - Deploying to Kubernetes

- [PostgreSQL Integration](guides/postgresql_integration.md) - Comprehensive PostgreSQL integration
- [Deployment Guide](guides/deployment.md) - General deployment information

### 📖 Reference
Technical reference documentation:

- [Overview](reference/README.md) - Introduction to reference documentation

- [API](reference/api/README.md) - API documentation
  - [API Resources](reference/api/api-resource.md) - Core API resources
  - [Authentication API](reference/api/authentication-api.md) - Authentication endpoints
  - [Database API](reference/api/database-api.md) - Database interaction APIs

- [Architecture](reference/architecture/README.md) - Architecture patterns and principles
  - [Principles](reference/architecture/principles.md) - Architectural principles
  - [Project Structure](reference/architecture/project-structure-recommendations.md) - Recommended structure
  - [Directory Organization](reference/architecture/directory-organization.md) - How directories are organized
  - [Component Architecture](reference/architecture/component-architecture.md) - Component design

- [Configuration](reference/configuration/README.md) - Configuration options and settings
  - [Environment Variables](reference/configuration/environment-variables.md) - Environment configuration
  - [Application Config](reference/configuration/application-config.md) - Application settings
  - [Cache Config](reference/configuration/cache-config.md) - Cache system configuration
  - [Feature Config](reference/configuration/feature-config.md) - Server customization configuration
  - [Logging Config](reference/configuration/logging-config.md) - Logging configuration
  - [Security Config](reference/configuration/security-config.md) - Security settings

- [Patterns](reference/patterns/README.md) - Common design patterns
  - [API Resource Pattern](reference/patterns/api-resource-pattern.md) - API design patterns
  - [Import Patterns](reference/patterns/import-patterns.md) - Module import patterns
  - [Caching Patterns](reference/patterns/caching-patterns.md) - Effective caching strategies
  - [Error Handling](reference/patterns/error-handling-patterns.md) - Error handling approaches
  - [Testing Patterns](reference/patterns/testing-patterns.md) - Testing best practices
  - [Repository Pattern](reference/patterns/repository-pattern.md) - Entity repository pattern
  - [Logging Service Pattern](reference/patterns/logging-service-pattern.md) - Generic logging service implementations

- [Standards](reference/standards/README.md) - Code and documentation standards
  - [Naming Conventions](reference/standards/naming-conventions.md) - Naming guidelines
  - [Code Style](reference/standards/code-style.md) - Code formatting standards
  - [Generated Code](reference/standards/generated-code-standards.md) - Generated code guidelines
  - [Security Standards](reference/standards/security-standards.md) - Security best practices
  - [Documentation Standards](reference/standards/documentation-standards.md) - Documentation guidelines

### 🗺️ Roadmaps
Project roadmaps and future plans:

- [Overview](roadmaps/README.md) - Introduction to project roadmaps
- [Template for Updating](roadmaps/template-for-updating.md) - How to update roadmaps
- [Dependency Injection](roadmaps/01-dependency-injection.md) - DI implementation roadmap
- [Database Integration](roadmaps/02-database-integration.md) - Database features roadmap
- [Testing Framework](roadmaps/03-testing-framework.md) - Testing capabilities roadmap
- [AWS Integration](roadmaps/04-aws-integration.md) - AWS services integration
- [Data Validation](roadmaps/05-data-validation.md) - Input validation roadmap
- [Resilience Patterns](roadmaps/06-resilience-patterns.md) - Fault tolerance features
- [Enhanced Caching](roadmaps/07-enhanced-caching.md) - Advanced caching strategies
- [API Versioning](roadmaps/08-api-versioning.md) - API evolution approach
- [Metrics & Observability](roadmaps/09-metrics-observability.md) - Monitoring capabilities
- [Declarative Features](roadmaps/09-declarative-features.md) - Configuration-driven features
- [Developer Experience](roadmaps/10-developer-experience.md) - Improving DX
- [Security Features](roadmaps/11-security-features.md) - Security enhancements
- [Document Overhaul](roadmaps/12_document_overhaul.md) - Documentation improvements
- [Generic Service Implementations](roadmaps/25-generic-service-implementations.md) - Generic service interfaces with pluggable providers

### 🤝 Contributing
Guidelines for contributors:

- [Overview](contributing/README.md) - Introduction to contributing
- [Contributing Guide](contributing/contribution-guide.md) - How to contribute
- [Code of Conduct](contributing/code-of-conduct.md) - Community guidelines
- [Development Process](contributing/development-process.md) - Development workflow
- [Testing Guidelines](contributing/testing-guidelines.md) - Writing tests
- [Onboarding](contributing/onboarding.md) - Getting started as a contributor
- [IDE Setup](contributing/ide-setup.md) - Setting up your development environment
- [Testing Prompt](contributing/testing-prompt.md) - Testing guidelines
- [Test Implementation Template](contributing/test-implementation-template.md) - Templates for tests

### 📚 Examples
Practical code examples:

- [Spring Boot Comparison](examples/20_spring-boot-comparison.md) - Comparing with Spring Boot
- [Two-Tier Cache Implementation](examples/two-tier-cache-example.md) - Implementing two-tier caching
- [Server Customization System](examples/server-customization-example.md) - Using the feature system
- [Repository Pattern Example](examples/repository-pattern-example.md) - Implementing the generic repository pattern
- [Logging Service Example](examples/logging-service-example.md) - Using the generic logging service

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