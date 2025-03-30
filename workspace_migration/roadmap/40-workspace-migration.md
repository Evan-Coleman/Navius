# Workspace Migration Roadmap

## Overview

This document outlines the plan for migrating the Navius project from its current feature flag-based organization to a Rust workspace with multiple crates.

## Current Status

- **Current Approach**: Using feature flags with `#[cfg(feature = "...")]` annotations throughout the codebase
- **Problem**: As the codebase grows, feature flags become harder to manage, and compilation time increases
- **Initial Analysis**: Completed, determined that workspace approach will provide significant advantages
- **Documentation**: Created detailed migration plan and examples
- **Progress**: Phase 3 (40% complete) - Working on navius-db and navius-db-postgres implementation
- **Updated**: March 29, 2025

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
- 🔄 Create navius-db crate (75% complete)
  - [x] Define database interfaces
  - [x] Implement repository pattern
  - 🔄 Implement query building
  - 🔄 Implement transaction interfaces
  - 🔄 Update tests
- 🔄 Create navius-db-postgres crate (25% complete)
  - 🔄 Implement PostgreSQL-specific functionality
  - 🔄 Implement SQLx integration
  - ⬜️ Add database migration support
  - ⬜️ Update tests
- ⬜️ Create navius-cache crate (0% complete)
  - ⬜️ Create cache connection management
  - ⬜️ Implement cache operations
  - ⬜️ Add cache invalidation logic
  - ⬜️ Add Redis implementation
  - ⬜️ Add metrics and telemetry
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
| navius-auth | ✅ 100% | Authentication and authorization |
| navius-db | 🔄 75% | Database interfaces and abstractions |
| navius-db-postgres | 🔄 25% | PostgreSQL implementation of database interfaces |
| navius-cache | ⬜️ 0% | Caching functionality with Redis support |
| navius-plugin | ⬜️ 0% | Plugin system and component registry |
| navius-event | ⬜️ 0% | Event handling and notifications |
| navius-job | ⬜️ 0% | Background job processing |
| navius-template | ⬜️ 0% | Template rendering and email |
| navius-cli | ⬜️ 0% | Command line tools |

## Overall Progress: 40%

## Next Steps

1. Complete the query building functionality in navius-db
2. Implement transaction interfaces in navius-db
3. Complete the SQLx integration in navius-db-postgres
4. Add database migration support to navius-db-postgres
5. Begin preparation for navius-cache implementation

For detailed plans about implementing the next crate (navius-cache), see [next-crate-implementation-plan.md](./next-crate-implementation-plan.md).

## Recent Updates

| Date | Description |
|------|-------------|
| 2025-03-29 | Created architectural decision record (ADR) for the database provider pattern |
| 2025-03-29 | Documented database provider implementation approach in DATABASE_PROVIDER_GUIDE.md |
| 2025-03-29 | Split database functionality into navius-db interfaces and navius-db-postgres implementation |
| 2025-03-29 | Working on navius-db and navius-db-postgres crates |
| 2025-03-01 | Completed navius-auth crate implementation with OAuth, JWT support |
| 2025-02-15 | Completed navius-http crate implementation with routing and middleware |
| 2025-01-30 | Completed navius-core crate implementation with config, logging |

## Updates

| Date | Update | Updated By |
|------|--------|------------|
| March 29, 2025 | Created architectural decision record for database provider pattern | goblin |
| March 29, 2025 | Updated progress report with provider pattern implementation details | goblin |
| March 29, 2025 | Created detailed implementation progress tracking document | goblin |
| March 29, 2025 | Created migration plan and supporting documentation | goblin |
| March 29, 2025 | Completed Phase 1 and Phase 2, working on Phase 3 | goblin |
| March 29, 2025 | Implemented provider pattern for database access with navius-db-postgres | goblin |
| March 29, 2025 | Created database provider implementation guide | goblin |
| March 29, 2025 | Working on database interfaces in navius-db | goblin |
| March 29, 2025 | Working on PostgreSQL implementation in navius-db-postgres | goblin |
| March 15, 2025 | Initial migration plan developed, repository structure established | goblin | 