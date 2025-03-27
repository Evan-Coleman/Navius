use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

use crate::core::services::error::ServiceError;

/// Generic identifier trait for entity IDs
pub trait EntityId: Clone + Debug + Serialize + Send + Sync + 'static {}

// Implement EntityId for common ID types
impl EntityId for Uuid {}
impl EntityId for String {}
impl EntityId for i32 {}
impl EntityId for i64 {}

/// Core entity trait for domain objects
pub trait Entity: Clone + Debug + Serialize + DeserializeOwned + Send + Sync + 'static {
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

/// Generic entity interface for CRUD operations
#[async_trait]
pub trait Repository<E: Entity>: Debug + Send + Sync + 'static {
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

/// Repository provider trait for creating repositories
#[async_trait]
pub trait RepositoryProvider: Send + Sync + 'static {
    /// Create a repository for the given entity type
    async fn create_repository<E>(
        &self,
        config: RepositoryConfig,
    ) -> Result<Box<dyn Repository<E>>, ServiceError>
    where
        E: Entity;

    /// Check if this provider supports the given repository configuration
    fn supports(&self, config: &RepositoryConfig) -> bool;
}

/// Configuration for repository creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Provider name
    pub provider: String,

    /// Database URL
    pub url: Option<String>,

    /// Collection/table name (if not using entity default)
    pub collection_name: Option<String>,

    /// Whether to use optimistic locking
    pub optimistic_locking: bool,

    /// Entity-specific configuration
    pub entity_config: std::collections::HashMap<String, String>,

    /// Provider-specific configuration
    pub provider_config: std::collections::HashMap<String, String>,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            provider: "memory".to_string(),
            url: None,
            collection_name: None,
            optimistic_locking: false,
            entity_config: std::collections::HashMap::new(),
            provider_config: std::collections::HashMap::new(),
        }
    }
}

/// Registry for repository providers
#[derive(Debug)]
pub struct RepositoryProviderRegistry {
    providers: std::collections::HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl RepositoryProviderRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    /// Register a provider
    pub fn register<P: RepositoryProvider>(&mut self, name: &str, provider: P) {
        self.providers.insert(name.to_string(), Box::new(provider));
    }

    /// Get a provider by name with type
    pub fn get<P: RepositoryProvider>(&self, name: &str) -> Option<&P> {
        self.providers.get(name).and_then(|p| p.downcast_ref::<P>())
    }

    /// Create a repository with a specific provider type
    pub async fn create_repository<P: RepositoryProvider, E: Entity>(
        &self,
        provider_name: &str,
        config: RepositoryConfig,
    ) -> Result<Box<dyn Repository<E>>, ServiceError> {
        let provider = self.get::<P>(provider_name).ok_or_else(|| {
            ServiceError::not_found(format!("Repository provider not found: {}", provider_name))
        })?;

        if !provider.supports(&config) {
            return Err(ServiceError::configuration_error(format!(
                "Provider {} does not support the given repository configuration",
                provider_name
            )));
        }

        provider.create_repository::<E>(config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define a test entity for unit tests
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestUser {
        id: Uuid,
        name: String,
        email: String,
    }

    impl Entity for TestUser {
        type Id = Uuid;

        fn id(&self) -> &Self::Id {
            &self.id
        }

        fn collection_name() -> String {
            "users".to_string()
        }

        fn validate(&self) -> Result<(), ServiceError> {
            if self.name.is_empty() {
                return Err(ServiceError::validation("Name cannot be empty"));
            }
            if !self.email.contains('@') {
                return Err(ServiceError::validation("Invalid email format"));
            }
            Ok(())
        }
    }

    #[test]
    fn test_entity_validation() {
        // Valid user
        let user = TestUser {
            id: Uuid::new_v4(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        assert!(user.validate().is_ok());

        // Invalid name
        let user = TestUser {
            id: Uuid::new_v4(),
            name: "".to_string(),
            email: "test@example.com".to_string(),
        };
        assert!(user.validate().is_err());

        // Invalid email
        let user = TestUser {
            id: Uuid::new_v4(),
            name: "Test User".to_string(),
            email: "invalid-email".to_string(),
        };
        assert!(user.validate().is_err());
    }
}
