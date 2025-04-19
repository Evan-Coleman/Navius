# 01 - Simmr: Social Cooking Platform Backend

## Overview
This roadmap details the process of building a fully functional backend for Simmr, a social cooking website, using the Navius crate ecosystem. The backend will serve as both a production-ready application and a comprehensive test case for the Navius framework, helping to enhance and validate its capabilities. The app will be developed using Test-Driven Development (TDD) methodology, with incremental feature implementation and continuous verification.

## Implementation Philosophy

Before beginning implementation of the actual features, we must ensure that the Navius framework provides a stable, consistent, and developer-friendly foundation. **Framework stability and developer ergonomics are essential prerequisites** for efficient development of the recipe website.

Recent improvements in this area include:
- Converting mod.rs files to the Rust 2018 module system for better code navigation and IDE experience
- Standardizing error handling across the framework
- Improving type safety and compiler feedback
- Ensuring a Spring Boot-like developer experience

By addressing these fundamental framework issues first, we'll be able to implement the recipe website more efficiently and with higher quality code.

## Timeline and Milestones

### Phase 0: Framework Stability (Weeks 1-2)
- [x] Improve module structure (convert mod.rs files to Rust 2018 module system)
- [x] Standardize error handling patterns
- [x] Ensure consistent API patterns across framework
- [x] Improve developer experience and ergonomics
- [ ] Verify backward compatibility of core APIs
- [ ] Add comprehensive examples for common patterns

### Phase 1: Project Setup (Week 3)
- [x] Create project structure
- [x] Configure Cargo.toml with initial dependencies
- [x] Set up basic project documentation
- [x] Establish test infrastructure
- [x] Verify initial build with no errors/warnings

### Phase 2: Core Application Components (Weeks 4-5)
- [x] Design core domain model for social cooking platform
- [ ] Implement user profiles and account management service
- [ ] Set up configuration management
- [x] Implement error handling strategy
- [x] Add logging and observability
- [x] Enhance navius-core crate as needed

### Phase 3: Recipe Data Layer (Weeks 6-7)
- [ ] Implement database integration for recipe storage
- [ ] Create repository pattern for recipe management
- [ ] Add caching mechanism for popular recipes
- [ ] Implement recipe data validation
- [ ] Create data models for recipes, ingredients, and cooking steps
- [ ] Enhance navius-db and navius-cache crates as needed

### Phase 4: Social Features (Weeks 8-9)
- [ ] Implement following/follower relationships
- [ ] Create recipe sharing functionality
- [ ] Develop comment and rating system
- [ ] Add recipe collections and favorites
- [ ] Implement user activity feed
- [ ] Design notification system

### Phase 5: API Development (Weeks 10-11)
- [ ] Design RESTful API using TDD
- [ ] Implement authentication using navius-auth
- [ ] Create API endpoints for all social cooking features
- [ ] Implement request validation
- [ ] Add rate limiting and security features
- [ ] Create API documentation
- [ ] Enhance navius-http and navius-auth crates as needed

### Phase 6: Advanced Features (Weeks 12-13)
- [ ] Implement background job processing for image processing
- [ ] Add event handling for social interactions
- [ ] Implement messaging for user communications
- [ ] Set up metrics for platform usage tracking
- [ ] Create search functionality for recipes
- [ ] Develop recommendation engine
- [ ] Enhance related navius crates as needed

### Phase 7: Testing and Documentation (Weeks 14-15)
- [ ] Ensure comprehensive test coverage
- [ ] Document architectural decisions
- [ ] Create usage examples
- [ ] Polish and finalize application
- [ ] Prepare summary of enhancements to navius crates

## Simmr Backend Features

The Simmr backend will include the following key features:

1. **User Management**
   - Registration and authentication
   - Profile management
   - Following/follower relationships

2. **Recipe Management**
   - Create, read, update, delete recipes
   - Ingredient management
   - Step-by-step instructions
   - Media attachments (photos)

3. **Social Features**
   - Comments and ratings
   - Recipe sharing
   - Collections and favorites
   - Activity feed

4. **Search and Discovery**
   - Recipe search
   - Tag-based filtering
   - Recommendation engine

5. **Notifications**
   - Social interaction alerts
   - New content notifications
   - System announcements

## API Design Style

The Simmr API will be designed using a clean, declarative style with attribute macros, as shown in the following example:

```rust
mod jwt;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};
use jwt::Claims;
use serde::Deserialize;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .run()
        .await;

    tracing::info!("Simmr Server Shutdown")
}

// User routes
#[nest("/api/v1/users", middleware = [auth_required, request_logging])]
mod user_routes {
    #[route(path: "/profile", method: "GET")]
    async fn get_profile(claims: Claims) -> impl IntoResponse {
        // Return the user profile
    }
    
    #[route(path: "/follow/{user_id}", method: "POST")]
    async fn follow_user(claims: Claims, Path(user_id): Path<String>) -> impl IntoResponse {
        // Follow a user
    }
}

// Recipe routes
#[nest("/api/v1/recipes", middleware = [request_logging])]
mod recipe_routes {
    #[route(path: "/", method: "GET")]
    async fn get_recipes() -> impl IntoResponse {
        // Return a list of recipes
    }
    
    #[route(path: "/{id}", method: "GET")]
    async fn get_recipe(Path(id): Path<String>) -> impl IntoResponse {
        // Return a specific recipe
    }
    
    #[route(path: "/", method: "POST", auth_policy: "user_required")]
    async fn create_recipe(claims: Claims, Json(recipe): Json<Recipe>) -> impl IntoResponse {
        // Create a new recipe
    }
    
    #[route(path: "/{id}/rate", method: "POST", auth_policy: "user_required")]
    async fn rate_recipe(claims: Claims, Path(id): Path<String>, Json(rating): Json<Rating>) -> impl IntoResponse {
        // Rate a recipe
    }
}
```

To support this style, we'll need to develop several enhancements to the Navius crates, including:

1. **Route Macros (`#[route]`, `#[nest]`)** - For declarative route definition (NC-2).
2. **Configuration System Macros (`#[auto_config]`, `#[derive(Configurable)]`)** - For automatic configuration loading and component setup (NC-3).
3. **Dependency Injection Improvements (`Component<T>`)** - For simplified component injection (NC-4).
4. **Social Features Support** - Enhancing navius crates to better support social application patterns.
5. **Media Handling** - Adding capabilities for image processing and storage.

See the [Crate Enhancements](./sub-process/crate-enhancements.md) document for detailed tracking of these enhancements.

## Development Approach

For each feature, the following TDD approach will be strictly followed:

1. **Write Tests First**
   - Write unit tests that define the expected behavior
   - Ensure tests fail initially (Red phase)

2. **Implement Functionality**
   - Implement minimal code to make tests pass (Green phase)
   - Run cargo build to ensure no errors or warnings

3. **Refactor**
   - Improve code while maintaining test coverage
   - Consider design patterns and best practices

4. **Verify**
   - Run cargo build to ensure no errors or warnings
   - Run all tests to ensure nothing breaks
   - Check test coverage metrics

5. **Document**
   - Add code documentation
   - Update roadmap progress
   - Note any issues with navius crates

**Note on Crate Usage:** A key principle is to **maximize the use of existing Navius crate functionality** (e.g., `navius-core` for config/logging, `navius-auth` for JWT, `navius-db` for database, `navius-http` for server/middleware). Custom implementations will be avoided unless functionality is missing from the core crates. The Simmr backend should showcase idiomatic usage with minimal boilerplate.

## Navius Crate Enhancement Strategy

Throughout development, we'll identify areas where the Navius crates need enhancement:

1. **Problem Identification**
   - Document specific limitations encountered
   - Track workarounds implemented

2. **Solution Design**
   - Propose enhancements to the affected crates
   - Design improvements that maintain compatibility

3. **Implementation**
   - Make necessary changes to crates
   - Add tests for new functionality
   - Update documentation

4. **Verification**
   - Ensure Simmr backend works with enhancements
   - Verify no regressions in crate functionality

## Key Performance Indicators (KPIs)

The following metrics will be tracked throughout development:

- **Test Coverage**: Target of 90%+ for all code
- **Build Quality**: Zero errors/warnings in cargo build
- **Crate Enhancements**: Number of improvements made to navius crates
- **Documentation Quality**: Comprehensive documentation for all features
- **API Response Time**: < 100ms for non-complex requests
- **User Capacity**: Support for 10,000+ active users
- **Recipe Storage**: Support for 100,000+ recipes with efficient retrieval

## Current Progress
- Status: Framework Stability Phase
- Progress: 15%
- Updated at: May 30, 2025 