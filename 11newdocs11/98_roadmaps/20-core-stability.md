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
last_updated: March 27, 2025
version: 1.5
---

# Core Stability Roadmap

**Goal**: Create a stable core with a Spring Boot-like developer experience

## Current Status
- Initial endpoints implemented
- Basic routing and middleware in place
- Health endpoints in place but need enhancements
- Information endpoint needs implementation
- Actuator endpoints need standardization
- Need cleaner structure for Spring Boot developers

## Target State
We want Navius to feel familiar to Spring Boot developers, with clear conventions and an intuitive structure. The developer should be able to easily understand where to place code and how to approach common tasks.

## Implementation Progress

### Phase 1: Core Infrastructure
- [x] Enhance health endpoint with detailed responses
- [x] Implement info endpoint with application, build, git and environment info
- [x] Clean up module declarations to reduce mod.rs files
- [ ] Complete module declaration restructuring to eliminate all mod.rs files
  - [x] Remove mod.rs files from examples directory (completed)
  - [x] Remove initial mod.rs files from app directory (partially completed)
  - [x] Remove initial mod.rs files from core directory (partially completed)
  - [ ] Remaining app directory mod.rs files to be converted
  - [ ] Remaining core directory mod.rs files to be converted
  - [ ] Test directory mod.rs files to be converted
- [x] Create app module structure (controllers, services, repositories)
- [x] Add custom extensions to health endpoint

### Phase 2: Core Abstraction Development
- [x] Build annotation-like macros for common patterns (RestController, RequestMapping, etc.)
  - [x] Create macro definitions for RestController, RequestMapping, Service, etc.
  - [x] Implement example controllers using these macros
  - [x] Implement example services with caching annotations
  - [x] Document extension points for custom annotations

### Phase 3: Documentation and Examples
- [x] Create comprehensive documentation for Spring Boot developers
- [ ] Document common migration patterns and gotchas
- [ ] Create migration checklists for different Spring Boot components
- [ ] Add step-by-step tutorials for common tasks
- [ ] Develop comprehensive API documentation with examples

## Success Criteria
- Navius can be built with stable Rust
- Zero critical/high CVEs
- Documentation clear for Spring Boot developers
- Examples cover common patterns from Spring Boot 