# Workspace Migration Roadmap

## Overview

This document outlines the plan for migrating the Navius project from its current feature flag-based organization to a Rust workspace with multiple crates.

## Current Status

- **Current Approach**: Using feature flags with `#[cfg(feature = "...")]` annotations throughout the codebase
- **Problem**: As the codebase grows, feature flags become harder to manage, and compilation time increases
- **Initial Analysis**: Completed, determined that workspace approach will provide significant advantages
- **Documentation**: Created detailed migration plan and examples
- **Progress**: Phase 4 - Integration and API Stabilization
- **Updated**: April 3, 2025

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

### Phase 1: Initial Workspace Setup (COMPLETE)

- [x] Create workspace structure
- [x] Set up build system
- [x] Define module boundaries
- [x] Create initial crates
- [x] Configure CI/CD pipeline

### Phase 2: Implement Core Functionality (COMPLETE)

- [x] Implement core framework
- [x] Implement auth framework
- [x] Implement database framework
- [x] Implement HTTP framework
- [x] Implement cache framework
- [x] Implement config framework

### Phase 3: Create Additional Crates (COMPLETE)

- [x] Create documentation generation tools
- [x] Create example applications
- [x] Create benchmarking tools

### Phase 4: Refactor Application Code (IN PROGRESS)

- [ ] Update application entry points
  - [x] Adapt main.rs to use workspace crates
  - [x] Update configuration handling
  - [x] Implement component registry and dependency injection

- [ ] Reorganize application modules
  - [x] Update imports to use workspace crates
  - [x] Clean up legacy structures

- [x] Implement dependency injection
  - [x] Create component registry
  - [x] Update service initialization

- [ ] Implement cross-crate testing infrastructure
  - [x] Design testing architecture
  - [x] Create test fixture framework
  - [x] Create mock registry
  - [x] Create test harness
  - [x] Implement error testing framework
  - [ ] Create mock implementations for interfaces (25% complete)
  - [ ] Create integration test utilities (10% complete)
  - [ ] Update existing tests to use new infrastructure

### Phase 5: Finalize Documentation and Build (PLANNED)

- [ ] Update API documentation
- [ ] Create migration guide
- [ ] Create architecture documentation
- [ ] Optimize build process
- [ ] Review error handling

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

- **Phase 1**: Completed January 15, 2025
- **Phase 2**: Completed February 20, 2025
- **Phase 3**: Completed March 29, 2025
- **Phase 4**: Integration and API Stabilization - April 1 to June 30, 2025
- **Phase 5**: Planned (Target: July 15, 2025)

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

| Crate | Status | Progress |
|-------|--------|----------|
| navius-core | Complete | 100% |
| navius-auth | Complete | 100% |
| navius-http | Complete | 100% |
| navius-cache | Complete | 100% |
| navius-config | Complete | 100% |
| navius-db | Complete | 100% |
| navius-test | In Progress | 40% |
| navius-auth-entra | Complete | 100% |
| navius-template | Not Started | 0% |
| navius-cli | Not Started | 0% |

## Overall Progress: 95%

## Next Steps

1. Continue implementing mock interfaces for the Cross-Crate Testing Infrastructure
2. Develop integration test utilities for the testing framework
3. Complete documentation and examples for the testing infrastructure
4. Begin planning for the Template Engine crate implementation
5. Start design for the CLI interface

## Timeline

- **Phase 1**: Completed January 15, 2025
- **Phase 2**: Completed February 20, 2025
- **Phase 3**: Completed March 29, 2025
- **Phase 4**: Integration and API Stabilization - April 1 to June 30, 2025
- **Phase 5**: Planned (Target: July 15, 2025)

## Recent Updates

- April 3, 2025: Completed the Error Testing Framework implementation in the `navius-test` crate
- March 29, 2025: Started implementation of the Cross-Crate Testing Infrastructure
- March 25, 2025: Completed the Microsoft Entra authentication provider implementation
- March 20, 2025: Completed the Component Registry with lifecycle hooks
- March 15, 2025: Finished Phase 3 with the completion of all core crates
- March 10, 2025: Implemented the caching framework with Redis support
- March 5, 2025: Implemented the database framework with PostgreSQL support
- March 1, 2025: Configured CI/CD pipeline and automated testing

## Spring-rs Integration

The integration of spring-rs lessons has been completed and documented in the following locations:

- [Spring-rs Analysis](../analysis/spring-rs-analysis.md)
- [Component Registry Design](../design/component-registry-design.md)
- [Dependency Injection Implementation](../implementation/dependency-injection.md)

## Updates

*Last Updated: April 3, 2025*

## Milestones

| Milestone | Target Date | Status |
|-----------|-------------|--------|
| Complete Phase 1 | January 31, 2025 | ✅ Complete |
| Complete Phase 2 | February 28, 2025 | ✅ Complete |
| Complete Phase 3 | March 15, 2025 | ✅ Complete |
| Complete Cross-Crate Testing | April 20, 2025 | 🔄 In Progress (40%) |
| Complete Phase 4 | May 15, 2025 | 🔄 In Progress |
| Complete Phase 5 | May 31, 2025 | ⏳ Not Started | 