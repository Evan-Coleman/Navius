---
title: "Core Stability Roadmap"
description: "Stabilizing core functionality with Spring Boot-like developer experience"
category: roadmap
tags:
  - core
  - stability
  - developer-experience
  - spring-boot
  - ease-of-use
last_updated: March 24, 2025
version: 1.4
---

# Core Stability Roadmap

## Overview
This roadmap outlines the steps needed to achieve a stable, functioning core with Spring Boot-like developer experience. We're focusing on fixing build errors, creating intuitive abstractions, and ensuring core endpoints function properly, with an emphasis on making Navius accessible to developers coming from Java Spring Boot.

## Current Status
The codebase has been improved with several key fixes:
- ✅ Fixed router state issues and type mismatches
- ✅ Fixed service error handling and repository errors
- ✅ Fixed cache registry implementation for optional Arc references
- ✅ Fixed API resource fetch_closure implementation
- ⚠️ Some module imports need cleanup and organization
- ⚠️ Health endpoints need enhancement to match Spring Boot's format
- ⚠️ Documentation is still missing

## Target State
A stable application with:
- Clean builds with no errors
- Spring Boot-like developer experience with Rust's performance benefits
- Developer-friendly abstractions for routing, database access, caching, and error handling
- Functioning `/actuator/health` endpoint returning detailed health information
- Functioning `/actuator/info` endpoint with system information
- Simple public `/health` endpoint for basic availability checking
- Clear extension points for custom implementation
- Comprehensive examples demonstrating ease of use
- All tests passing

## Implementation Progress Tracking

### Phase 1: Fix Build Errors and Core Structure
1. **Core Naming Standardization (HIGH PRIORITY)**
   - [ ] Create consistent naming pattern for core files with `core_` prefix
     - Rename generic files like `router.rs` to `core_router.rs` to avoid conflicts
     - Rename `router/app_router.rs` to `router/core_app_router.rs`
     - Rename model files like `models/response.rs` to `models/core_response.rs`
     - Rename handler files like `handlers/health.rs` to `handlers/core_health.rs`
     - Rename utility files like `utils/api_client.rs` to `utils/core_api_client.rs`
   - [ ] Update all imports and references to reflect new naming pattern
   - [ ] Remove old non-prefixed files (backward compatibility is not needed)
   - [ ] Create user-extensible "shadow" files in app directory for customization
   - [ ] Document naming conventions and extension points
   
   *Updated at: June 14, 2024*

2. **Router Module Fixes**
   - [x] Create missing `app_router.rs` file in the core router module
   - [x] Define the `AppState` struct in the correct location
   - [x] Fix imports across the codebase to use the correct `AppState` path
   - [x] Create Spring Boot-like router abstractions for easy endpoint creation
   - [x] Implement intuitive builder pattern for router configuration
   - [x] Implement a simple public `/health` endpoint that returns server status
   
   *Updated at: May 31, 2024*

3. **Module Structure Cleanup**
   - [ ] Create missing `examples` module with Spring Boot-like implementation examples
   - [x] Fix module declarations and re-exports
   - [x] Clean up any unnecessary imports
   - [x] Ensure proper module visibility
   - [ ] Create clear separation between core and user implementation
   - [ ] Remove all backward compatibility code (not needed per latest requirements)
   - [ ] **Eliminate mod.rs files and centralize module declarations**
     - Move all module declarations to lib.rs
     - Remove all mod.rs files from the codebase
     - Use inline module declarations (e.g., `mod core { pub mod models { ... } }`)
     - Maintain explicit visibility controls and re-exports in lib.rs
     - Update imports across the codebase to reflect new module structure
   
   *Updated at: March 24, 2025*

4. **Error Handling Improvements**
   - [x] Fix ServiceError implementation to include Repository errors
   - [x] Fix error propagation between services and API layers
   - [x] Ensure proper error conversion between types
   - [ ] Create consistent error response format
   - [ ] Add request ID to error responses
   
   *Updated at: May 31, 2024*

5. **Cache System Stabilization**
   - [x] Fix cache registry to properly handle Option<Arc<CacheRegistry>>
   - [x] Fix resource fetch closures and future handling
   - [x] Fix type conversion issues in cache get/store operations
   - [x] Clean up unused variables
   - [ ] Document caching patterns and best practices
   
   *Updated at: May 31, 2024*

### Phase 2: Developer Experience Enhancement
1. **Core Abstraction Development**
   - [ ] Create intuitive database access abstractions (similar to Spring Data)
   - [x] Implement caching abstractions (similar to Spring Cache)
   - [x] Develop error handling framework with clear patterns
   - [ ] Build annotation-like macros for common patterns
   - [ ] Create dependency injection pattern that feels familiar to Spring users
   
   *Updated at: May 31, 2024*

2. **Health and Info Endpoint Implementation**
   - [x] Update health model to match Spring Boot's health endpoint format
   - [ ] Implement environment detection
   - [x] Add uptime tracking in seconds
   - [x] Create proper dependency status reporting
   - [ ] Add appropriate status codes based on health status
   - [ ] Implement info endpoint with Spring Boot-like information structure
   
   *Updated at: May 31, 2024*

### Phase 3: Documentation and Examples
1. **Spring-to-Rust Migration Guides**
   - [ ] Create comprehensive documentation for Spring Boot developers
   - [ ] Add side-by-side comparisons of Spring Boot and Navius patterns
   - [ ] Document common migration patterns and gotchas
   - [ ] Create migration checklists for different Spring Boot components
   
   *Updated at: Not started*

2. **Example Applications and Tutorials**
   - [ ] Create fully-functional example application
   - [ ] Add step-by-step tutorials for common tasks
   - [ ] Develop comprehensive API documentation with examples
   - [ ] Include working examples for auth, database, caching, and error handling
   - [ ] Add example of extending the simple `/health` endpoint with custom checks
   
   *Updated at: Not started*

### Phase 4: Testing and Refinement
1. **Comprehensive Testing**
   - [ ] Add unit tests for all core components
   - [ ] Create integration tests for typical usage patterns
   - [ ] Test with users familiar with Spring Boot for feedback
   - [ ] Ensure all tests pass
   
   *Updated at: Not started*

2. **Performance Optimization**
   - [ ] Benchmark core components
   - [ ] Optimize critical paths
   - [ ] Ensure Rust performance benefits are preserved
   - [ ] Document performance characteristics compared to Spring Boot
   
   *Updated at: Not started*

## Implementation Notes
- **Backward Compatibility**: Backward compatibility is NOT needed per project requirements. Old non-prefixed files should be removed once their core_* replacements are complete.
- **File Naming**: All core framework files should use the core_* prefix without exception.
- **Module Structure**: Clear separation between framework code and user extension points should be maintained.
- **Module Organization**: All module declarations should be centralized in lib.rs without using mod.rs files to reduce file clutter and simplify the codebase structure.

## Implementation Status
- **Overall Progress**: 30% complete
- **Last Updated**: March 24, 2025
- **Next Milestone**: Finish module structure cleanup and enhance health endpoints
- **Current Focus**: Health endpoint enhancement and examples module

## Success Criteria
- Server builds with zero errors
- Server starts successfully
- Simple `/health` endpoint returns server status with 200 OK
- Java Spring Boot developers can create new endpoints with minimal code
- Core abstractions are intuitive and well-documented
- `/actuator/health` endpoint returns proper health information with all dependencies
- `/actuator/info` endpoint returns system information
- All tests pass
- Example applications demonstrate ease of use
- API documentation is complete and accurate 