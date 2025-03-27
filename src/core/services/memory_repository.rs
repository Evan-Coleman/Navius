use std::collections::HashMap;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

use crate::core::models::Entity;
use crate::core::models::Repository;
use crate::core::models::RepositoryConfig;
use crate::core::models::RepositoryProvider;
use crate::core::models::RepositoryProviderRegistry;
use crate::core::services::error::ServiceError;
use crate::core::services::{Lifecycle, Service};

/// Memory-based repository implementation for entities
pub struct InMemoryRepository<E: Entity + Serialize + DeserializeOwned> {
    /// Repository configuration
    config: RepositoryConfig,

    /// Collection name
    collection_name: String,

    /// Data store for all collections
    data_store: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,

    /// Entity type marker
    _entity_type: PhantomData<E>,
}

impl<E: Entity + Serialize + DeserializeOwned> Service for InMemoryRepository<E> {}

#[async_trait]
impl<E: Entity + Serialize + DeserializeOwned> Lifecycle for InMemoryRepository<E> {
    async fn init(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!(
            "Initializing in-memory repository for {}",
            self.collection_name
        );
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!(
            "Shutting down in-memory repository for {}",
            self.collection_name
        );
        Ok(())
    }

    async fn health_check(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}

impl<E: Entity + Serialize + DeserializeOwned> InMemoryRepository<E> {
    /// Create a new in-memory repository
    pub fn new(
        config: RepositoryConfig,
        data_store: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
    ) -> Self {
        let collection_name = config
            .collection_name
            .clone()
            .unwrap_or_else(|| E::collection_name());
        Self {
            config,
            collection_name,
            data_store,
            _entity_type: PhantomData,
        }
    }

    /// Serialize an entity to JSON
    fn serialize_entity(&self, entity: &E) -> Result<String, ServiceError> {
        serde_json::to_string(entity).map_err(|e| {
            ServiceError::conversion_error(format!("Failed to serialize entity: {}", e))
        })
    }

    /// Deserialize an entity from JSON
    fn deserialize_entity(&self, json: &str) -> Result<E, ServiceError> {
        serde_json::from_str(json).map_err(|e| {
            ServiceError::conversion_error(format!("Failed to deserialize entity: {}", e))
        })
    }

    /// Convert entity ID to string
    fn id_to_string(&self, id: &E::Id) -> String {
        serde_json::to_string(id)
            .unwrap_or_else(|_| format!("{:?}", id))
            .trim_matches('"')
            .to_string()
    }
}

#[async_trait]
impl<E: Entity + Serialize + DeserializeOwned> Repository<E> for InMemoryRepository<E> {
    async fn find_by_id(&self, id: &E::Id) -> Result<Option<E>, ServiceError> {
        let data = self.data_store.lock().await;
        let id_str = self.id_to_string(id);

        match data.get(&self.collection_name) {
            Some(collection) => match collection.get(&id_str) {
                Some(json) => {
                    let entity = self.deserialize_entity(json)?;
                    Ok(Some(entity))
                }
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> Result<Vec<E>, ServiceError> {
        let data = self.data_store.lock().await;

        match data.get(&self.collection_name) {
            Some(collection) => {
                let mut entities = Vec::new();
                for json in collection.values() {
                    let entity = self.deserialize_entity(json)?;
                    entities.push(entity);
                }
                Ok(entities)
            }
            None => Ok(Vec::new()),
        }
    }

    async fn save(&self, entity: &E) -> Result<E, ServiceError> {
        // Validate the entity first
        entity.validate()?;

        let json = self.serialize_entity(entity)?;
        let id_str = self.id_to_string(entity.id());

        let mut data = self.data_store.lock().await;

        // Ensure collection exists
        let collection = data
            .entry(self.collection_name.clone())
            .or_insert_with(HashMap::new);

        // Save entity
        collection.insert(id_str, json);

        // Return a clone of the saved entity
        Ok(entity.clone())
    }

    async fn delete(&self, id: &E::Id) -> Result<bool, ServiceError> {
        let mut data = self.data_store.lock().await;
        let id_str = self.id_to_string(id);

        match data.get_mut(&self.collection_name) {
            Some(collection) => Ok(collection.remove(&id_str).is_some()),
            None => Ok(false),
        }
    }

    async fn count(&self) -> Result<usize, ServiceError> {
        let data = self.data_store.lock().await;

        match data.get(&self.collection_name) {
            Some(collection) => Ok(collection.len()),
            None => Ok(0),
        }
    }
}

/// In-memory repository provider
#[derive(Clone)]
pub struct InMemoryRepositoryProvider {
    /// Shared data store for all repositories
    data_store: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
}

impl InMemoryRepositoryProvider {
    /// Create a new in-memory repository provider
    pub fn new() -> Self {
        Self {
            data_store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryRepositoryProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RepositoryProvider for InMemoryRepositoryProvider {
    async fn create_repository<E: Entity + Serialize + DeserializeOwned>(
        &self,
        config: RepositoryConfig,
    ) -> Result<Box<dyn Repository<E>>, ServiceError> {
        let repository = InMemoryRepository::<E>::new(config, self.data_store.clone());
        repository.init().await.map_err(|e| {
            ServiceError::initialization_error(format!("Failed to initialize repository: {}", e))
        })?;
        Ok(Box::new(repository))
    }

    fn supports(&self, config: &RepositoryConfig) -> bool {
        config.provider == "memory"
            || config
                .url
                .as_ref()
                .map_or(false, |url| url.starts_with("memory://"))
    }
}

/// Register in-memory repository provider
pub fn register_memory_repository_provider(registry: &mut RepositoryProviderRegistry) {
    registry.register("memory", InMemoryRepositoryProvider::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tokio::test;

    // Define a test entity
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
            "test_users".to_string()
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
    async fn test_in_memory_repository() {
        // Create a repository
        let data_store = Arc::new(Mutex::new(HashMap::new()));
        let config = RepositoryConfig::default();
        let repository = InMemoryRepository::<TestUser>::new(config, data_store);

        // Create a test user
        let user_id = Uuid::new_v4();
        let user = TestUser {
            id: user_id,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        // Save the user
        let saved_user = repository.save(&user).await.unwrap();
        assert_eq!(saved_user.id, user.id);

        // Find by ID
        let found_user = repository.find_by_id(&user_id).await.unwrap();
        assert!(found_user.is_some());
        let found_user = found_user.unwrap();
        assert_eq!(found_user.id, user.id);
        assert_eq!(found_user.name, user.name);
        assert_eq!(found_user.email, user.email);

        // Find all
        let all_users = repository.find_all().await.unwrap();
        assert_eq!(all_users.len(), 1);
        assert_eq!(all_users[0].id, user.id);

        // Count
        let count = repository.count().await.unwrap();
        assert_eq!(count, 1);

        // Delete
        let deleted = repository.delete(&user_id).await.unwrap();
        assert!(deleted);

        // Verify deletion
        let found_after_delete = repository.find_by_id(&user_id).await.unwrap();
        assert!(found_after_delete.is_none());

        // Count after delete
        let count_after_delete = repository.count().await.unwrap();
        assert_eq!(count_after_delete, 0);
    }

    #[test]
    async fn test_repository_validation() {
        // Create a repository
        let data_store = Arc::new(Mutex::new(HashMap::new()));
        let config = RepositoryConfig::default();
        let repository = InMemoryRepository::<TestUser>::new(config, data_store);

        // Create an invalid user (empty name)
        let user_id = Uuid::new_v4();
        let invalid_user = TestUser {
            id: user_id,
            name: "".to_string(),
            email: "test@example.com".to_string(),
        };

        // Try to save - should fail validation
        let result = repository.save(&invalid_user).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ServiceError::Validation(_)));

        // Create an invalid user (invalid email)
        let invalid_user = TestUser {
            id: user_id,
            name: "Test User".to_string(),
            email: "invalid-email".to_string(),
        };

        // Try to save - should fail validation
        let result = repository.save(&invalid_user).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ServiceError::Validation(_)));
    }

    #[test]
    async fn test_repository_provider() {
        // Create provider
        let provider = InMemoryRepositoryProvider::new();

        // Test supports method
        let config = RepositoryConfig {
            provider: "memory".to_string(),
            ..Default::default()
        };
        assert!(provider.supports(&config));

        let config = RepositoryConfig {
            provider: "other".to_string(),
            url: Some("memory://test".to_string()),
            ..Default::default()
        };
        assert!(provider.supports(&config));

        let config = RepositoryConfig {
            provider: "other".to_string(),
            url: Some("postgres://localhost".to_string()),
            ..Default::default()
        };
        assert!(!provider.supports(&config));

        // Create repository through provider
        let config = RepositoryConfig::default();
        let repository = provider
            .create_repository::<TestUser>(config)
            .await
            .unwrap();

        // Test the repository
        let user_id = Uuid::new_v4();
        let user = TestUser {
            id: user_id,
            name: "Provider Test".to_string(),
            email: "provider@example.com".to_string(),
        };

        let saved_user = repository.save(&user).await.unwrap();
        assert_eq!(saved_user.id, user.id);

        let found_user = repository.find_by_id(&user_id).await.unwrap();
        assert!(found_user.is_some());
        assert_eq!(found_user.unwrap().name, "Provider Test");
    }
}
