# Workspace Migration Roadmap - March 29, 2025

## Overview

This roadmap outlines the plan for migrating the Navius project from a feature flag-based organization to a Rust workspace with multiple crates. This change will provide better maintainability, performance improvements, and a cleaner development experience.

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

## Implementation Plan

### Phase 1: Setup Workspace Structure (COMPLETED)

- [x] Create workspace configuration in root Cargo.toml
- [x] Define shared dependencies
- [x] Create navius-core crate with essential functionality
- [x] Create navius-test-utils crate for testing infrastructure
- [x] Update build scripts and CI configuration for workspace

### Phase 1.5: Architecture Research (PLANNED)

- [ ] Analyze spring-rs plugin architecture
  - [ ] Document key patterns for potential adoption
  - [ ] Evaluate auto-configuration approach
  - [ ] Study component extraction patterns
- [ ] Prototype potential improvements to Navius architecture
  - [ ] Test plugin-style registration for core services
  - [ ] Evaluate procedural macros for configuration
- [ ] Make architectural decisions for extraction phase

*See the [spring-rs integration research](./sub-process/spring-rs-integration-research.md) document for detailed tracking.*

### Phase 2: Core Module Extraction (COMPLETED)

- [x] Extract navius-core crate
  - [x] Analyze dependencies
  - [x] Move code
  - [x] Update references
  - [x] Implement tests
- [x] Extract navius-http crate
  - [x] Analyze dependencies
  - [x] Move code
  - [x] Update references
  - [x] Implement tests
- [x] Extract navius-auth crate
  - [x] Analyze dependencies
  - [x] Move code
  - [x] Update references
  - [x] Implement tests

*See the [implementation progress](./sub-process/implementation-progress.md) document for detailed tracking.*

### Phase 3: Additional Module Extraction (IN PROGRESS)

- [ ] Extract navius-db crate (Database functionality)
  - [x] Analyze dependencies
  - [x] Move code
  - [ ] Update references
  - [ ] Implement tests
- [ ] Extract navius-metrics crate (Metrics)
  - [ ] Analyze dependencies
  - [ ] Move code
  - [ ] Update references
  - [ ] Implement tests
- [ ] Extract navius-metrics-prometheus crate (Prometheus Provider)
  - [ ] Analyze dependencies
  - [ ] Move code
  - [ ] Update references
  - [ ] Implement tests
- [ ] Extract navius-cache crate (Caching functionality)
  - [ ] Analyze dependencies
  - [ ] Move code
  - [ ] Update references
  - [ ] Implement tests

### Phase 4: Application Integration (1-2 weeks)

- [ ] Create navius-api crate (Main application)
  - [ ] Wire up modules with dependency injection
  - [ ] Create clean public API
- [ ] Simplify feature configuration
  - [ ] Update build scripts
  - [ ] Document new approach
- [ ] Update examples and documentation

### Phase 5: Cleanup and Optimization (1 week)

- [ ] Remove unused code
- [ ] Optimize build process
- [ ] Performance testing
- [ ] Binary size verification
- [ ] Final documentation update

## Success Criteria

- All functionality preserved with same or better performance
- Binary size reduced by at least 30% for minimal builds
- Build times improved by at least 20%
- Clear, well-documented APIs between crates
- All tests passing across the workspace
- Documentation updated to reflect new structure

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

- Overall Progress: 75%
- Current Phase: Phase 3 - Additional Module Extraction
- Next Milestone: Complete navius-db crate implementation
- Estimated Completion: May 2025 (adjusted for spring-rs research phase)

## Updates

| Date | Update | Updated By |
|------|--------|------------|
| March 29, 2025 | Created migration plan and supporting documentation | goblin |
| March 29, 2025 | Completed Phase 1 and Phase 2, working on Phase 3 | goblin |
| March 29, 2025 | Added spring-rs integration research phase | goblin | 