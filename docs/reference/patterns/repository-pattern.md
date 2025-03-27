---
title: Repository Pattern
description: Implementing the repository pattern for domain entities
category: patterns
tags:
  - patterns
  - repository
  - entity
  - data-access
related:
  - examples/repository-pattern-example.md
  - roadmaps/25-generic-service-implementations.md
last_updated: March 26, 2025
version: 1.0
---

# Repository Pattern

## Overview

The repository pattern provides an abstraction layer between the domain model and data access layers. It centralizes data access logic, making it easier to maintain and test application code.

## Key Benefits

- **Separation of Concerns**: Isolates domain logic from data access code
- **Testability**: Simplifies writing unit tests with mock repositories
- **Flexibility**: Enables switching storage mechanisms without changing business logic
- **Type Safety**: Ensures domain objects are handled correctly across the application
- **Maintainability**: Centralizes data access logic in a consistent pattern

## Implementation in Navius

In the Navius framework, the repository pattern is implemented with several key components:

### Entity Trait

The `Entity` trait defines common properties and behaviors for domain objects:

```rust
pub trait Entity: Clone + Debug + Serialize + Send + Sync + 'static {
    /// The ID type for this entity
    type Id: EntityId;

    /// Get the entity's unique identifier
    fn id(&self) -> &Self::Id;

    /// Get the collection/table name this entity belongs to
    fn collection_name() -> String;

    /// Validates that the entity data is valid
    fn validate(&self) -> Result<(), ServiceError> {
        Ok(())
    }
}
```

### Repository Trait

The `Repository<E>` trait defines standard CRUD operations for entities:

```rust
#[async_trait]
pub trait Repository<E: Entity>: Send + Sync + 'static {
    /// Find an entity by its ID
    async fn find_by_id(&self, id: &E::Id) -> Result<Option<E>, ServiceError>;

    /// Find all entities in the collection
    async fn find_all(&self) -> Result<Vec<E>, ServiceError>;

    /// Save an entity (create or update)
    async fn save(&self, entity: &E) -> Result<E, ServiceError>;

    /// Delete an entity by its ID
    async fn delete(&self, id: &E::Id) -> Result<bool, ServiceError>;

    /// Count entities in the collection
    async fn count(&self) -> Result<usize, ServiceError>;

    /// Check if an entity with the given ID exists
    async fn exists(&self, id: &E::Id) -> Result<bool, ServiceError> {
        Ok(self.find_by_id(id).await?.is_some())
    }
}
```

### Repository Provider

The `RepositoryProvider` trait enables creating repositories for different entity types:

```rust
#[async_trait]
pub trait RepositoryProvider: Send + Sync + 'static {
    /// Create a repository for the given entity type
    async fn create_repository<E: Entity>(
        &self,
        config: RepositoryConfig,
    ) -> Result<Box<dyn Repository<E>>, ServiceError>;

    /// Check if this provider supports the given repository configuration
    fn supports(&self, config: &RepositoryConfig) -> bool;
}
```

### Repository Service

The `RepositoryService` manages repository creation and configuration:

```rust
pub struct RepositoryService {
    providers: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
    configs: Arc<RwLock<HashMap<String, RepositoryConfig>>>,
    default_provider: String,
}
```

## Usage Examples

### Defining an Entity

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub active: bool,
}

impl Entity for User {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn collection_name() -> String {
        "users".to_string()
    }

    fn validate(&self) -> Result<(), ServiceError> {
        // Validation logic...
        Ok(())
    }
}
```

### Creating and Using a Repository

```rust
// Create a repository service
let repo_service = RepositoryService::new();

// Register a repository provider
repo_service.register_provider("memory", InMemoryRepositoryProvider::new()).await?;

// Create a repository for User entities
let config = RepositoryConfig {
    provider: "memory".to_string(),
    ..Default::default()
};
let user_repo = repo_service.create_repository::<User>(config).await?;

// Use the repository
let user = User::new("username", "email@example.com", "Display Name");
let saved_user = user_repo.save(&user).await?;
let found_user = user_repo.find_by_id(&saved_user.id).await?;
```

### Using the Generic Repository

The `GenericRepository<E>` provides a simplified facade for repositories:

```rust
// Create a generic repository
let user_repo = GenericRepository::<User>::with_service(&repo_service).await?;

// Use the generic repository
let user = User::new("username", "email@example.com", "Display Name");
let saved_user = user_repo.save(&user).await?;
```

### Creating Custom Repository Methods

Create a custom repository with specialized query methods:

```rust
pub struct UserRepository {
    inner: Arc<dyn Repository<User>>,
}

impl UserRepository {
    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, ServiceError> {
        let all_users = self.inner.find_all().await?;
        Ok(all_users.into_iter().find(|u| u.username == username))
    }
    
    // Implement other custom methods...
}

// Delegate standard operations to the inner repository
#[async_trait]
impl Repository<User> for UserRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, ServiceError> {
        self.inner.find_by_id(id).await
    }
    
    // Implement other required methods...
}
```

## Best Practices

1. **Entity Validation**: Implement thorough validation in the `validate()` method
2. **Custom Repositories**: Create specialized repositories for complex query needs
3. **Exception Handling**: Use the `ServiceError` for consistent error handling
4. **Type Safety**: Use the proper entity types and ID types throughout
5. **Test Coverage**: Create comprehensive tests for repositories
6. **Immutability**: Treat entities as immutable objects when possible
7. **Transaction Support**: Add transaction support for repository operations when needed

## Related Resources

- [Repository Pattern Example](../../examples/repository-pattern-example.md)
- [Generic Service Implementations Roadmap](../../roadmaps/25-generic-service-implementations.md)

## See Also

- [Entity-Component-System Pattern](https://en.wikipedia.org/wiki/Entity_component_system)
- [Domain-Driven Design](https://en.wikipedia.org/wiki/Domain-driven_design) 