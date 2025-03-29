# Workspace Migration Roadmap

## Overview

This document outlines the plan for migrating the Navius project from its current feature flag-based organization to a Rust workspace with multiple crates.

## Current Status

- **Current Approach**: Using feature flags with `#[cfg(feature = "...")]` annotations throughout the codebase
- **Problem**: As the codebase grows, feature flags become harder to manage, and compilation time increases
- **Initial Analysis**: Completed, determined that workspace approach will provide significant advantages
- **Documentation**: Created detailed migration plan and examples
- **Progress**: Phase 3 (75% complete) - Working on navius-db implementation
- **Updated**: March 29, 2025

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

### Phase 1.5: Architecture Research (COMPLETED)

- [x] Analyze spring-rs plugin architecture
  - [x] Document key patterns for potential adoption
  - [x] Evaluate auto-configuration approach
  - [x] Study component extraction patterns
- [x] Prototype potential improvements to Navius architecture
  - [x] Test plugin-style registration for core services
  - [x] Evaluate procedural macros for configuration
- [x] Make architectural decisions for extraction phase

**Key Findings**:
- Plugin architecture provides clear separation of concerns and explicit dependencies
- Component-based dependency injection reduces boilerplate while maintaining type safety
- Configuration management with validation improves robustness
- Selective use of procedural macros can enhance developer experience

**Recommendations**:
- Implement a lightweight plugin system for specific crates
- Create a component registry for dependency injection
- Adopt hierarchical, typed configuration with validation
- Develop targeted macros for common patterns

*See the [spring-rs integration research](./sub-process/spring-rs-integration-research.md) document for detailed analysis.*

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
- [x] Create navius-db crate (100% complete)
  - [x] Create database connection management
  - [x] Move repository pattern
  - [x] Extract query building
  - [x] Add transaction management
  - [x] Update tests
- [🔄] Create navius-cache crate (90% complete)
  - [x] Create cache connection management
  - [x] Implement cache operations
  - [x] Add cache invalidation logic
  - [x] Add basic Redis implementation
  - [🔄] Add metrics and telemetry
  - [x] Create tests

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
│   ├── navius-cache/
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
- Phase 1.5: Completed (May 30, 2025)
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

*Updated: May 30, 2025*

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
- [spring-rs Integration Research](./sub-process/spring-rs-integration-research.md) - Research on incorporating spring-rs patterns
- [Workspace Migration Plan](./workspace-migration-plan.md) - Detailed migration approach

## Implementation Status

- Overall Progress: 95%
- Current Phase: Phase 3 - Additional Module Extraction
- Next Milestone: Complete navius-cache crate implementation with metrics
- Estimated Completion: June 2025 (on track)

## Updates

| Date | Update | Updated By |
|------|--------|------------|
| March 29, 2025 | Created migration plan and supporting documentation | goblin |
| March 29, 2025 | Completed Phase 1 and Phase 2, working on Phase 3 | goblin |
| March 29, 2025 | Added spring-rs integration research phase | goblin |
| May 30, 2025 | Completed spring-rs architecture research | goblin |
| May 30, 2025 | Completed navius-db crate with transaction management | goblin |
| May 30, 2025 | Implemented navius-cache crate with Redis support | goblin | 