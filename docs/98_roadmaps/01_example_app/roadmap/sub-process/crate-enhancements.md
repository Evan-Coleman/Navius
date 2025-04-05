# Navius Crate Enhancement Tracking

## Overview
This document tracks enhancements to Navius crates identified during the development of the example app. As we implement features using TDD, we'll document limitations, enhancement proposals, and implementations.

## Related Documents
- [Main Roadmap](../01-example-app.md)
- [Implementation Plan](../example-app-plan.md)

## Enhancement Process

For each enhancement:

1. **Problem Identification**
   - Document specific limitations encountered
   - Describe the use case that revealed the limitation
   - Note any workarounds implemented

2. **Enhancement Proposal**
   - Propose specific changes to the affected crate
   - Document the expected benefits
   - Consider backward compatibility

3. **Implementation**
   - Create a branch for the enhancement
   - Implement with TDD methodology
   - Add tests for new functionality
   - Update documentation

4. **Verification**
   - Test the enhancement in the example app
   - Verify no regressions in the crate
   - Document the updated usage

## Main Application Example Support Requirements

Based on the target main.rs file example, we've identified the following key enhancements needed:

### Required Macro Support
- `#[auto_config(WebConfigurator)]` - For automatic configuration loading
- `#[routes]`, `#[get]`, `#[post]` - For route definition
- `#[route]` with method parameters - For multi-method routes
- `#[nest]` - For nested route modules
- `#[config_prefix]` - For configuration prefix specification
- `#[derive(Configurable)]` - For auto-configurable types

### Required Component Features
- Plugin system for modular application setup
- Dependency injection for components
- Automatic parameter extraction in handlers
- JWT integration for authentication
- SQL integration with component injection
- Configuration system with type-safe config extraction

## Crate Enhancement Tracking

### navius-core

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NC-1 | Plugin System | Completed | Implement plugin system for modular application setup | Implemented App and AppBuilder classes that leverage the existing navius-plugin Registry. Created SqlxPlugin and WebPlugin implementations. |
| NC-2 | Application Builder | Not Started | Create App builder pattern for clean initialization | Should support add_plugin() and run() methods |
| NC-3 | Component Registration | Not Started | Implement component registration and retrieval system | Required for dependency injection |

### navius-http

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NH-1 | Route Macros | Not Started | Implement macros for route definition (#[routes], #[get], etc.) | Should support multiple route definitions per handler |
| NH-2 | Nested Routes | Not Started | Support for nested route modules with #[nest] | Allow modular organization of routes |
| NH-3 | Response Types | Not Started | Implement IntoResponse trait | Allow diverse return types from handlers |
| NH-4 | Path Parameters | Not Started | Support path parameter extraction | Extract parameters from URL paths |
| NH-5 | WebPlugin | Not Started | Create web server plugin | Should integrate with App::new() builder |

### navius-db / navius-db-postgres

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| ND-1 | SqlxPlugin | Not Started | Create SQLx integration plugin | Should integrate with App::new() builder |
| ND-2 | Connection Pool | Not Started | Implement ConnectPool component | Should be injectable into handlers |
| ND-3 | Query Helpers | Not Started | Provide simplified query interface | Should wrap SQLx functionality |

### navius-auth / navius-auth-entra

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NA-1 | JWT Support | Not Started | Implement JWT generation and validation | Required for user authentication |
| NA-2 | Claims Extraction | Not Started | Auto-extract claims from requests | Allow direct injection of Claims into handlers |
| NA-3 | Authentication Middleware | Not Started | Create authentication middleware | Should validate JWT tokens |

### navius-di

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NDI-1 | Component Type | Not Started | Create Component<T> wrapper for DI | Allow automatic injection of components |
| NDI-2 | Service Registration | Not Started | Implement service registration system | Support for registering services with the DI container |
| NDI-3 | Parameter Extraction | Not Started | Auto-extract dependencies in handlers | Automatically provide dependencies to handlers |

### navius-config

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NCF-1 | auto_config Macro | Not Started | Implement #[auto_config] macro | Automatically load and configure application |
| NCF-2 | Configurable Derive | Not Started | Implement #[derive(Configurable)] | Generate code for configuration structs |
| NCF-3 | Config Extraction | Not Started | Create Config<T> wrapper | Allow auto-extraction of configuration |
| NCF-4 | Config Prefix | Not Started | Support #[config_prefix] | Allow specifying config key prefixes |

### navius-test / navius-test-utils

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NT-1 | Route Testing | Not Started | Create utilities for route testing | Allow testing routes without server |
| NT-2 | Plugin Mocking | Not Started | Support for mocking plugins | Enable testing with mock plugins |
| NT-3 | Configuration Testing | Not Started | Tools for testing configuration | Verify correct configuration loading |

## Enhancement Proposals

### Enhancement Proposal: Plugin System

**ID**: NC-1  
**Crate**: navius-core  
**Title**: Plugin System Implementation  
**Status**: Proposed  

**Problem Statement**:  
The main.rs example shows a clean application setup with plugins (SqlxPlugin, WebPlugin), but we currently lack a standardized plugin system in the Navius ecosystem.

**Use Case**:  
Developers need a way to modularly extend application functionality through plugins without tightly coupling components. This enables better separation of concerns and more maintainable code.

**Proposed Solution**:  
Implement a trait-based plugin system:

```rust
pub trait Plugin {
    fn build(&self, app: &mut AppBuilder);
    fn name(&self) -> &str;
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }
}

pub struct AppBuilder {
    components: HashMap<TypeId, Box<dyn Any>>,
    plugins: Vec<Box<dyn Plugin>>,
}

impl AppBuilder {
    pub fn add_plugin<P: Plugin + 'static>(&mut self, plugin: P) -> &mut Self {
        self.plugins.push(Box::new(plugin));
        self
    }
    
    pub fn run(self) -> App {
        // Initialize and run application
    }
}
```

**Benefits**:  
- Clean, fluent API for application setup
- Modular design with pluggable components
- Clear dependency management between plugins
- Simplified testing through mock plugins

**Backward Compatibility**:  
This is a new feature, so backward compatibility is not a concern.

**Implementation Plan**:  
1. Define the Plugin trait
2. Create the AppBuilder with plugin registration
3. Implement app startup process with plugin initialization
4. Create standard plugins (SqlxPlugin, WebPlugin)
5. Add comprehensive tests and documentation

### Enhancement Proposal: Route Macros

**ID**: NH-1  
**Crate**: navius-http  
**Title**: Route Definition Macros  
**Status**: Proposed  

**Problem Statement**:  
The main.rs example shows elegant route definition using attributes like #[routes], #[get], and #[post], but we need to implement these macros to enable this declarative style.

**Use Case**:  
Developers need a clean, expressive way to define routes without boilerplate. The attribute-based approach is more readable and maintainable than manual route registration.

**Proposed Solution**:  
Implement a set of procedural macros for route definition:

```rust
#[proc_macro_attribute]
pub fn routes(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Mark function for route collection
}

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Generate route registration for GET method
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Generate route registration for POST method
}

#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Generate route registration for multiple methods
}

#[proc_macro_attribute]
pub fn nest(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Generate nested route module
}
```

**Benefits**:  
- Declarative route definition
- Reduced boilerplate
- Improved code readability
- Support for method-specific handlers

**Backward Compatibility**:  
This is a new feature, but the design should allow for both macro-based and manual route registration to coexist.

**Implementation Plan**:  
1. Create proc-macro crate for route attribute macros
2. Implement route collection mechanism
3. Support method constraints (GET, POST, etc.)
4. Add nested route support
5. Provide comprehensive testing and documentation

## Current Status
- Status: Not Started
- Progress: 0%
- Updated at: May 30, 2024 