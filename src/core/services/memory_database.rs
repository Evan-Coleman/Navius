use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::core::services::database_interface::{
    DatabaseConfig, DatabaseOperations, DatabaseProvider, DatabaseProviderRegistry,
};
use crate::core::services::error::ServiceError;
use crate::core::services::{Lifecycle, Service};

/// In-memory database for demonstration purposes
#[derive(Debug, Clone)]
pub struct InMemoryDatabase {
    /// Configuration
    config: Arc<DatabaseConfig>,

    /// Data store
    data: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,

    /// Whether the database is initialized
    initialized: bool,
}

impl Service for InMemoryDatabase {}

#[async_trait]
impl Lifecycle for InMemoryDatabase {
    async fn init(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Initializing in-memory database");
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Shutting down in-memory database");
        Ok(())
    }

    async fn health_check(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.initialized {
            Ok(())
        } else {
            Err(Box::new(ServiceError::unavailable(
                "Database not initialized",
            )))
        }
    }
}

impl InMemoryDatabase {
    /// Create a new in-memory database
    pub fn new(config: Arc<DatabaseConfig>) -> Self {
        Self {
            config,
            data: Arc::new(Mutex::new(HashMap::new())),
            initialized: true,
        }
    }
}

#[async_trait]
impl DatabaseOperations for InMemoryDatabase {
    /// Get data from the database
    async fn get(&self, collection: &str, key: &str) -> Result<Option<String>, ServiceError> {
        let data = self.data.lock().await;

        match data.get(collection) {
            Some(collection_data) => Ok(collection_data.get(key).cloned()),
            None => Ok(None),
        }
    }

    /// Set data in the database
    async fn set(&self, collection: &str, key: &str, value: &str) -> Result<(), ServiceError> {
        let mut data = self.data.lock().await;

        let collection_data = data
            .entry(collection.to_string())
            .or_insert_with(HashMap::new);
        collection_data.insert(key.to_string(), value.to_string());

        Ok(())
    }

    /// Delete data from the database
    async fn delete(&self, collection: &str, key: &str) -> Result<bool, ServiceError> {
        let mut data = self.data.lock().await;

        match data.get_mut(collection) {
            Some(collection_data) => Ok(collection_data.remove(key).is_some()),
            None => Ok(false),
        }
    }

    /// Query the database with a filter
    async fn query(&self, collection: &str, filter: &str) -> Result<Vec<String>, ServiceError> {
        let data = self.data.lock().await;

        match data.get(collection) {
            Some(collection_data) => {
                // Simple contains filter for demonstration
                let results = collection_data
                    .iter()
                    .filter(|(_, v)| v.contains(filter))
                    .map(|(_, v)| v.clone())
                    .collect();

                Ok(results)
            }
            None => Ok(Vec::new()),
        }
    }
}

/// In-memory database provider
#[derive(Debug, Clone)]
pub struct InMemoryDatabaseProvider;

#[async_trait]
impl DatabaseProvider for InMemoryDatabaseProvider {
    type Database = InMemoryDatabase;

    async fn create_database(
        &self,
        config: Arc<DatabaseConfig>,
    ) -> Result<Self::Database, ServiceError> {
        let db = InMemoryDatabase::new(config);

        // Initialize the database
        if let Err(e) = db.init().await {
            return Err(ServiceError::initialization_error(format!(
                "Database initialization failed: {}",
                e
            )));
        }

        Ok(db)
    }

    fn supports(&self, config: &DatabaseConfig) -> bool {
        config.url.starts_with("memory://")
    }
}

/// Register in-memory database provider
pub fn register_memory_database_provider(registry: &mut DatabaseProviderRegistry) {
    registry.register("memory", InMemoryDatabaseProvider);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_in_memory_database() {
        // Create a config
        let config = Arc::new(DatabaseConfig::default());

        // Create a new database
        let db = InMemoryDatabase::new(config);

        // Set some data
        db.set("users", "1", "Alice").await.unwrap();
        db.set("users", "2", "Bob").await.unwrap();

        // Get data
        let alice = db.get("users", "1").await.unwrap();
        assert_eq!(alice, Some("Alice".to_string()));

        // Delete data
        let deleted = db.delete("users", "1").await.unwrap();
        assert!(deleted);

        // Verify deletion
        let alice_after = db.get("users", "1").await.unwrap();
        assert_eq!(alice_after, None);

        // Test query
        let bob = db.query("users", "Bob").await.unwrap();
        assert_eq!(bob, vec!["Bob".to_string()]);
    }

    #[test]
    async fn test_provider() {
        // Create a config
        let config = Arc::new(DatabaseConfig {
            provider: "memory".to_string(),
            url: "memory://test".to_string(),
            ..DatabaseConfig::default()
        });

        // Create a provider
        let provider = InMemoryDatabaseProvider;

        // Check if provider supports config
        assert!(provider.supports(&config));

        // Create a database
        let db = provider.create_database(config).await.unwrap();

        // Test the database
        db.set("users", "1", "Alice").await.unwrap();
        let alice = db.get("users", "1").await.unwrap();
        assert_eq!(alice, Some("Alice".to_string()));
    }
}
