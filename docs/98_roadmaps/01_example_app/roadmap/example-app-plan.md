# Simmr: Social Cooking Platform Implementation Plan

## Application Overview

Simmr is a comprehensive social cooking platform that will serve as both a production-ready application and a showcase for the capabilities of the Navius crate ecosystem. The platform will allow users to:

1. Create and share recipes with detailed ingredients and steps
2. Follow other users and discover their recipes
3. Comment on and rate recipes
4. Create collections of favorite recipes
5. Search for recipes by ingredients, tags, or users
6. Receive notifications about social interactions
7. View personalized activity feeds

## Technical Stack

- **Framework**: Axum web framework
- **Database**: PostgreSQL via navius-db-postgres
- **Authentication**: JWT and OAuth via navius-auth
- **Caching**: Redis via navius-cache
- **API Documentation**: OpenAPI/Swagger
- **Media Storage**: Cloud storage via navius-media
- **Search**: Full-text search via navius-search
- **Messaging**: Real-time notifications via navius-messaging
- **Metrics**: Prometheus via navius-metrics-prometheus
- **Testing**: navius-test and navius-test-utils

## Application Structure

The Simmr backend will follow a clean, declarative style with modular organization:

```rust
mod jwt;
mod user;
mod recipe;
mod social;
mod media;
mod search;

use axum::http::StatusCode;
use jwt::Claims;
use serde::Deserialize;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .add_plugin(MediaPlugin)
        .add_plugin(SearchPlugin)
        .add_plugin(SocialPlugin)
        .run()
        .await;

    tracing::info!("Simmr Server Shutdown")
}

// Example API modules will be defined here using nest and route macros
// for user profiles, recipe management, social features, etc.
```

## Implementation Details

### Phase 1: Project Setup and Core Framework

#### Step 1.1: Basic Project Structure
- [x] Set up project with cargo new
- [x] Configure Cargo.toml with initial dependencies
- [x] Set up directory structure

#### Step 1.2: Core Framework Components
- [x] Implement App builder pattern
- [x] Create Plugin trait and registration mechanism
- [x] Implement basic application lifecycle management
- [x] Create route collection and registration system

#### Step 1.3: Route Macro Implementation
- [x] Create proc-macro crate for route macros
- [x] Implement #[route] macro with method parameter
- [x] Add support for nested routes with #[nest]
- [x] Implement path parameter extraction

#### Step 1.4: Configuration System
- [ ] Implement #[auto_config] macro
- [ ] Create Configurable derive macro
- [ ] Build configuration loading system
- [ ] Add support for configuration prefixes

### Phase 2: User Management and Authentication

#### Step 2.1: User Domain Model
- [ ] Implement User entity and validation
- [ ] Create profile management functionality
- [ ] Build authentication system

#### Step 2.2: User API
- [ ] Create user registration endpoint
- [ ] Implement authentication endpoints
- [ ] Build profile management API
- [ ] Add avatar upload functionality

#### Step 2.3: JWT Authentication
- [ ] Implement JWT generation and validation
- [ ] Create authentication middleware
- [ ] Add Claims extraction for handlers
- [ ] Implement social login (OAuth)

### Phase 3: Recipe Management

#### Step 3.1: Recipe Domain Model
- [ ] Implement Recipe entity with ingredients and steps
- [ ] Create tag and category system
- [ ] Build media reference model for photos
- [ ] Implement nutrition information tracking

#### Step 3.2: Recipe Services
- [ ] Implement RecipeService with creation and updates
- [ ] Create IngredientService for ingredient management
- [ ] Build TagService for recipe categorization
- [ ] Implement media handling for recipe photos

#### Step 3.3: Recipe Repositories
- [ ] Implement database schema with migrations
- [ ] Create Repository interfaces
- [ ] Implement PostgreSQL repositories
- [ ] Add caching for popular recipes

#### Step 3.4: Recipe API
- [ ] Create recipe CRUD endpoints
- [ ] Implement ingredient management
- [ ] Build tag and category endpoints
- [ ] Add media upload functionality

### Phase 4: Social Features

#### Step 4.1: Following System
- [ ] Implement follower/following relationships
- [ ] Create following management endpoints
- [ ] Build user discovery functionality
- [ ] Add notification system for new followers

#### Step 4.2: Engagement Features
- [ ] Implement comments and ratings
- [ ] Create recipe sharing functionality
- [ ] Build collections and favorites
- [ ] Implement activity feed generation

#### Step 4.3: Social API
- [ ] Create follow/unfollow endpoints
- [ ] Implement comment and rating API
- [ ] Build sharing endpoints
- [ ] Add activity feed endpoints

### Phase 5: Search and Discovery

#### Step 5.1: Search Infrastructure
- [ ] Implement full-text search for recipes
- [ ] Create user search functionality
- [ ] Build tag-based filtering
- [ ] Implement autocomplete suggestions

#### Step 5.2: Recommendation Engine
- [ ] Create basic recommendation algorithms
- [ ] Implement personalized suggestions
- [ ] Build trending recipes functionality
- [ ] Add seasonal recipe highlights

#### Step 5.3: Search and Discovery API
- [ ] Create search endpoints
- [ ] Implement filter endpoints
- [ ] Build recommendation API
- [ ] Add exploration endpoints

### Phase 6: Advanced Features

#### Step 6.1: Notifications
- [ ] Implement real-time notifications
- [ ] Create notification preferences
- [ ] Build notification center
- [ ] Add email notifications

#### Step 6.2: Media Processing
- [ ] Implement image upload and storage
- [ ] Create thumbnail generation
- [ ] Build CDN integration
- [ ] Add image optimization

#### Step 6.3: Metrics and Analytics
- [ ] Implement usage tracking
- [ ] Create performance metrics
- [ ] Build analytics dashboard
- [ ] Add health monitoring

### Phase 7: Testing and Documentation

#### Step 7.1: Testing
- [ ] Complete test coverage
- [ ] Implement integration tests
- [ ] Add performance tests
- [ ] Build CI/CD pipeline

#### Step 7.2: Documentation
- [ ] Complete API documentation
- [ ] Create usage guides
- [ ] Document architecture decisions
- [ ] Build developer portal

## Implementation Strategy

Our approach will be to build the Simmr platform incrementally:

1. **Core Platform First**
   - Start with user management and authentication
   - Implement basic recipe functionality
   - Build minimal viable social features

2. **Feature Expansion**
   - Add more sophisticated social features
   - Implement search and discovery
   - Build media handling capabilities

3. **Polish and Optimize**
   - Enhance performance with caching
   - Optimize search and recommendations
   - Improve user experience

4. **Scale and Monitor**
   - Add metrics and monitoring
   - Implement scalability features
   - Build administrative tools

Throughout this process, we'll identify and implement necessary enhancements to the Navius crate ecosystem.

## Navius Crate Enhancement Focus Areas

The implementation will require several enhancements to existing Navius crates and the creation of new ones:

1. **navius-core**
   - Enhanced error handling
   - Improved configuration system
   - Telemetry integration

2. **navius-http**
   - Media upload handling
   - Rate limiting
   - API versioning

3. **navius-db-postgres**
   - Specialized repositories for recipes
   - Social graph storage optimization
   - Full-text search integration

4. **navius-auth**
   - Social login capabilities
   - Role-based access control
   - Enhanced JWT management

5. **navius-media** (new crate)
   - Image upload and storage
   - Thumbnail generation
   - CDN integration

6. **navius-social** (new crate)
   - Following system
   - Activity feed generation
   - Notification management

7. **navius-search** (new crate)
   - Recipe search functionality
   - Autocomplete suggestions
   - Recommendation algorithms

These enhancements will be tracked in the [Crate Enhancements](./sub-process/crate-enhancements.md) document.

## Success Criteria

The Simmr implementation will be considered successful when:

1. All core features are implemented and working correctly
2. Test coverage exceeds 90%
3. No errors or warnings in cargo build
4. All identified crate enhancements are implemented
5. API response times are under 100ms for non-complex requests
6. The platform can support at least 10,000 active users
7. Recipe storage can handle at least 100,000 recipes
8. Comprehensive documentation is available

## Current Status
- Status: In Progress
- Progress: 12%
- Updated at: April 06, 2025 