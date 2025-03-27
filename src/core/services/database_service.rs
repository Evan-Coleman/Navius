use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use tracing::{error, info};

use crate::core::services::database_interface::{
    DatabaseConfig, DatabaseOperations, DatabaseProviderRegistry,
};
use crate::core::services::error::ServiceError;
use crate::core::services::memory_database::InMemoryDatabaseProvider;
use crate::core::services::{Lifecycle, Service, ServiceProvider, ServiceRegistry};

/// Generic database service
#[derive(Debug)]
pub struct DatabaseService<DB: DatabaseOperations + Debug> {
    db: Arc<DB>,
}

impl<DB: DatabaseOperations + Debug> Service for DatabaseService<DB> {}

#[async_trait]
impl<DB: DatabaseOperations + Debug> Lifecycle for DatabaseService<DB> {
    async fn init(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Initializing database service");
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Shutting down database service");
        Ok(())
    }

    async fn health_check(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Performing database service health check");
        Ok(())
    }
}

impl<DB: DatabaseOperations + Debug> DatabaseService<DB> {
    /// Create a new database service
    pub fn new(db: DB) -> Self {
        Self { db: Arc::new(db) }
    }

    /// Get a value
    pub async fn get(&self, collection: &str, key: &str) -> Result<Option<String>, ServiceError> {
        self.db.get(collection, key).await
    }

    /// Set a value
    pub async fn set(&self, collection: &str, key: &str, value: &str) -> Result<(), ServiceError> {
        self.db.set(collection, key, value).await
    }

    /// Delete a value
    pub async fn delete(&self, collection: &str, key: &str) -> Result<bool, ServiceError> {
        self.db.delete(collection, key).await
    }

    /// Query values
    pub async fn query(&self, collection: &str, filter: &str) -> Result<Vec<String>, ServiceError> {
        self.db.query(collection, filter).await
    }

    // Convenience methods for specific collections

    /// Get a user by ID
    pub async fn get_user(&self, id: &str) -> Result<Option<String>, ServiceError> {
        self.get("users", id).await
    }

    /// Create a new user
    pub async fn create_user(&self, id: &str, name: &str) -> Result<(), ServiceError> {
        self.set("users", id, name).await
    }

    /// Delete a user
    pub async fn delete_user(&self, id: &str) -> Result<bool, ServiceError> {
        self.delete("users", id).await
    }
}

// We'll implement a DatabaseServiceProvider specifically for InMemoryDatabase
// In a real-world application, we would likely want a more generic solution
pub struct InMemoryDatabaseServiceProvider {
    provider_registry: Arc<DatabaseProviderRegistry>,
}

impl InMemoryDatabaseServiceProvider {
    /// Create a new provider
    pub fn new(provider_registry: Arc<DatabaseProviderRegistry>) -> Self {
        Self { provider_registry }
    }
}

#[async_trait]
impl ServiceProvider for InMemoryDatabaseServiceProvider {
    type Service = DatabaseService<crate::core::services::memory_database::InMemoryDatabase>;
    type Config = DatabaseConfig;
    type Error = ServiceError;

    async fn create(
        config: Self::Config,
        registry: &ServiceRegistry,
    ) -> Result<Self::Service, Self::Error> {
        info!(
            "Creating in-memory database service with config: {:?}",
            config
        );

        // Get the provider registry from the service registry or create a new one
        let provider_registry = match registry.get::<Arc<DatabaseProviderRegistry>>() {
            Some(registry) => registry.clone(),
            None => {
                // Create a new registry if not found
                let mut reg = DatabaseProviderRegistry::new();

                // Register built-in providers
                crate::core::services::memory_database::register_memory_database_provider(&mut reg);

                let reg = Arc::new(reg);

                // Note: In a real implementation, we would register this for future use
                // registry.register::<Arc<DatabaseProviderRegistry>>(reg.clone());

                reg
            }
        };

        // Create the database using the InMemoryDatabaseProvider
        let db = provider_registry
            .create_database::<InMemoryDatabaseProvider>("memory", Arc::new(config))
            .await?;

        // Create the database service
        Ok(DatabaseService::new(db))
    }

    async fn health_check(&self) -> Result<(), Self::Error> {
        info!("In-memory database service health check");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::services::database_interface::DatabaseProvider;
    use crate::core::services::memory_database::InMemoryDatabaseProvider;
    use tokio::test;

    #[test]
    async fn test_database_service() {
        // Create a config
        let config = Arc::new(DatabaseConfig {
            provider: "memory".to_string(),
            url: "memory://test".to_string(),
            ..DatabaseConfig::default()
        });

        // Create a provider
        let provider = InMemoryDatabaseProvider;

        // Create a database
        let db = provider.create_database(config).await.unwrap();

        // Create the database service
        let service = DatabaseService::new(db);

        // Test the service
        service.create_user("1", "Alice").await.unwrap();
        let alice = service.get_user("1").await.unwrap();
        assert_eq!(alice, Some("Alice".to_string()));

        // Test delete
        let deleted = service.delete_user("1").await.unwrap();
        assert!(deleted);

        // Test after deletion
        let alice_after = service.get_user("1").await.unwrap();
        assert_eq!(alice_after, None);
    }

    #[test]
    async fn test_database_service_provider() {
        // Create a config
        let config = DatabaseConfig {
            provider: "memory".to_string(),
            url: "memory://test".to_string(),
            ..DatabaseConfig::default()
        };

        // Create a registry
        let mut registry = ServiceRegistry::new();

        // Create a provider registry
        let mut provider_registry = DatabaseProviderRegistry::new();
        crate::core::services::memory_database::register_memory_database_provider(
            &mut provider_registry,
        );
        let provider_registry = Arc::new(provider_registry);

        // Register the provider registry
        registry.register::<Arc<DatabaseProviderRegistry>>(provider_registry);

        // We don't need to create the provider explicitly since we call the static create method
        // Create the service
        let service = InMemoryDatabaseServiceProvider::create(config, &registry)
            .await
            .unwrap();

        // Test the service
        service.create_user("1", "Alice").await.unwrap();
        let alice = service.get_user("1").await.unwrap();
        assert_eq!(alice, Some("Alice".to_string()));
    }
}
