# Repository Pattern Example

This guide demonstrates how to use the generic repository pattern in Navius to manage domain entities.

## Overview

The repository pattern provides a separation between the domain model layer and the data access layer. It allows you to:

- Work with domain objects instead of raw data
- Switch data sources without changing business logic
- Test business logic without a real data source
- Implement rich query methods beyond simple CRUD

This pattern is implemented in the Navius framework through these components:

1. `Entity` trait - Defines the core interface for domain objects
2. `Repository<E>` trait - Defines CRUD operations for a specific entity type
3. `RepositoryProvider` trait - Creates repositories for different storage types
4. `GenericRepository<E>` - Type-safe repository facade for easy usage

## Basic Example

Here's a simple example of how to use the repository pattern with a User entity:

```rust
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use crate::app::models::user_entity::{User, UserRole};
use crate::core::models::Entity;
use crate::core::services::error::ServiceError;
use crate::core::services::repository_service::{GenericRepository, RepositoryService};

async fn user_repository_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create the repository service
    let mut repo_service = RepositoryService::new();
    repo_service.init().await?;

    // Create a repository for User entities
    let user_repo = GenericRepository::<User>::with_service(&repo_service).await?;

    // Create a new user
    let user = User::new(
        "johndoe".to_string(),
        "john@example.com".to_string(),
        "John Doe".to_string(),
    ).with_role(UserRole::Admin);

    // Save the user to the repository
    let saved_user = user_repo.save(&user).await?;
    println!("User saved with ID: {}", saved_user.id);

    // Find the user by ID
    let found_user = user_repo.find_by_id(saved_user.id()).await?;
    
    if let Some(found_user) = found_user {
        println!("Found user: {}", found_user.display_name);
    }

    // Delete the user
    let deleted = user_repo.delete(saved_user.id()).await?;
    println!("User deleted: {}", deleted);

    Ok(())
}
```

## Creating Custom Entity Types

To create your own entity type, implement the `Entity` trait:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::models::Entity;
use crate::core::services::error::ServiceError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub price: f64,
    pub sku: String,
    pub in_stock: bool,
}

impl Entity for Product {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn collection_name() -> String {
        "products".to_string()
    }

    fn validate(&self) -> Result<(), ServiceError> {
        if self.name.is_empty() {
            return Err(ServiceError::validation("Product name cannot be empty"));
        }
        if self.price <= 0.0 {
            return Err(ServiceError::validation("Product price must be positive"));
        }
        if self.sku.is_empty() {
            return Err(ServiceError::validation("SKU cannot be empty"));
        }
        Ok(())
    }
}

impl Product {
    pub fn new(name: String, price: f64, sku: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            price,
            sku,
            in_stock: true,
        }
    }
}
```

## Using Different Repository Providers

The framework supports different storage providers:

```rust
use crate::core::models::RepositoryConfig;
use crate::core::services::repository_service::RepositoryService;

async fn configure_repository_providers() -> Result<(), Box<dyn std::error::Error>> {
    // Create repository service
    let mut repo_service = RepositoryService::new();
    
    // Configure repository for users with memory storage
    let user_config = RepositoryConfig {
        provider: "memory".to_string(),
        // Other configuration options...
        ..Default::default()
    };
    repo_service.register_config("users", user_config);
    
    // Initialize the service
    repo_service.init().await?;
    
    // Now repositories will use the configured providers
    let user_repo = repo_service.create_typed_repository::<User>().await?;
    
    Ok(())
}
```

## Creating Custom Repository Methods

For specialized query needs beyond basic CRUD, you can create custom repository implementations:

```rust
use crate::core::models::{Entity, Repository};
use crate::core::services::error::ServiceError;
use std::marker::PhantomData;

// Example of a custom user repository with specialized methods
pub struct CustomUserRepository<R: Repository<User>> {
    inner: R,
    _marker: PhantomData<User>,
}

impl<R: Repository<User>> CustomUserRepository<R> {
    pub fn new(repository: R) -> Self {
        Self {
            inner: repository,
            _marker: PhantomData,
        }
    }
    
    // Delegate standard operations to inner repository
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, ServiceError> {
        self.inner.find_by_id(id).await
    }
    
    // Add custom methods
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, ServiceError> {
        // Get all users and filter by email
        let all_users = self.inner.find_all().await?;
        Ok(all_users.into_iter().find(|u| u.email == email))
    }
    
    pub async fn find_by_role(&self, role: UserRole) -> Result<Vec<User>, ServiceError> {
        // Get all users and filter by role
        let all_users = self.inner.find_all().await?;
        Ok(all_users.into_iter().filter(|u| u.role == role).collect())
    }
}
```

## Testing With Mock Repositories

The repository pattern makes testing business logic easy:

```rust
use mockall::predicate::*;
use mockall::mock;

// Generate a mock repository
mock! {
    pub UserRepository {}
    
    #[async_trait]
    impl Repository<User> for UserRepository {
        async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, ServiceError>;
        async fn find_all(&self) -> Result<Vec<User>, ServiceError>;
        async fn save(&self, entity: &User) -> Result<User, ServiceError>;
        async fn delete(&self, id: &Uuid) -> Result<bool, ServiceError>;
        async fn count(&self) -> Result<usize, ServiceError>;
        async fn exists(&self, id: &Uuid) -> Result<bool, ServiceError>;
    }
}

#[tokio::test]
async fn test_user_service() {
    // Create a mock repository
    let mut mock_repo = MockUserRepository::new();
    
    // Set expectations
    let test_user = User::new(
        "testuser".to_string(),
        "test@example.com".to_string(), 
        "Test User".to_string()
    );
    
    mock_repo.expect_find_by_id()
        .with(eq(test_user.id))
        .returning(move |_| Ok(Some(test_user.clone())));
    
    // Create the service with the mock repository
    let user_service = UserService::new(GenericRepository::new(Box::new(mock_repo)));
    
    // Test service methods
    let result = user_service.find_by_id(*test_user.id()).await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().username, "testuser");
}
```

## Benefits of the Repository Pattern

1. **Abstraction**: Domain logic doesn't need to know about data storage details
2. **Testability**: Easy to test with mock repositories
3. **Flexibility**: Switch storage implementations without changing business logic
4. **Consistency**: Standard interface for all entity types
5. **Type Safety**: Generic repositories provide type-safe operations
6. **Domain-Driven**: Focus on domain objects rather than data structures
7. **Performance**: Repositories can implement caching or optimizations

## Best Practices

1. Keep entity validation in the `validate()` method
2. Use the repository service for configuration and creation
3. Use specialized repository implementations for complex queries
4. Always use transactions for operations that modify multiple entities
5. Consider using a facade for related repositories when dealing with aggregates
6. Add proper error handling in repository implementations
7. Use the generic repository for simple cases, custom repositories for complex ones 