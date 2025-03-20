# Dependency Injection Roadmap

## Overview
Spring Boot's powerful dependency injection (DI) system is one of its core strengths, allowing for clean, decoupled, and testable code. This roadmap outlines steps to implement a similar system in our Rust backend framework.

## Current State
Currently, our application passes `Arc<AppState>` manually to handlers and services, which works but lacks the flexibility and declarative nature of Spring's DI system.

## Target State
A lightweight yet powerful DI container that:
- Allows declarative registration of services and components
- Supports automatic resolution of dependencies
- Enables easy mocking for testing
- Maintains Rust's compile-time safety guarantees

## Implementation Steps

### Phase 1: Basic DI Container
1. **Create DI Container Interface**
   - Define traits for registering services
   - Implement lookup mechanisms for retrieving services
   - Add support for singleton and transient service lifetimes

2. **Implement Service Registration Macros**
   - Create ergonomic macros for service registration
   - Add compile-time checking where possible to maintain Rust's safety guarantees

3. **Build Service Provider**
   - Develop a centralized service provider
   - Implement factory pattern for service instantiation
   - Add support for lazy initialization

### Phase 2: Handler Integration
1. **Create Extractor Middleware**
   - Implement Axum extractors that pull services from the DI container
   - Ensure proper error handling for missing dependencies

2. **Build Handler Dependency Decorators**
   - Create macros that allow declaring handler dependencies
   - Ensure minimal runtime overhead

### Phase 3: Scoped Services
1. **Implement Request Scoping**
   - Add support for services scoped to the request lifetime
   - Ensure proper cleanup at the end of the request

2. **Build Hierarchical Service Resolution**
   - Support parent-child container relationships
   - Allow service overriding in child containers

### Phase 4: Testing Support
1. **Create Mock Service Providers**
   - Build a simplified mock service container for tests
   - Support easy service substitution in test environments

2. **Add Test Utility Functions**
   - Provide helper functions to easily set up test services
   - Create a standardized testing pattern

## Success Criteria
- Services can be registered and resolved with minimal boilerplate
- Handlers can declare their dependencies declaratively
- Testing with mock services is straightforward
- Runtime overhead is minimal
- Compile-time safety is maintained where possible

## Implementation Notes
This approach should balance between Spring Boot's flexible DI system and Rust's emphasis on compile-time safety. Complete dynamic resolution may not be possible in all cases given Rust's type system, but we should aim for maximum ergonomics while maintaining safety.

## References
- [Spring Framework IoC Container](https://docs.spring.io/spring-framework/reference/core/beans.html)
- [Shaku - Rust DI Container](https://crates.io/crates/shaku)
- [Factory Pattern in Rust](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html) 