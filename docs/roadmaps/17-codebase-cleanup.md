---
title: "Codebase Cleanup and Error Resolution Roadmap"
description: "A roadmap for fixing errors and updating documentation after Pet API integration"
category: roadmap
tags:
  - cleanup
  - testing
  - documentation
  - error-resolution
last_updated: March 24, 2025
version: 1.1
---

# 17: Codebase Cleanup

**Progress: 70%**

This roadmap outlines our plan to clean up and organize the Navius codebase, primarily focusing on standardizing error handling, removing duplication, and establishing clear architectural boundaries.

## CRITICAL: Phase 0: Fix Error Count (Urgent Priority ⚠️)
- [ ] Reduce error count from 84 errors and 11 warnings to 0 to enable successful `cargo run`
- [ ] Fix critical Executor trait implementation errors (20+ errors)
- [x] Resolve type mismatches between services and repositories (15+ errors)
- [ ] Address SQLx offline mode errors (4 errors)
- [x] Fix Arc wrapper consistency issues (10+ errors)
- [x] Solve metrics handler lifetime issues (6+ errors)
- [x] Fix missing trait implementations (15+ errors)
- [ ] Resolve database connection and pool type issues (10+ errors)

## Phase 1: Analysis and Documentation (Completed ✅)
- [x] Review current API implementations and identify inconsistencies
- [x] Document existing error handling approaches
- [x] Create schema diagrams for the primary data models
- [x] Identify areas with duplicate code or conflicting implementations

## Phase 2: Implementation (In Progress 🔄)

### 2.1 Error Handling (Completed ✅)
- [x] Fix the AppError to include missing variants
- [x] Standardize error handling across all endpoints
- [x] Implement comprehensive logging for errors
- [x] Ensure proper error conversion between layers
- [x] Resolve duplicate error conversions

### 2.2 Code Organization (In Progress 🔄)
- [x] Fix ambiguous imports
- [x] Fix metrics handler
- [x] Clean up duplicate type definitions (PgPool, PgTransaction, MockDatabaseConnection)
- [x] Fix visibility issues in trait implementations
- [x] Resolve type mismatches between UUID and i32 ID fields
- [x] Implement missing cache methods in ResourceCache and CacheRegistry
- [x] Clean up metrics handling across the application
- [x] Harmonize API response formats
- [ ] Remove all legacy and deprecated code (as this is a greenfield project)

### 2.3 API Consistency (In Progress 🔄)
- [ ] Standardize route naming conventions
- [ ] Normalize query parameter handling
- [x] Fix type mismatches between service and repository layers
- [ ] Implement consistent API versioning strategy

## Current Focus
- HIGHEST PRIORITY: Fixing remaining compilation errors
- Fixed AppError to ServiceError conversion issues
- Fixed module structure and import path issues
- Addressing type mismatches in core functions and database handling

## Next Steps
- Resolve remaining database connection and executor issues
- Fix remaining type errors in repository implementations
- Complete API consistency fixes in Phase 2.3
- Begin implementing API versioning strategy

## Timeline
- Start Date: March 22, 2025
- Target Completion: July 15, 2025
- Last Updated: March 24, 2025
- URGENT: Error count zero goal: July 2, 2025

## Dependencies
- Core team availability for code reviews
- Backward compatibility requirements
- Integration with new roadmap features

## Overview
This roadmap outlines the strategy to address approximately 100 errors (up from 60 test errors and 32 build errors) following the implementation of the Pet API Database Integration. We will also update outdated documentation and ensure consistency across the codebase.

## Current Status
- Error count has been significantly reduced from ~70 errors to only 6 errors, though still blocking successful `cargo run`
- Fixed AppError to ServiceError conversion with proper handling of all variants
- Fixed module structure issues with proper exports
- Resolved import path conflicts and visibility issues
- Fixed type mismatches between service and repository layers
- Implemented missing or incomplete mod.rs files
- Pet API Database Integration has been completed
- Codebase organization has been improved with clear separation of examples and core code
- Error handling has been standardized with proper logging across endpoints
- SQLx offline mode errors have been addressed
- Type mismatches between UUID and i32 ID fields have been fixed
- User service has been successfully removed
- Cache methods in ResourceCache and CacheRegistry have been implemented and fixed
- Metrics handling has been standardized and lifetime issues resolved
- API response formats have been harmonized with a new consistent structure

## Target State
- IMMEDIATE GOAL: Zero errors when running `cargo run`
- All tests passing successfully
- Clean build with zero errors
- Up-to-date documentation reflecting current architecture
- Consistent implementation patterns across similar components
- Users service completely removed and replaced by petdb service

## Implementation Progress Tracking

### Phase 0: Critical Error Resolution (IN PROGRESS)
1. **Error Reduction Tracking** 🔄
   - [x] Create script to count and categorize build errors
   - [x] Resolve highest-frequency error types first
   - [x] Track daily error count reduction
   - [x] Prioritize errors blocking `cargo run`
   
   *Started - March 24, 2025*

2. **Database Executor Issues** 🔄
   - [ ] Update all &dyn PgPool usages to concrete types
   - [ ] Fix Executor implementations for database connections
   - [ ] Standardize connection handling across repositories
   - [ ] Verify database connection pools are properly created and managed
   
   *Started - March 24, 2025*

3. **Critical Type System Fixes** 🔄
   - [x] Fix Arc wrapper inconsistencies
   - [x] Resolve service vs repository model mismatches
   - [x] Standardize UUID vs i32 ID field usage
   - [x] Fix inconsistent trait implementations
   
   *Started - March 24, 2025*

### Phase 1: Error Analysis and Categorization (COMPLETED)
1. **Build Error Analysis** ✅
   - [x] Run `cargo build -v` to get detailed error information
   - [x] Categorize errors by type (imports, traits, implementations, etc.)
   - [x] Identify root causes for each category
   - [x] Document dependencies between errors
   
   *Completed - March 24, 2025*

2. **Test Error Analysis** ✅
   - [x] Run `cargo test -v` to get detailed test error information
   - [x] Group failing tests by module/component
   - [x] Identify common failure patterns
   - [x] Prioritize test fixes based on dependency chains
   
   *Completed - March 24, 2025*

3. **Initial Setup** ✅
   - [x] Create scripts for SQLx cache generation
   - [x] Implement MockTokenClient for testing
   - [x] Set up simplified workflow for fixes
   
   *Completed - March 24, 2025*

### Phase 2: Build Error Resolution (IN PROGRESS)
1. **Module Structure Fixes** ✅
   - [x] Resolve module visibility issues
   - [x] Fix incorrect module paths
   - [x] Update imports to reflect new module structure
   - [x] Add missing mod.rs files
   
   *Updated - March 25, 2025*

2. **Implementation Fixes** 🔄
   - [x] Resolve trait implementation errors
   - [x] Remove users service
   - [x] Tag and organize example code
   - [x] Fix type mismatches
   - [x] Fix metrics lifetime issues
   - [x] Implement missing cache methods
   - [x] Fix service and repository type conversion errors
   - [ ] Address remaining functionality issues
   
   *Updated - March 24, 2025*

3. **Example Code Organization** ✅
   - [x] Move pet API code to /examples directories
   - [x] Add @example tags to all example code
   - [x] Tag example-only dependencies
   - [x] Create example removal script
   
   *Completed - March 24, 2025*

4. **Database Integration Issues** 🔄
   - [x] Standardize error handling across endpoints
   - [x] Implement proper error logging
   - [ ] Fix database connection errors
   - [ ] Update repository implementations
   - [x] Resolve migration issues
   
   *Updated - March 24, 2025*

### Phase 3: Test Resolution (IN PROGRESS)
1. **Unit Test Fixes** (IN PROGRESS)
   - [x] Remove users service tests
   - [x] Update outdated test expectations
   - [ ] Fix mocking implementations
   - [ ] Update test data for new model changes
   
   *Started - March 24, 2025*

2. **Integration Test Fixes** (NOT STARTED)
   - [ ] Update API endpoint tests
   - [ ] Fix database test setups
   - [ ] Resolve authentication-related test failures
   
   *Not started*

3. **Test Coverage Improvement** (NOT STARTED)
   - [ ] Identify components lacking test coverage
   - [ ] Add missing tests for new functionality
   - [ ] Ensure all error paths are tested
   
   *Not started*

### Phase 4: Documentation Updates (IN PROGRESS)
1. **API Documentation** (IN PROGRESS)
   - [x] Remove users service documentation
   - [ ] Update endpoint documentation
   - [ ] Create/update Swagger annotations
   - [ ] Ensure example requests/responses are current
   
   *Started - March 24, 2025*

2. **Architecture Documentation** (NOT STARTED)
   - [ ] Update core vs. app layer documentation
   - [ ] Document new repository pattern implementation
   - [ ] Update dependency diagrams
   
   *Not started*

3. **Developer Guides** (NOT STARTED)
   - [ ] Update getting started guides
   - [ ] Update database integration guides
   - [ ] Create example implementations for common tasks
   
   *Not started*

## Implementation Status
- **Overall Progress**: 70% complete
- **Last Updated**: March 24, 2025
- **Next Milestone**: Resolve remaining errors to enable successful `cargo run` 
- **Current Focus**: 
  - Successfully fixed AppError to ServiceError conversion issues
  - Successfully resolved module structure and import path issues
  - Fixed type mismatches between service and repository layers
  - Working on database connection and executor issues

## Success Criteria
- IMMEDIATE GOAL: Zero errors when running `cargo run`
- All tests pass with `cargo test`
- Documentation accurately reflects current codebase structure
- 80%+ test coverage maintained
- Users service completely removed and replaced by petdb service

## Implementation Guidelines

### Error Resolution Approach
1. **Critical-blockers-first approach**:
   - Fix errors blocking `cargo run` first
   - Use error-tracking.mdc to avoid repeating failed approaches
   - Systematically resolve error categories in order of frequency
   - Track progress with daily error count reports

2. **Database executor issues priority**:
   - Focus on fixing the Executor trait implementation errors
   - Switch from trait objects to concrete types where possible
   - Ensure consistent error handling for database operations
   - Update all repository constructors to accept proper pool types

3. **Incremental testing**:
   - Use `cargo test <module>::` to test specific modules
   - Run full test suite periodically to check overall progress

4. **Documentation updates**:
   - Update documentation alongside code fixes
   - Follow the date format standards for "Updated at" timestamps
   - Use `date "+%B %d, %Y"` to generate the current date

5. **Legacy code removal**:
   - Being a greenfield project, we should have zero legacy or deprecated code
   - Remove all `#[deprecated]` attributes and the associated legacy functions
   - Replace any functions marked as deprecated with their modern equivalents
   - Update all import paths to reference the new function names directly

### Common Error Patterns to Look For
1. **Executor trait implementation issues**:
   - Trait Executor<'_> not implemented for &dyn PgPool
   - Inconsistent use of Arc<PgPool> vs Box<dyn PgPool>
   - Missing or incorrect lifetime parameters
   - Improper error handling in async database code

2. **Pet model integration issues**:
   - Inconsistent field names/types between pet_service::Pet and pet_repository::Pet
   - Repository implementation mismatches with PgPool and DatabaseConnection
   - Service method signatures not matching handler expectations

3. **Module structure problems**:
   - Incorrect imports after restructuring
   - Missing re-exports in mod.rs files
   - Public vs. private visibility issues

4. **Test-specific issues**:
   - ~~Remove users service tests~~ (COMPLETED)
   - Update test data fixtures to match new schemas
   - Ensure all CRUD operations are thoroughly tested

### Testing Requirements
- Run database tests with proper test isolation
- Update test data fixtures to match new schemas
- Ensure all CRUD operations are thoroughly tested

### Resources
- Detailed instructions: [docs/roadmaps/roadmap-instructions/17-codebase-cleanup-instructions.md](docs/roadmaps/roadmap-instructions/17-codebase-cleanup-instructions.md)
- Prompt template: [docs/roadmaps/roadmap-instructions/codebase-cleanup-prompt.md](docs/roadmaps/roadmap-instructions/codebase-cleanup-prompt.md)
- SQLx cache script: [scripts/generate_sqlx_cache.sh](scripts/generate_sqlx_cache.sh)
- Error tracking guide: Refer to error-tracking.mdc for known error patterns and solutions

### Example Code Tagging Standards
1. **Code Comments**:
   - Use `@example` tag for example code files
   - Use `@example_dependency` for dependencies only used by examples
   - Add brief description of what the example demonstrates

2. **Directory Structure**:
   - Move example code to /examples directories where possible
   - Keep minimal example code in main source if needed for documentation
   - Create clear separation between core and example code

3. **Dependency Management**:
   - Tag dependencies used only by examples
   - Consider making examples optional features
   - Document example-specific setup requirements