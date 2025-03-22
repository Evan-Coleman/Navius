# Navius Project Structure

This document provides an overview of the Navius project structure, helping you understand where different components are located and how they work together.

## Directory Overview

```
📁 navius/
├── 📁 src/                  # Source code
│   ├── 📁 api/              # API endpoints and controllers
│   ├── 📁 app/              # Application-specific code
│   ├── 📁 core/             # Core framework functionality
│   ├── 📁 models/           # Data models and schemas
│   ├── 📁 repository/       # Data access layer
│   ├── 📁 services/         # Business logic
│   ├── 📄 lib.rs            # Library module declarations
│   └── 📄 main.rs           # Application entry point
├── 📁 config/               # Configuration files
│   ├── 📄 default.yaml      # Default configuration
│   ├── 📄 development.yaml  # Development environment configuration
│   ├── 📄 production.yaml   # Production environment configuration
│   └── 📄 local.yaml        # Local overrides (not in version control)
├── 📁 docs/                 # Documentation
├── 📁 scripts/              # Utility scripts
├── 📁 tests/                # Integration tests
├── 📁 .gitlab/              # GitLab CI/CD configuration
└── 📁 target/               # Build output (generated)
```

## Version Control System (VCS) Configuration

Navius uses a dual VCS approach with GitLab as the primary repository for business operations and GitHub as a secondary mirror for marketing and community exposure.

### GitLab (Primary - Business Operations)

The primary repository is hosted on GitLab, which serves as the central hub for all business and development operations:
- Complete CI/CD pipeline configuration in `.gitlab-ci.yml`
- Production builds and deployment workflows
- Comprehensive artifact management and versioning
- Issue tracking and formal project management
- Code review workflow using Merge Requests
- Container registry for Docker images
- Artifact storage for build outputs
- Security scanning and vulnerability reporting
- Integration with internal systems and deployment targets
- Team member access control and permissions
- Release management and versioning

**Key files:**
- `.gitlab-ci.yml` - CI/CD pipeline configuration
- `.gitlab/issue_templates/` - Issue templates
- `.gitlab/merge_request_templates/` - MR templates
- `.gitlab/ci/` - Additional CI configuration files

### GitHub (Secondary - Marketing & Community)

GitHub serves as a public-facing mirror, primarily for:
- Public visibility and marketing exposure
- Community engagement and contributions
- Developer documentation accessibility
- Open source presence and reputation building
- Backup repository
- Additional security scanning via GitHub Actions
- Public issue tracking for community feedback
- Showcase of project quality and maintenance

**Key files:**
- `.github/workflows/` - Limited GitHub Actions workflows (mainly for badges/status)
- `.github/ISSUE_TEMPLATE/` - GitHub issue templates for community
- `.github/PULL_REQUEST_TEMPLATE.md` - PR template
- `README.md` - Enhanced for public presentation with badges and marketing information

### Repository Sync Strategy

The repositories are synchronized using GitLab's mirroring feature:
- Automatic one-way sync from GitLab → GitHub after successful builds
- Production code and releases are pushed to GitHub only after validation
- Critical security fixes may be temporarily withheld from the public mirror until patches are deployed
- Documentation updates are immediately mirrored to maximize public visibility
- Public-facing assets and examples receive priority in mirroring

### Contribution Workflow

1. All internal development occurs exclusively on GitLab
2. Business-critical issues and roadmap are tracked in GitLab
3. Feature planning and sprint management happen in GitLab
4. Merge Requests are submitted and reviewed in GitLab
5. After approval, testing, and merge, changes are built via GitLab CI
6. Successful builds are automatically mirrored to GitHub
7. Community contributions via GitHub PRs are manually reviewed, imported to GitLab, and processed through the standard pipeline
8. Public discussions on GitHub are monitored and relevant information is incorporated into GitLab planning

## Key Directories in Detail

### 📁 src/api/

Contains API endpoint definitions, controllers, and request/response handlers. This is where you define your HTTP API endpoints.

**Key files:**
- `users.rs` - User API endpoints
- `health.rs` - Health check endpoints
- `metrics.rs` - Metrics endpoints

### 📁 src/app/

Contains application-specific initialization code, router setup, and state management.

**Key files:**
- `router.rs` - Application router setup
- `state.rs` - Application state management

### 📁 src/core/

Contains the core framework functionality that powers Navius. This directory is intended to remain stable and should not be modified by users.

**Sub-directories:**
- `auth/` - Authentication and authorization
- `cache/` - Caching infrastructure
- `config/` - Configuration management
- `database/` - Database connection management
- `error/` - Error handling framework
- `handlers/` - Core request handlers
- `metrics/` - Metrics collection and reporting
- `reliability/` - Reliability patterns (circuit breakers, retries)
- `router/` - Core routing infrastructure
- `utils/` - Core utility functions

### 📁 src/models/

Contains data models, schemas, and DTOs (Data Transfer Objects) used throughout the application.

**Key files:**
- `user.rs` - User model definitions
- `schemas.rs` - Shared schema definitions

### 📁 src/repository/

Contains data access code, database queries, and repository implementations. This layer is responsible for data persistence and retrieval.

**Key files:**
- `user_repository.rs` - User repository implementation
- `models.rs` - Repository-specific model definitions

### 📁 src/services/

Contains business logic and service implementations. This is where the core application logic resides.

**Key files:**
- `user_service.rs` - User service implementation
- `error.rs` - Service-specific error types

## Module Organization

Navius follows a modular architecture with clean separation of concerns:

1. **HTTP Layer** (API): Defines REST endpoints, handles HTTP requests/responses
2. **Business Logic** (Services): Implements core application functionality
3. **Data Access** (Repository): Manages data persistence and retrieval
4. **Domain Model** (Models): Defines data structures used across the application
5. **Infrastructure** (Core): Provides framework capabilities like auth, caching, etc.

## DDD-Inspired Design

Navius's design is inspired by Domain-Driven Design principles:

- **Domain Layer**: Models and core business logic
- **Application Layer**: Services that orchestrate domain operations
- **Infrastructure Layer**: Technical capabilities like persistence, caching
- **Presentation Layer**: API endpoints and controllers

## Understanding Inter-module Dependencies

The dependencies between modules follow a clean architecture approach:

```
API → Services → Repository → Database
        ↓
      Models
        ↑
       Core
```

This ensures:
- High cohesion (related functionality stays together)
- Low coupling (modules depend on abstractions, not implementations)
- Testability (components can be tested in isolation)
- Maintainability (changes in one layer have minimal impact on others)

## Extension Points

When building on Navius, you should focus on these extension points:

1. **Add new API endpoints**: Create new handler functions in `src/api/`
2. **Add new business logic**: Implement new services in `src/services/`
3. **Add new data access**: Create repositories in `src/repository/`
4. **Define new data models**: Add models in `src/models/`

## Integration with External Services

Navius provides built-in patterns for integrating with external services:

1. **API Resource Abstraction**: For consuming external APIs with resilience
2. **Repository Pattern**: For data persistence with different backends
3. **Service Interfaces**: For defining clear boundaries between components

## Testing Structure

Each component type has its own testing approach:

- **Services**: Unit tests focus on business logic
- **Repositories**: Integration tests focus on data access
- **API Endpoints**: End-to-end tests focus on HTTP interactions
- **Models**: Property-based tests focus on invariants

## Example: Adding a New Feature

To add a new "Product" feature, you would:

1. Create `src/models/product.rs` with Product model definitions
2. Create `src/repository/product_repository.rs` for data access
3. Create `src/services/product_service.rs` for business logic
4. Create `src/api/products.rs` for HTTP endpoints
5. Update `src/app/router.rs` to register the new API routes
6. Add tests for each component

This modular approach ensures that your new feature is well-structured, maintainable, and follows the established patterns of the framework. 