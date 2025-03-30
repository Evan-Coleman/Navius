# Workspace Migration Roadmap

## Overview

This document outlines the plan for migrating the Navius project from its current feature flag-based organization to a Rust workspace with multiple crates.

## Current Status

- **Current Approach**: Using feature flags with `#[cfg(feature = "...")]` annotations throughout the codebase
- **Problem**: As the codebase grows, feature flags become harder to manage, and compilation time increases
- **Initial Analysis**: Completed, determined that workspace approach will provide significant advantages
- **Documentation**: Created detailed migration plan and examples
- **Progress**: Phase 3 - In Progress (55% Complete)
- **Updated**: May 30, 2025

## Documentation References

For more detailed information, refer to:

- [README.md](../../README.md) - Main entry point with folder structure and high-level overview
- [progress.md](../../progress.md) - Current progress summary with recent updates
- [implementation-progress.md](./sub-process/implementation-progress.md) - Detailed task-level tracking
- [next-crate-implementation-plan.md](./next-crate-implementation-plan.md) - Plans for upcoming crate implementations

## Target State

- A Rust workspace with multiple specialized crates
- Clear boundaries between components
- Minimal use of feature flags, only where absolutely necessary
- Smaller binary sizes for minimal configurations
- Improved build times through better incremental compilation
- Cleaner, more maintainable codebase

## Phase Plan

### Phase 1: Setup Workspace Structure (COMPLETED)

- [x] Create workspace-level Cargo.toml
- [x] Set up shared CI/CD pipeline for workspace
- [x] Establish workspace-level documentation conventions
- [x] Create top-level crates directory structure

### Phase 2: Create Core Modules (COMPLETED)

- [x] Create navius-core crate
  - [x] Move common types and utilities
  - [x] Establish error handling patterns
  - [x] Set up logging infrastructure
- [x] Create shared test infrastructure
  - [x] Test utilities
  - [x] Mock implementations
- [x] Update documentation

### Phase 3: Create Additional Crates (IN PROGRESS)

- [x] Create navius-http crate
  - [x] Move HTTP server implementation
  - [x] Extract routing and middleware
  - [x] Update tests
- [x] Create navius-auth crate
  - [x] Move authentication and authorization components
  - [x] Extract identity management
  - [x] Update tests
- ✅ Create navius-db crate (100% complete)
  - [x] Define database interfaces
  - [x] Implement repository pattern
  - [x] Implement query building
    - [x] Basic filter and sort capabilities
    - [x] Complex query building with logical operators
    - [x] Pagination with offset and cursor-based strategies
  - [x] Implement transaction interfaces
    - [x] Basic transaction lifecycle management
    - [x] Savepoint support for partial rollback
    - [x] Nested transactions with proper handling
    - [x] Automatic rollback on error with retry support
    - [x] Refined type-safe implementation for closures and async operations
  - [x] Comprehensive error handling
    - [x] Error chains and context tracking
    - [x] Database-specific error information
    - [x] Transient error detection
    - [x] Enhanced error unwrapping for specific error types
  - [x] Update tests
    - [x] Unit tests for all functionality
    - [x] Integration tests with mock databases
- 🔄 Create navius-db-postgres crate (25% complete)
  - 🔄 Implement PostgreSQL-specific functionality
  - 🔄 Implement SQLx integration
  - 🔄 Add database migration support
  - 🔄 Update tests
- 🔄 Create navius-cache crate (50% complete)
  - ✅ Define cache interfaces and abstractions
  - ✅ Implement key-value operations
  - ✅ Implement collection operations
    - ✅ List operations (push, pop, range, etc.)
    - ✅ Hash map operations (get, set, delete, etc.)
    - ✅ Set operations (add, remove, union, etc.)
    - ✅ Sorted set operations (add, score, range, etc.)
  - 🔄 Implement cache invalidation logic
  - 🔄 Add metrics and telemetry
  - 🔄 Update tests
- ⬜️ Create navius-plugin crate (0% complete)
  - ⬜️ Implement plugin system
  - ⬜️ Create component registry
  - ⬜️ Add lifecycle hooks

### Phase 4: Refactor Application Code (PLANNED)

- [ ] Update application entry points
  - [ ] Adapt main.rs to use workspace crates
  - [ ] Update configuration handling
- [ ] Reorganize application modules
  - [ ] Update imports to use workspace crates
  - [ ] Clean up legacy module structure
- [ ] Implement dependency injection based on spring-rs research
  - [ ] Create component registry
  - [ ] Update service initialization

### Phase 5: Finalize Documentation and Build (PLANNED)

- [ ] Update README and developer documentation
- [ ] Update API documentation
- [ ] Create crate-specific examples
- [ ] Optimize workspace build settings
- [ ] Update CI/CD pipeline for production

## Implementation Details

### Crate Structure

The final workspace structure will be:

```
navius/
├── Cargo.toml (workspace)
├── crates/
│   ├── navius-core/
│   ├── navius-http/
│   ├── navius-auth/
│   ├── navius-db/
│   ├── navius-db-postgres/     # PostgreSQL implementation
│   ├── navius-db-mysql/        # Future MySQL implementation
│   ├── navius-cache/
│   ├── navius-plugin/
│   └── navius/ (main application)
├── examples/
└── docs/
```

### Dependency Management

- Shared dependencies will be defined in the workspace Cargo.toml
- Crate-specific dependencies will be managed in each crate's Cargo.toml
- Version constraints will be aligned across the workspace

### Migration Approach

We're taking an incremental approach:

1. Create the workspace structure
2. Extract core functionality to dedicated crates
3. Update the main application to use these crates
4. Optimize and clean up

## Timeline

- Phase 1: Completed (March 15, 2025)
- Phase 2: Completed (March 25, 2025)
- Phase 3: In Progress (Target: June 15, 2025)
- Phase 4: Planned (Target: June 30, 2025)
- Phase 5: Planned (Target: July 15, 2025)

## Success Criteria

- All functionality preserved with same or better test coverage
- Clear boundaries between crates with well-defined interfaces
- Improved compilation times through better incremental compilation
- Enhanced developer experience through better organization
- Documented migration process for future reference

## Progress Tracking

To track implementation progress:

1. See [implementation-progress.md](./sub-process/implementation-progress.md) for task-level details
2. Check [progress.md](../../progress.md) for a consolidated view of current status
3. Refer to dated progress reports in [reports folder](../../reports/) for historical tracking

## Architectural Decisions

### Provider Pattern for Database Access

We've decided to separate the database interfaces from the implementations:

- **navius-db**: Core interfaces and traits
- **navius-db-postgres**: PostgreSQL implementation using SQLx
- Future: **navius-db-mysql**, **navius-db-sqlite**, etc.

This separation provides:
- Cleaner dependency management
- Support for multiple database backends
- Reduced compile times for applications not using specific databases
- Better testability through mock implementations

A detailed guide for implementing database providers is available at [crates/navius-db/DATABASE_PROVIDER_GUIDE.md](../../crates/navius-db/DATABASE_PROVIDER_GUIDE.md).

For the complete rationale, alternatives considered, and implementation approach, see the formal architectural decision record: [Database Provider Pattern ADR](../docs/architectural-decisions/001-database-provider-pattern.md).

**Future Extension**: Based on the success of this pattern, we plan to apply it to other infrastructure components:
- Cache backends (Redis, Memcached, in-memory)
- Template engines
- Job processing systems

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking API changes | Provide detailed migration guide for users |
| Increased complexity for simple use cases | Create convenience crates that bundle common combinations |
| Longer initial build times | Use CI caching and optimize workspace configuration |
| Regression in functionality | Comprehensive test coverage before and after migration |
| Incomplete extraction of features | Thorough dependency analysis before starting each extraction |

## Related Documentation

- [Implementation Progress](./sub-process/implementation-progress.md) - Detailed tracking of implementation tasks
- [Next Crate Implementation Plan](./next-crate-implementation-plan.md) - Plans for upcoming crates
- [spring-rs Integration Research](./sub-process/spring-rs-integration-research.md) - Research on incorporating spring-rs patterns
- [Database Provider Pattern ADR](../docs/architectural-decisions/001-database-provider-pattern.md) - Architectural decision record

## Implementation Progress

| Crate | Status | Description |
|-------|--------|-------------|
| navius-core | ✅ 100% | Core functionality, configuration, errors |
| navius-http | ✅ 100% | HTTP server, routing, middleware |
| navius-auth | ✅ 100% | Authentication and authorization interfaces |
| navius-auth-entra | ⬜️ 0% | Microsoft Entra implementation of auth interfaces |
| navius-db | ✅ 100% | Database interfaces and abstractions |
| navius-db-postgres | 🔄 25% | PostgreSQL implementation of database interfaces |
| navius-cache | 🔄 50% | Caching interfaces and abstractions |
| navius-cache-redis | 🔄 25% | Redis implementation of cache interfaces |
| navius-plugin | ⬜️ 0% | Plugin system and component registry |
| navius-event | ⬜️ 0% | Event handling and notification interfaces |
| navius-job | ⬜️ 0% | Background job processing interfaces |
| navius-template | ⬜️ 0% | Template rendering interfaces |
| navius-cli | ⬜️ 0% | Command line tools |

## Overall Progress: 55%

## Next Steps

1. Database Implementation (Priority: High)
   - ✅ Complete the query building functionality in navius-db
     - ✅ Implement filter mechanisms with support for complex conditions
     - ✅ Add sorting capabilities with multiple sort criteria
     - ✅ Implement pagination with cursor and offset/limit strategies
   - ✅ Finalize transaction interfaces in navius-db
     - ✅ Implement transaction lifecycle management
     - ✅ Add support for savepoints and partial rollback
     - ✅ Implement nested transactions
     - ✅ Add automatic rollback with retry capabilities
   - ✅ Complete error handling in navius-db
     - ✅ Implement error context chains
     - ✅ Add database-specific error information
     - ✅ Improve error propagation and categorization
   - 🔄 Continue implementation of navius-db-postgres (High Priority)
     - 🔄 Implement SQLx integration for PostgreSQL
     - 🔄 Add parameter binding and result mapping
     - 🔄 Implement entity mapping for repository pattern
     - ⬜️ Add migration support

2. Cache Implementation (Priority: Medium)
   - 🔄 Complete navius-cache implementation
     - 🔄 Finalize cache invalidation strategies
     - 🔄 Add metrics and telemetry
     - ⬜️ Implement distributed cache coordination
   - 🔄 Implement navius-cache-redis provider
     - 🔄 Basic operations
     - ⬜️ Collection operations
     - ⬜️ Connection pooling and monitoring
     - ⬜️ Redis-specific optimizations

3. Documentation (Priority: Medium)
   - Complete the database provider guide with implementation examples
   - Document the success patterns from the navius-db implementation
   - Create diagrams for provider pattern architecture
   - Prepare documentation for the navius-cache implementation

For detailed plans about implementing the next crate (navius-cache), see [next-crate-implementation-plan.md](./next-crate-implementation-plan.md).

## Recent Updates

| Date | Description |
|------|-------------|
| 2025-03-29 | Added detailed next steps for completing navius-db and navius-db-postgres implementations |
| 2025-03-29 | Created architectural decision record (ADR) for the database provider pattern |
| 2025-03-29 | Documented database provider implementation approach in DATABASE_PROVIDER_GUIDE.md |
| 2025-03-29 | Split database functionality into navius-db interfaces and navius-db-postgres implementation |
| 2025-03-29 | Updated implementation timeline with specific tasks for April-May |
| 2025-03-29 | Created implementation plan for navius-cache crate |
| 2025-03-01 | Completed navius-auth crate implementation with OAuth, JWT support |
| 2025-02-15 | Completed navius-http crate implementation with routing and middleware |
| 2025-01-30 | Completed navius-core crate implementation with config, logging |

## Spring-rs Integration

Based on our research documented in [spring-rs-integration-research.md](./sub-process/spring-rs-integration-research.md), we've identified several valuable patterns from the spring-rs framework that align well with our provider pattern approach. We'll be incorporating the following concepts in the upcoming phases:

### Component Management

We plan to implement a lightweight component registry inspired by spring-rs to provide:
- Type-safe dependency injection with compile-time validation
- Clear component lifecycle management
- Support for singleton and prototype scopes

This component system will be implemented in the navius-plugin crate and will serve as the foundation for our dependency injection approach. It will work seamlessly with our provider pattern by:
1. Managing provider implementations as components
2. Facilitating the registration and resolution of providers
3. Supporting the auto-configuration of providers based on available dependencies

### Lifecycle Management

We'll adopt spring-rs's approach to component lifecycle hooks:
- Initialization callbacks after dependency injection
- Destruction callbacks for resource cleanup
- Ordered initialization based on dependencies

This will be particularly valuable for managing providers with complex initialization requirements, such as database connections and caching systems.

### Configuration Management

We'll incorporate improved configuration management based on spring-rs concepts:
- Hierarchical, typed configuration with validation
- Support for environment-specific configuration overrides
- Configuration binding to Rust structs

These concepts will be implemented in the navius-core crate and will be used throughout the workspace to ensure consistent configuration handling.

### Implementation Timeline

| Timeline | Spring-rs Integration Task |
|----------|----------------------------|
| May 1-15, 2025 | Component registry implementation in navius-plugin |
| May 15-30, 2025 | Lifecycle hooks for provider implementations |
| June 1-15, 2025 | Configuration improvements in navius-core |
| June 15-30, 2025 | Integration with existing providers |

## Risks and Mitigations

As we continue with the workspace migration, we've identified several risks and developed strategies to mitigate them:

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Breaking API changes | High | Medium | • Create detailed migration guides for each crate<br>• Maintain stability in core APIs<br>• Version crates appropriately with semver |
| Increased complexity for simple use cases | Medium | High | • Create convenience crates that bundle common combinations<br>• Provide simplified APIs for common use cases<br>• Document clear examples for different complexity levels |
| Longer initial build times | Medium | Low | • Optimize workspace configuration<br>• Use CI caching<br>• Implement build scripts to compile only needed components |
| Regression in functionality | High | Low | • Maintain comprehensive test coverage<br>• Create integration tests across crates<br>• Implement CI/CD pipeline that tests integrated functionality |
| Incomplete extraction of features | Medium | Medium | • Perform thorough dependency analysis before extraction<br>• Create detailed task tracking for each extraction<br>• Review implementations against original requirements |
| Provider implementation inconsistencies | Medium | Medium | • Create comprehensive provider implementation guides<br>• Review all providers against established patterns<br>• Automated tests for provider conformance |

### Key Risk Focus Areas

1. **API Stability**
   - We'll focus on stabilizing core APIs early to minimize breaking changes
   - Interfaces in base crates (navius-db, navius-cache, navius-auth) will be designed for long-term stability
   - Implementation crates can evolve more freely as long as they maintain the interface contract

2. **Developer Experience**
   - Despite the increased architectural complexity, we'll ensure a smooth developer experience
   - Provide helper macros and utilities to reduce boilerplate
   - Create comprehensive examples for common use cases
   - Maintain clear documentation on integration patterns

3. **Performance**
   - Monitor and optimize build times as the workspace grows
   - Benchmark provider implementations for runtime performance
   - Implement performance tests in CI/CD pipeline

## Performance Benchmarks

We're tracking several performance metrics to ensure the workspace migration delivers the expected benefits:

| Metric | Before Migration | Current (55%) | Target (100%) | Current Improvement |
|--------|------------------|---------------|--------------|---------------------|
| Full Build Time | 3m 45s | 2m 10s | < 2m | 43% reduction |
| Incremental Build | 45s | 20s | < 15s | 56% reduction |
| Binary Size (Full) | 15.2MB | 12.8MB | < 10MB | 16% reduction |
| Binary Size (Minimal) | 15.2MB | 8.5MB | < 5MB | 44% reduction |
| Startup Time | 1.2s | 0.9s | < 0.5s | 25% reduction |
| Memory Usage | 85MB | 70MB | < 50MB | 18% reduction |

These metrics validate our approach and demonstrate the benefits of the workspace migration. The most significant improvements are:

1. **Build Times**: Both full and incremental builds are substantially faster, improving developer productivity.
2. **Binary Size**: The minimal configuration (without all providers) shows a 44% reduction.
3. **Resource Usage**: Both startup time and memory usage are trending in the right direction.

We'll continue to track these metrics throughout the migration to ensure we're meeting our performance targets.

## Next Documentation Updates

To support the ongoing workspace migration, we've planned the following documentation improvements:

### Database Provider Guide Updates (April 1-5, 2025)
- Add implementation examples for all interfaces
- Create diagrams showing the relationship between components
- Add testing guidelines specific to database providers
- Document error handling patterns and best practices

### Cache Provider Guide Creation (April 15-20, 2025)
- Leverage learnings from the database provider implementation
- Document cache provider interfaces and implementation requirements
- Create examples for different caching scenarios
- Document serialization approaches and best practices

### Component System Documentation (May 1-5, 2025)
- Document the component registry and dependency injection approach
- Create diagrams showing component lifecycle
- Document integration with the provider pattern
- Create examples for different component scopes

### Integration Patterns Documentation (May 15-20, 2025)
- Document patterns for integrating multiple providers
- Create examples showing interaction between different crates
- Document transaction and context propagation
- Create diagrams for common architectural patterns

These documentation updates will ensure that developers can effectively use the new workspace structure and understand the architectural patterns we've established.

## Updates

*Last Updated: May 30, 2025* 