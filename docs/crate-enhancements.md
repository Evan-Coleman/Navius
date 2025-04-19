# Navius Crate Enhancement Plan

This document outlines the planned enhancements for the Navius crate ecosystem to support the Simmr social cooking platform backend.

## Enhancement Table

| ID    | Enhancement | Status | Priority | Implementation Notes |
|-------|------------|--------|----------|---------------------|
| NC-15 | Zero Boilerplate Initiative | In Progress | High | Eliminating unnecessary infrastructure code from user applications |
| NC-11 | WebPlugin Abstraction | In Progress - 70% | High | WebPlugin implementation complete in navius-http crate with examples and documentation |
| NC-12 | WebConfigurator Abstraction | In Progress | High | Standardizing the WebConfigurator implementation |
| NC-13 | App Builder Simplification | In Progress | High | Simplifying the Application builder |
| NC-14 | Main Entry Point Simplification | In Progress | High | Creating macros for a simplified main function |
| NC-01 | Database Integration | Complete | Medium | Standardizing data access |
| NC-02 | Plugin Architecture | Complete | Medium | Standardizing plugin development |
| NC-03 | Configuration Management | Complete | Medium | Standardizing configuration |
| NC-04 | Authentication | Planned | Medium | Standardizing authentication |
| NC-05 | Metrics | Planned | Medium | Standardizing metrics collection |
| NC-06 | Logging | Complete | Low | Standardizing logging |
| NC-07 | Caching | Planned | Low | Standardizing caching |
| NC-08 | Health Checks | Planned | Low | Standardizing health checks |
| NC-09 | Rate Limiting | Planned | Low | Standardizing rate limiting |
| NC-10 | Circuit Breaking | Planned | Low | Standardizing circuit breaking |

## Implementation Plans

### NC-15: Zero Boilerplate Initiative

**Current Status**: 35% Complete  
**Team**: Core Framework Team  
**Timeline**: 7 weeks  

**Technical Approach**:
1. Extract `WebPlugin` implementation into a dedicated crate
2. Extract `WebConfigurator` implementation into a dedicated crate
3. Create a simplified `App` builder API
4. Create macros for route declarations
5. Create a declarative main function macro

**Expected Outcome**:
- 80% reduction in infrastructure code required from users
- Focus on business logic rather than plumbing
- Improved developer onboarding experience

### NC-11: WebPlugin Abstraction

**Current Status**: 70% Complete  
**Team**: Web Team  
**Timeline**: 4 weeks  

**Technical Approach**:
1. ✅ Move `WebPlugin` from application code to `navius_http` crate
2. ✅ Standardize interface for web functionality
3. ✅ Provide default implementations for common use cases
4. 🔄 Implement automatic route discovery

**Recent Progress**:
- Implemented `WebPlugin` in the navius-http crate
- Integrated with existing `HttpServerConfig` for server setup
- Added comprehensive documentation with usage examples
- Created unit tests for the implementation
- Added a working example in the navius-http crate

**Expected Outcome**:
- Elimination of custom `WebPlugin` code in user applications
- Support for declarative route definitions
- Integration with authentication framework

### NC-12: WebConfigurator Abstraction

**Current Status**: 30% Complete  
**Team**: Configuration Team  
**Timeline**: 3 weeks  

**Technical Approach**:
1. Move `WebConfigurator` from application code to `navius_config` crate
2. Create a configuration schema for web settings
3. Implement environment variable overrides
4. Add support for profiles (dev, test, prod)

**Expected Outcome**:
- Standard configuration mechanisms for all web properties
- Environment-aware configuration
- Elimination of custom configuration code

### NC-13: App Builder Simplification

**Current Status**: 15% Complete  
**Team**: Core Framework Team  
**Timeline**: 5 weeks  

**Technical Approach**:
1. Create a declarative application builder
2. Implement automatic plugin discovery
3. Provide sensible defaults for all components
4. Create a dependency injection system

**Expected Outcome**:
- Single line application instantiation
- Automatic component registration
- Simplified component dependency management

## Final Target Architecture

After implementing the Zero Boilerplate Initiative, the target architecture will involve:

1. **User Code**:
   - `main.rs` - Minimal entry point with macro annotations
   - `handlers/*.rs` - Route handlers with declarative annotations
   - `services/*.rs` - Business logic in services
   - `models/*.rs` - Domain model definitions

2. **Framework Code** (Moved from user space):
   - `WebConfigurator` → navius-config crate
   - `WebPlugin` → navius-http crate
   - `App`/`AppBuilder` → navius-core crate
   - Route registration → navius-http macros
   - Configuration loading → navius-config macros

## Current Status

As of May 30, 2025, the overall progress of the Zero Boilerplate Initiative is at 35%. Teams have made significant progress on the WebPlugin Abstraction (70% complete), with work continuing on the other components. 