# Database Integration Architecture

## Overview

This document outlines the approach for database integration in the Navius application, focusing on the separation of concerns between the core and application layers. The architecture follows clean code principles to ensure maintainability, testability, and flexibility.

## Core Principles

1. **Separation of Concerns**: Clear distinction between database abstractions and concrete implementations
2. **Dependency Inversion**: High-level modules depend on abstractions, not concrete implementations
3. **Single Responsibility**: Each component has a specific, focused purpose
4. **Testability**: Architecture designed to facilitate mocking and testing
5. **Error Handling**: Consistent error handling across all database operations

## Architecture Layers

### Core Layer

The core layer contains generic abstractions and interfaces that define the contracts for database operations. These components are not tied to specific entities or business domains.

#### Key Components:

1. **EntityRepository Trait** (`core/database/repository.rs`):
   - Generic trait defining standard CRUD operations
   - Parameterized by entity type and error type
   - Provides consistent interface for database access

```rust
#[async_trait]
pub trait EntityRepository<T, E: Error + Send + Sync>: Send + Sync {
    async fn find_all(&self) -> Result<Vec<T>, E>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<T>, E>;
    async fn create(&self, entity: T) -> Result<T, E>;
    async fn update(&self, entity: T) -> Result<T, E>;
    async fn delete(&self, id: Uuid) -> Result<bool, E>;
}
```

2. **Database Utilities** (`core/database/utils.rs`):
   - Transaction management
   - Common database operations
   - Error handling helpers

```rust
pub async fn with_transaction<F, T, E>(
    pool: &Pool<Postgres>,
    f: F,
) -> Result<T, E>
where
    F: FnOnce(&mut Transaction<'_, Postgres>) -> futures::future::BoxFuture<'_, Result<T, E>> + Send,
    E: From<SqlxError> + Send,
    T: Send
{
    // Transaction handling implementation
}

pub fn db_error_message(operation: &str, entity: &str, error: SqlxError) -> AppError {
    // Error formatting implementation
}
```

3. **PgPool Trait**:
   - Defines interface for database connection pools
   - Enables dependency injection and mock implementations

### Application Layer

The application layer contains concrete implementations that extend the core abstractions for specific business domains and entities.

#### Key Components:

1. **Entity Repository Implementations** (`app/database/repositories/`):
   - Concrete implementations of the EntityRepository trait
   - Specific to business entities (e.g., PetRepository)
   - May include entity-specific query methods

```rust
pub struct PetRepository {
    db_pool: Arc<Box<dyn PgPool>>,
}

#[async_trait]
impl EntityRepository<Pet, AppError> for PetRepository {
    // Implementation of CRUD operations for Pet entity
}

impl PetRepository {
    // Pet-specific query methods
    pub async fn find_by_species(&self, species: &str) -> Result<Vec<Pet>, AppError> {
        // Implementation
    }
}
```

2. **Service Layer** (`app/services/`):
   - Business logic for entities
   - Validation rules and business constraints
   - Coordinates with repositories for data access

```rust
pub struct DefaultPetService {
    pet_repository: Arc<PetRepository>,
}

#[async_trait]
impl PetService for DefaultPetService {
    // Business logic for pet operations
}
```

3. **API Layer** (`app/api/`):
   - REST endpoints for entities
   - Request/response handling
   - Input validation and error mapping

```rust
// Configure pet database routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/petdb", get(get_all_pets))
        .route("/petdb", post(create_pet))
        .route("/petdb/:id", get(get_pet_by_id))
        .route("/petdb/:id", put(update_pet))
        .route("/petdb/:id", delete(delete_pet))
}
```

## Dependency Injection

The architecture uses dependency injection to maintain loose coupling between components:

1. **Repository Injection**:
   - Database connections are injected into repositories
   - Enables testing with mock database connections

2. **Service Injection**:
   - Repositories are injected into services
   - Services can be tested with mock repositories

3. **Service Registry**:
   - Central registry for application services
   - Provides access to services throughout the application

```rust
pub trait ServiceRegistry: Send + Sync {
    fn pet_service(&self) -> &dyn PetService;
    // Other services...
}
```

## Error Handling

The architecture implements a consistent approach to error handling:

1. **Error Types**:
   - Core error types defined in `core/error`
   - Specific error cases for database operations

2. **Error Mapping**:
   - Database errors mapped to appropriate application errors
   - Consistent error message formatting

3. **Error Responses**:
   - HTTP-appropriate status codes
   - User-friendly error messages

## Testing Strategy

The architecture is designed for thorough testing:

1. **Unit Tests**:
   - Mock database connections for repository tests
   - Mock repositories for service tests
   - Test both success and error cases

2. **Integration Tests**:
   - Test actual database interactions
   - Use test database or transactions to isolate tests

3. **API Tests**:
   - Test HTTP endpoints
   - Mock services for controlled testing

## Example: Pet API Implementation

The Pet API demonstrates this architecture in practice:

1. **Core Abstractions**:
   - EntityRepository trait for CRUD operations
   - Database utilities for common operations

2. **App Implementation**:
   - PetRepository implementing EntityRepository
   - PetService for business logic
   - Pet API endpoints

This separation allows:
- Core layer to evolve independently
- Multiple entities to share the same patterns
- Easy testing with mock components
- Clean, focused code

## Conclusion

This database integration architecture:
- Maintains clean separation of concerns
- Provides consistent patterns for database access
- Enables thorough testing
- Supports multiple database entities
- Follows best practices for error handling 