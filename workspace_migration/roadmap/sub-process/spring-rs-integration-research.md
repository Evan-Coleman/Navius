# spring-rs Integration Research

*This document tracks the research and potential integration of spring-rs patterns into the Navius workspace architecture. This is a sub-process of the [main workspace migration roadmap](../40-workspace-migration.md).*

## Overview

We are researching the spring-rs framework to identify beneficial architectural patterns that could enhance our workspace migration. The goal is to selectively adopt patterns that align with our architecture while maintaining our core decision to use specialized crates with clear boundaries.

## Research Phase (In Progress)

### Key Areas Examined

1. **Plugin Architecture**
   - ✅ Study how spring-rs implements `.add_plugin()` mechanism
   - ✅ Analyze plugin lifecycle management
   - ✅ Evaluate plugin-to-plugin communication patterns
   - ✅ Research extensibility approaches

2. **Component Extraction**
   - ✅ Examine `Component<T>` extractor pattern for dependency injection
   - ✅ Analyze how components are registered and discovered
   - ✅ Study component lifecycle management

3. **Auto-configuration**
   - ✅ Research the `#[auto_config]` macro implementation
   - ✅ Analyze configuration hierarchy and precedence rules
   - ✅ Study how conditional configuration works

4. **Procedural Macros**
   - ✅ Identify use cases for procedural macros in our architecture
   - ✅ Evaluate the complexity-to-benefit ratio
   - ✅ Research how spring-rs uses macros for routing and dependency injection

5. **Configuration Management**
   - ✅ Study the `Configurable` trait and `#[config_prefix]` attribute
   - ✅ Analyze how configuration values are loaded and validated
   - ✅ Research configuration overrides and environment-specific configs

## Research Findings

### Plugin Architecture

Spring-rs implements a plugin system through a central registry that manages the lifecycle of plugins:

1. **Plugin Registration**:
   - Plugins are registered using the `.add_plugin()` method on the application builder
   - Each plugin implements the `Plugin` trait with lifecycle hooks
   - Plugins can depend on other plugins, forming a dependency graph

2. **Plugin Discovery**:
   - Plugins are explicitly added rather than auto-discovered
   - This provides clear control over which plugins are active
   - Dependencies between plugins are resolved at startup

3. **Plugin-to-Plugin Communication**:
   - Plugins communicate through shared components in the application context
   - They can register services that other plugins can consume
   - Events can be published and subscribed to by different plugins

**Benefits for Navius**:
- Clear separation of concerns through modular plugins
- Explicit plugin dependencies provide predictable initialization
- Ability to enable/disable features at the plugin level

### Component Extraction

Spring-rs implements dependency injection through a component system:

1. **Component Registration**:
   - Components are registered with the application context
   - The `#[component]` attribute macro simplifies component definition
   - Components can be registered as singletons (default) or prototypes

2. **Component Resolution**:
   - Components are extracted using the `Component<T>` extractor
   - Dependencies are resolved automatically
   - Type-safe dependency injection with compile-time validation

3. **Lifecycle Management**:
   - Components have initialization and destruction hooks
   - Ordered initialization based on dependencies
   - Support for async initialization and shutdown

**Benefits for Navius**:
- Type-safe dependency injection
- Reduced boilerplate code when wiring services
- Clear component lifecycle management

### Auto-configuration

Spring-rs provides auto-configuration through procedural macros:

1. **Configuration Detection**:
   - The `#[auto_config]` macro automatically configures components based on conditions
   - Configuration is applied based on available dependencies
   - Conditional configuration based on environment, features, or other conditions

2. **Configuration Hierarchy**:
   - Configuration values cascade from different sources (files, environment, code)
   - Clear precedence rules determine which value wins
   - Support for profiles (dev, test, prod) for environment-specific configuration

**Benefits for Navius**:
- Reduced configuration boilerplate
- Sensible defaults with the ability to override
- Environment-specific configuration

### Procedural Macros

Spring-rs uses procedural macros extensively to simplify common patterns:

1. **Route Definition**:
   - Macros for defining API routes and handlers
   - Automatic parameter extraction from requests
   - Response mapping based on return types

2. **Dependency Injection**:
   - The `#[component]` macro registers a struct as a component
   - The `#[autowired]` macro injects dependencies
   - Type-safe dependency resolution at compile time

3. **Configuration Binding**:
   - The `#[config_prefix]` macro binds configuration to structs
   - Automatic validation of configuration values
   - Default values and documentation through attributes

**Benefits for Navius**:
- Reduced boilerplate code
- Clearer API definition and documentation
- Type safety for configuration and dependencies

### Configuration Management

Spring-rs provides robust configuration management:

1. **Configuration Sources**:
   - Multiple sources (TOML, environment variables, command line)
   - Clear precedence rules between sources
   - Profile-specific configuration files

2. **Type-safe Configuration**:
   - Configuration values are bound to typed structs
   - Validation at startup ensures configuration is valid
   - Default values provide resilience

3. **Reloadable Configuration**:
   - Configuration can be reloaded at runtime
   - Components can subscribe to configuration changes
   - Hot-reload support for development environments

**Benefits for Navius**:
- Structured, typed configuration
- Validation to catch configuration errors early
- Support for different environments

## Implementation Recommendations

Based on our research, we recommend the following patterns for adoption in Navius:

1. **Limited Plugin System**:
   - Implement a plugin system within specific crates (e.g., navius-http, navius-db)
   - Define clear plugin interfaces with lifecycle hooks
   - Use explicit registration rather than auto-discovery

2. **Component Management**:
   - Create a lightweight component registry for dependency injection
   - Implement both singleton and prototype scopes for different use cases
   - Support async initialization and shutdown

3. **Configuration Management**:
   - Adopt hierarchical, typed configuration with validation
   - Support for environment-specific configuration overrides
   - Implement a lighter weight version of configuration binding

4. **Selective Macro Usage**:
   - Develop targeted macros for repetitive patterns
   - Focus on type safety and reduced boilerplate
   - Balance between explicitness and convenience

## Implementation Plan

1. **Phase 1: Core Infrastructure**
   - ⬜️ Develop lightweight plugin trait and registry
   - ⬜️ Implement component registration and resolution
   - ⬜️ Create configuration management system

2. **Phase 2: Crate Integration**
   - ⬜️ Integrate plugin support into navius-http
   - ⬜️ Add component support to navius-db
   - ⬜️ Implement configuration binding for all crates

3. **Phase 3: Developer Experience**
   - ⬜️ Create helper macros for common patterns
   - ⬜️ Develop documentation and examples
   - ⬜️ Create testing utilities for the new architecture

## Next Steps

1. Create prototypes for the recommended patterns
2. Evaluate the impact on the existing codebase
3. Develop guidelines for implementing these patterns
4. Update the main roadmap with the new integration plan

## Timeline Impact

- Adding this research phase has extended the timeline by approximately 1 week
- Implementation of the selected patterns will be integrated into existing phases
- Expected completion of initial implementation: by the end of Phase 3

*This document was created on March 29, 2025 and last updated on May 30, 2025* 