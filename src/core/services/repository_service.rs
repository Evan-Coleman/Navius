use std::collections::HashMap;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::core::models::{
    Entity, Repository, RepositoryConfig, RepositoryProvider, RepositoryProviderRegistry,
};
use crate::core::services::error::ServiceError;
use crate::core::services::memory_repository::{
    InMemoryRepositoryProvider, register_memory_repository_provider,
};
use crate::core::services::{Lifecycle, Service};

/// Service for managing entity repositories
#[derive(Debug)]
pub struct RepositoryService {
    /// Registry of repository providers
    provider_registry: Arc<RwLock<RepositoryProviderRegistry>>,

    /// Repository configurations by entity name
    configs: HashMap<String, RepositoryConfig>,

    /// Default provider to use when none is specified
    default_provider: String,
}

impl Service for RepositoryService {}

#[async_trait]
impl Lifecycle for RepositoryService {
    async fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing repository service");

        // Register default providers if none exist
        let registry_lock = self.provider_registry.read().await;
        let needs_default_providers = registry_lock
            .get::<InMemoryRepositoryProvider>("memory")
            .is_none();
        drop(registry_lock);

        if needs_default_providers {
            info!("Registering default repository providers");
            let mut registry = self.provider_registry.write().await;
            register_memory_repository_provider(&mut registry);
        }

        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Shutting down repository service");
        Ok(())
    }

    async fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

impl RepositoryService {
    /// Create a new repository service
    pub fn new() -> Self {
        Self {
            provider_registry: Arc::new(RwLock::new(RepositoryProviderRegistry::new())),
            configs: HashMap::new(),
            default_provider: "memory".to_string(),
        }
    }

    /// Set the default provider
    pub fn with_default_provider(mut self, provider: &str) -> Self {
        self.default_provider = provider.to_string();
        self
    }

    /// Register a repository provider
    pub async fn register_provider<P: RepositoryProvider + 'static>(
        &self,
        name: &str,
        provider: P,
    ) {
        let mut registry = self.provider_registry.write().await;
        registry.register(name, provider);
    }

    /// Register a repository configuration
    pub fn register_config(&mut self, entity_name: &str, config: RepositoryConfig) {
        self.configs.insert(entity_name.to_string(), config);
    }

    /// Get repository configuration for an entity
    pub fn get_config(&self, entity_name: &str) -> Option<&RepositoryConfig> {
        self.configs.get(entity_name)
    }

    /// Create a repository for an entity type
    pub async fn create_repository<E, P>(
        &self,
        config: Option<RepositoryConfig>,
    ) -> Result<Box<dyn Repository<E>>, ServiceError>
    where
        E: Entity + Serialize + DeserializeOwned,
        P: RepositoryProvider + 'static,
    {
        let entity_name = E::collection_name();
        let config = match config {
            Some(config) => config,
            None => self.configs.get(&entity_name).cloned().unwrap_or_else(|| {
                let mut default_config = RepositoryConfig::default();
                default_config.provider = self.default_provider.clone();
                default_config
            }),
        };

        let registry = self.provider_registry.read().await;
        registry
            .create_repository::<P, E>(&config.provider.clone(), config)
            .await
    }

    /// Create a generic entity repository without knowing the specific provider type
    pub async fn create_typed_repository<E>(&self) -> Result<Box<dyn Repository<E>>, ServiceError>
    where
        E: Entity + Serialize + DeserializeOwned,
    {
        let entity_name = E::collection_name();
        let config = self.configs.get(&entity_name).cloned().unwrap_or_else(|| {
            let mut default_config = RepositoryConfig::default();
            default_config.provider = self.default_provider.clone();
            default_config
        });

        // Here we use pattern matching to determine which provider to create
        // In a real system, we'd use a more dynamic approach with a provider factory
        match config.provider.as_str() {
            "memory" => {
                let registry = self.provider_registry.read().await;
                registry
                    .create_repository::<InMemoryRepositoryProvider, E>("memory", config)
                    .await
            }
            // Add other provider types here as needed
            _ => Err(ServiceError::not_found(format!(
                "Unsupported repository provider: {}",
                config.provider
            ))),
        }
    }
}

impl Default for RepositoryService {
    fn default() -> Self {
        Self::new()
    }
}

/// A generic repository facade that simplifies working with repositories
#[derive(Debug)]
pub struct GenericRepository<E>
where
    E: Entity + Serialize + DeserializeOwned,
{
    /// The actual repository implementation
    repository: Box<dyn Repository<E>>,

    /// Entity type marker
    _entity_type: PhantomData<E>,
}

impl<E> GenericRepository<E>
where
    E: Entity + Serialize + DeserializeOwned,
{
    /// Create a new repository facade
    pub fn new(repository: Box<dyn Repository<E>>) -> Self {
        Self {
            repository,
            _entity_type: PhantomData,
        }
    }

    /// Create a repository with the repository service
    pub async fn with_service(repo_service: &RepositoryService) -> Result<Self, ServiceError> {
        let repository = repo_service.create_typed_repository::<E>().await?;
        Ok(Self::new(repository))
    }

    /// Find an entity by ID
    pub async fn find_by_id(&self, id: &E::Id) -> Result<Option<E>, ServiceError> {
        self.repository.find_by_id(id).await
    }

    /// Find all entities
    pub async fn find_all(&self) -> Result<Vec<E>, ServiceError> {
        self.repository.find_all().await
    }

    /// Save an entity
    pub async fn save(&self, entity: &E) -> Result<E, ServiceError> {
        self.repository.save(entity).await
    }

    /// Delete an entity
    pub async fn delete(&self, id: &E::Id) -> Result<bool, ServiceError> {
        self.repository.delete(id).await
    }

    /// Count entities
    pub async fn count(&self) -> Result<usize, ServiceError> {
        self.repository.count().await
    }

    /// Check if an entity exists
    pub async fn exists(&self, id: &E::Id) -> Result<bool, ServiceError> {
        self.repository.exists(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tokio::test;
    use uuid::Uuid;

    // Example entity for testing that doesn't rely on app models
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestEntity {
        id: Uuid,
        name: String,
        value: i32,
    }

    impl TestEntity {
        fn new(name: &str, value: i32) -> Self {
            Self {
                id: Uuid::new_v4(),
                name: name.to_string(),
                value,
            }
        }
    }

    impl Entity for TestEntity {
        type Id = Uuid;

        fn id(&self) -> &Self::Id {
            &self.id
        }

        fn collection_name() -> String {
            "test_entities".to_string()
        }

        fn validate(&self) -> Result<(), ServiceError> {
            if self.name.is_empty() {
                return Err(ServiceError::validation("Name cannot be empty"));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_repository_service() {
        // Create repository service
        let repo_service = RepositoryService::new();
        repo_service.init().await.unwrap();

        // Create a repository
        let repo = repo_service
            .create_typed_repository::<TestEntity>()
            .await
            .unwrap();

        // Create a test entity
        let entity = TestEntity::new("test", 42);

        // Save the entity
        let saved = repo.save(&entity).await.unwrap();
        assert_eq!(saved.name, "test");
        assert_eq!(saved.value, 42);

        // Find the entity
        let found = repo.find_by_id(entity.id()).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.name, "test");

        // Count entities
        let count = repo.count().await.unwrap();
        assert_eq!(count, 1);

        // Delete the entity
        let deleted = repo.delete(entity.id()).await.unwrap();
        assert!(deleted);
    }

    #[tokio::test]
    async fn test_generic_repository() {
        // Create repository service
        let repo_service = RepositoryService::new();
        repo_service.init().await.unwrap();

        // Create a generic repository
        let repo = GenericRepository::<TestEntity>::with_service(&repo_service)
            .await
            .unwrap();

        // Create a test entity
        let entity = TestEntity::new("generic", 123);

        // Save the entity
        let saved = repo.save(&entity).await.unwrap();
        assert_eq!(saved.name, "generic");
        assert_eq!(saved.value, 123);

        // Find the entity
        let found = repo.find_by_id(entity.id()).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.name, "generic");

        // Count entities
        let count = repo.count().await.unwrap();
        assert_eq!(count, 1);

        // Delete the entity
        let deleted = repo.delete(entity.id()).await.unwrap();
        assert!(deleted);
    }
}
