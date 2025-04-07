# Crate Enhancements for Simmr - Social Cooking Platform

This document tracks the specific enhancements required in the Navius crate ecosystem to support the Simmr social cooking platform backend.

| Enhancement ID | Crate(s) Affected | Title                   | Description                                                                                                                               | Status      | Implementation Notes                                         |
| -------------- | ----------------- | ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------------------ |
| NC-1           | navius-core       | Plugin/DI System        | Initial DI and config loading via `ApplicationBuilder`. Basic error handling improvements. (Initial plugin concept superseded by direct DI). | Completed   | Implemented via `ApplicationBuilder` refactoring.            |
| NC-2           | navius-http       | Declarative Route Macros | Implement `#[route(path="...", method=..., ...)]` and `#[nest(...)]` macros for defining routes and applying middleware/auth declaratively.   | Completed   | Basic implementation functioning; API routes working with proper nesting. |
| NC-3           | navius-core       | Config Macros           | Implement `#[auto_config]` and `#[derive(Configurable)]` for automated configuration setup and struct mapping.                              | Not Started | Requires proc-macro development.                             |
| NC-4           | navius-di         | DI Improvements         | Implement `Component<T>` wrapper or similar extractor for simplified, type-safe component injection in handlers/services.                     | Not Started | Explore alternatives like direct `State` extraction vs. wrapper. |
| NC-5           | navius-db         | Repository Abstraction  | Define standard repository traits/macros for recipe and user data. Simplify common CRUD operations.                                       | Not Started | Consider base traits or codegen for repositories.          |
| NC-6           | navius-auth       | Auth Policy Integration | Standardize how `auth_policy` in `#[route]` maps to specific auth middleware/configurations for user authentication.                      | Not Started | Define policy registration/lookup mechanism.               |
| NC-7           | navius-media      | Media Processing        | Add support for recipe image upload, processing, and storage.                                                                           | Not Started | Implement streaming uploads, image optimization, and CDN integration. |
| NC-8           | navius-social     | Social Features         | Create new crate with common social media patterns: following, sharing, activity feeds.                                                  | Not Started | Build reusable components for social interactions.         |
| NC-9           | navius-search     | Search Functionality    | Implement search capabilities for recipes, ingredients, and users.                                                                       | Not Started | Consider integration with Elasticsearch or similar technology. |
| NC-10          | navius-metrics    | Usage Analytics         | Track platform usage patterns, popular recipes, and user engagement metrics.                                                             | Not Started | Design metrics collection that respects privacy concerns.  |

---
*Updated at: April 06, 2025*

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
   - Test the enhancement in the Simmr backend
   - Verify no regressions in the crate
   - Document the updated usage

## Simmr Application Requirements

Based on the Simmr backend requirements, we've identified the following key enhancements needed:

### Required Macro Support
- `#[auto_config(WebConfigurator)]` - For automatic configuration loading
- `#[route]` with method parameters - For RESTful API routes
- `#[nest]` - For organizing routes by resource (recipes, users, etc.)
- `#[derive(Configurable)]` - For auto-configurable types

### Required Component Features
- User authentication and authorization
- Recipe data storage and retrieval
- Social interaction patterns (following, sharing, etc.)
- Media handling for recipe images
- Search functionality
- Analytics and metrics collection

## Detailed Crate Enhancement Tracking

### navius-core

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NC-1 | Plugin System | Completed | Implement plugin system for modular application setup | Implemented App and AppBuilder classes that leverage the existing navius-plugin Registry. Created SqlxPlugin and WebPlugin implementations. |
| NC-2 | Application Builder | Completed | Create App builder pattern for clean initialization | Supports add_plugin() and run() methods |
| NC-3 | Component Registration | In Progress | Implement component registration and retrieval system | Required for dependency injection |
| NC-4 | Error Handling | Completed | Implement standardized error handling for API responses | Consistent error pattern implemented |
| NC-5 | Repository Abstraction | Not Started | Define standard repository traits for recipe and user data | Will include specializations for social cooking context |
| NC-6 | Configuration System | Not Started | Implement hierarchical config with environment overrides | Support for complex Simmr configuration needs |

### navius-http

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NH-1 | Route Macros | Completed | Implement macros for route definition (#[route]) | Supports method specification and path parameters |
| NH-2 | Nested Routes | Completed | Support for nested route modules with #[nest] | Working implementation allows route organization by resource |
| NH-3 | Response Types | In Progress | Implement IntoResponse trait | Allow diverse return types from handlers |
| NH-4 | Path Parameters | Completed | Support path parameter extraction | Extract parameters from URL paths |
| NH-5 | WebPlugin | Completed | Create web server plugin | Integrates with App::new() builder |
| NH-6 | Media Uploads | Not Started | Support for multipart form handling | Required for recipe image uploads |
| NH-7 | Rate Limiting | Not Started | Implement rate limiting middleware | Protect Simmr API from abuse |

### navius-db / navius-db-postgres

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| ND-1 | Recipe Repository | Not Started | Create specialized repository for recipes | Include support for ingredients and steps as nested data |
| ND-2 | User Repository | Not Started | Implement user profile storage | Support for profile pictures and preferences |
| ND-3 | Social Graph Storage | Not Started | Design optimal schema for following/followers | Consider performance implications of social graph queries |
| ND-4 | Query Optimization | Not Started | Optimize common social cooking queries | Focus on recipe discovery and filtering |
| ND-5 | Full-Text Search | Not Started | Implement recipe search functionality | Consider PostgreSQL full-text search capabilities |

### navius-auth

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NA-1 | JWT Support | In Progress | Implement JWT generation and validation | Required for user authentication |
| NA-2 | Claims Extraction | In Progress | Auto-extract claims from requests | Allow direct injection of Claims into handlers |
| NA-3 | Authentication Middleware | In Progress | Create authentication middleware | Should validate JWT tokens |
| NA-4 | Social Login | Not Started | Support for OAuth providers | Allow login with Google, Facebook, etc. |
| NA-5 | Permission System | Not Started | Role-based access control | Control access to recipes and social features |

### navius-social (New Crate)

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NS-1 | Following System | Not Started | Implement follower/following relationships | Core social graph functionality |
| NS-2 | Activity Feed | Not Started | Create activity feed generation | Aggregate and personalize user activities |
| NS-3 | Notifications | Not Started | Design notification system | Support multiple notification channels |
| NS-4 | Content Sharing | Not Started | Implement recipe sharing | Allow users to share and repost recipes |
| NS-5 | Comments & Ratings | Not Started | Create comment and rating system | Support for threaded comments and star ratings |

### navius-media (New Crate)

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NM-1 | Image Upload | Not Started | Implement secure image uploading | Support for recipe photos |
| NM-2 | Image Processing | Not Started | Create image optimization pipeline | Generate thumbnails and responsive sizes |
| NM-3 | Storage Integration | Not Started | Integrate with cloud storage | Support for S3 or similar services |
| NM-4 | CDN Support | Not Started | Configure CDN for media delivery | Improve image loading performance |
| NM-5 | Video Support | Not Started | (Future) Support for recipe videos | Allow short cooking demonstrations |

### navius-search (New Crate)

| ID | Issue | Status | Description | Implementation Notes |
|----|-------|--------|-------------|---------------------|
| NSE-1 | Recipe Search | Not Started | Implement recipe search functionality | Support for ingredient and title search |
| NSE-2 | User Search | Not Started | Create user discovery features | Find users by name, interests, etc. |
| NSE-3 | Tag System | Not Started | Implement tagging for recipes | Improve discoverability via tags |
| NSE-4 | Search Suggestions | Not Started | Create search autocomplete | Enhance user experience with smart suggestions |
| NSE-5 | Filter System | Not Started | Build advanced recipe filters | Filter by cuisine, ingredients, time, etc. |

## Enhancement Proposals

### Enhancement Proposal: Social Features Support

**ID**: NC-8  
**Crate**: navius-social (New)  
**Title**: Social Interaction Patterns  
**Status**: Proposed  

**Problem Statement**:  
The Simmr platform requires comprehensive social features, but the current Navius ecosystem lacks standardized components for social interaction patterns like following, activity feeds, and content sharing.

**Use Case**:  
Developers building social platforms need reusable components for common social features without reimplementing these patterns from scratch.

**Proposed Solution**:  
Create a new `navius-social` crate with the following components:

```rust
// Following system
pub struct FollowService<R: FollowRepository> {
    repository: R,
}

impl<R: FollowRepository> FollowService<R> {
    pub async fn follow(&self, follower_id: UserId, followee_id: UserId) -> Result<()>;
    pub async fn unfollow(&self, follower_id: UserId, followee_id: UserId) -> Result<()>;
    pub async fn get_followers(&self, user_id: UserId) -> Result<Vec<User>>;
    pub async fn get_following(&self, user_id: UserId) -> Result<Vec<User>>;
}

// Activity feed
pub struct ActivityFeedService<R: ActivityRepository> {
    repository: R,
}

impl<R: ActivityRepository> ActivityFeedService<R> {
    pub async fn record_activity(&self, activity: Activity) -> Result<()>;
    pub async fn get_feed(&self, user_id: UserId) -> Result<Vec<Activity>>;
}

// Notification system
pub struct NotificationService<R: NotificationRepository, S: NotificationSender> {
    repository: R,
    sender: S,
}

impl<R: NotificationRepository, S: NotificationSender> NotificationService<R, S> {
    pub async fn send_notification(&self, notification: Notification) -> Result<()>;
    pub async fn get_notifications(&self, user_id: UserId) -> Result<Vec<Notification>>;
}
```

**Benefits**:  
- Standardized implementations of common social patterns
- Modular design with pluggable storage backends
- Clear separation of concerns
- Comprehensive testing of social interaction logic

**Backward Compatibility**:  
This is a new crate, so backward compatibility is not a concern.

**Implementation Plan**:  
1. Create the new navius-social crate
2. Implement core social graph functionality
3. Add activity feed generation and aggregation
4. Create notification system with multiple channels
5. Build content sharing mechanisms
6. Provide comprehensive testing and documentation

### Enhancement Proposal: Recipe Data Model

**ID**: ND-1  
**Crate**: navius-db-postgres  
**Title**: Specialized Recipe Repository  
**Status**: Proposed  

**Problem Statement**:  
The Simmr platform needs efficient storage and retrieval of complex recipe data, including ingredients, steps, nutrition information, and media references.

**Use Case**:  
Developers need a specialized repository that handles the unique requirements of recipe data, including efficient querying and relationship management.

**Proposed Solution**:  
Implement a specialized recipe repository with the following features:

```rust
pub struct Recipe {
    id: RecipeId,
    title: String,
    description: String,
    author_id: UserId,
    ingredients: Vec<Ingredient>,
    steps: Vec<CookingStep>,
    media: Vec<MediaReference>,
    tags: Vec<Tag>,
    nutrition: Option<NutritionInfo>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    // Additional fields
}

pub trait RecipeRepository: Send + Sync {
    async fn create(&self, recipe: NewRecipe) -> Result<Recipe>;
    async fn get_by_id(&self, id: RecipeId) -> Result<Option<Recipe>>;
    async fn update(&self, id: RecipeId, updates: RecipeUpdates) -> Result<Recipe>;
    async fn delete(&self, id: RecipeId) -> Result<()>;
    
    // Specialized queries
    async fn find_by_ingredient(&self, ingredient_name: &str) -> Result<Vec<Recipe>>;
    async fn find_by_author(&self, author_id: UserId) -> Result<Vec<Recipe>>;
    async fn find_by_tags(&self, tags: &[Tag]) -> Result<Vec<Recipe>>;
    async fn search(&self, query: &str) -> Result<Vec<Recipe>>;
    
    // Social-related queries
    async fn get_latest_from_following(&self, user_id: UserId) -> Result<Vec<Recipe>>;
    async fn get_popular(&self, limit: usize) -> Result<Vec<Recipe>>;
}

pub struct PostgresRecipeRepository {
    pool: PgPool,
}

impl PostgresRecipeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RecipeRepository for PostgresRecipeRepository {
    // Implementation of all repository methods
}
```

**Benefits**:  
- Specialized handling of recipe-specific data
- Optimized queries for recipe discovery
- Support for complex filtering and search
- Integration with social features

**Backward Compatibility**:  
This extends the existing repository pattern and is compatible with the current architecture.

**Implementation Plan**:  
1. Define the recipe data model
2. Implement the specialized repository trait
3. Create the PostgreSQL implementation
4. Add indexes and optimizations for common queries
5. Integrate with search functionality
6. Provide comprehensive testing and documentation

## Current Status
- Status: In Progress
- Progress: 15%
- Updated at: April 06, 2025 