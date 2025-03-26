use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::core::services::error::ServiceError;
use crate::core::services::{Lifecycle, Service, ServiceProvider, ServiceRegistry};

/// Configuration for database connections
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,

    /// Maximum connection pool size
    pub max_connections: u32,

    /// Connection timeout in seconds
    pub timeout_seconds: u32,

    /// Whether to use SSL for connections
    pub use_ssl: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "memory://".to_string(),
            max_connections: 10,
            timeout_seconds: 30,
            use_ssl: false,
        }
    }
}

/// In-memory database for demonstration purposes
#[derive(Debug, Clone)]
pub struct InMemoryDatabase {
    /// Configuration
    config: DatabaseConfig,

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
            Err(Box::new(ServiceError::Unavailable(
                "Database not initialized".to_string(),
            )))
        }
    }
}

impl InMemoryDatabase {
    /// Create a new in-memory database
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            data: Arc::new(Mutex::new(HashMap::new())),
            initialized: true,
        }
    }

    /// Get data from the database
    pub async fn get(&self, collection: &str, key: &str) -> Result<Option<String>, ServiceError> {
        let data = self.data.lock().await;

        match data.get(collection) {
            Some(collection_data) => Ok(collection_data.get(key).cloned()),
            None => Ok(None),
        }
    }

    /// Set data in the database
    pub async fn set(&self, collection: &str, key: &str, value: &str) -> Result<(), ServiceError> {
        let mut data = self.data.lock().await;

        let collection_data = data
            .entry(collection.to_string())
            .or_insert_with(HashMap::new);
        collection_data.insert(key.to_string(), value.to_string());

        Ok(())
    }

    /// Delete data from the database
    pub async fn delete(&self, collection: &str, key: &str) -> Result<bool, ServiceError> {
        let mut data = self.data.lock().await;

        match data.get_mut(collection) {
            Some(collection_data) => Ok(collection_data.remove(key).is_some()),
            None => Ok(false),
        }
    }
}

/// Database provider implementation
pub struct DatabaseProvider;

#[async_trait]
impl ServiceProvider for DatabaseProvider {
    type Service = InMemoryDatabase;
    type Config = DatabaseConfig;
    type Error = ServiceError;

    async fn create(
        config: Self::Config,
        _registry: &ServiceRegistry,
    ) -> Result<Self::Service, Self::Error> {
        info!("Creating database with config: {:?}", config);

        // In a real implementation, this would initialize a database connection pool
        let database = InMemoryDatabase::new(config);

        // Initialize the database
        if let Err(e) = database.init().await {
            error!("Failed to initialize database: {}", e);
            return Err(ServiceError::InitializationError(format!(
                "Database initialization failed: {}",
                e
            )));
        }

        Ok(database)
    }

    async fn health_check(&self) -> Result<(), Self::Error> {
        info!("Database health check");
        Ok(())
    }
}

/// A higher-level database service that uses the in-memory database
#[derive(Debug)]
pub struct DatabaseService {
    db: Arc<InMemoryDatabase>,
}

impl Service for DatabaseService {}

#[async_trait]
impl Lifecycle for DatabaseService {
    async fn init(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Initializing database service");
        self.db.init().await
    }

    async fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Shutting down database service");
        self.db.shutdown().await
    }

    async fn health_check(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Performing database service health check");
        self.db.health_check().await
    }
}

impl DatabaseService {
    /// Create a new database service
    pub fn new(db: Arc<InMemoryDatabase>) -> Self {
        Self { db }
    }

    /// Get a user by ID
    pub async fn get_user(&self, id: &str) -> Result<Option<String>, ServiceError> {
        self.db.get("users", id).await
    }

    /// Create a new user
    pub async fn create_user(&self, id: &str, name: &str) -> Result<(), ServiceError> {
        self.db.set("users", id, name).await
    }

    /// Delete a user
    pub async fn delete_user(&self, id: &str) -> Result<bool, ServiceError> {
        self.db.delete("users", id).await
    }
}

/// Database service provider implementation
pub struct DatabaseServiceProvider;

#[async_trait]
impl ServiceProvider for DatabaseServiceProvider {
    type Service = DatabaseService;
    type Config = DatabaseConfig;
    type Error = ServiceError;

    async fn create(
        config: Self::Config,
        registry: &ServiceRegistry,
    ) -> Result<Self::Service, Self::Error> {
        info!("Creating database service with config: {:?}", config);

        // Look for an existing database instance in the registry
        if let Some(db) = registry.get::<InMemoryDatabase>() {
            // Use the existing database
            info!("Using existing database instance");
            return Ok(DatabaseService::new(Arc::new(db.clone())));
        }

        // Create a new database instance
        let db = DatabaseProvider::create(config, registry).await?;

        // Create the database service
        Ok(DatabaseService::new(Arc::new(db)))
    }

    async fn health_check(&self) -> Result<(), Self::Error> {
        info!("Database service health check");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_in_memory_database() {
        // Create a new database
        let db = InMemoryDatabase::new(DatabaseConfig::default());

        // Set some data
        db.set("users", "1", "Alice").await.unwrap();
        db.set("users", "2", "Bob").await.unwrap();

        // Get data
        let alice = db.get("users", "1").await.unwrap();
        assert_eq!(alice, Some("Alice".to_string()));

        let bob = db.get("users", "2").await.unwrap();
        assert_eq!(bob, Some("Bob".to_string()));

        // Delete data
        let deleted = db.delete("users", "1").await.unwrap();
        assert!(deleted);

        // Verify deletion
        let alice = db.get("users", "1").await.unwrap();
        assert_eq!(alice, None);
    }

    #[test]
    async fn test_database_service() {
        // Create a service registry
        let registry = ServiceRegistry::new();

        // Create a database service
        let db_service = DatabaseServiceProvider::create(DatabaseConfig::default(), &registry)
            .await
            .unwrap();

        // Create users
        db_service.create_user("1", "Alice").await.unwrap();
        db_service.create_user("2", "Bob").await.unwrap();

        // Get users
        let alice = db_service.get_user("1").await.unwrap();
        assert_eq!(alice, Some("Alice".to_string()));

        let bob = db_service.get_user("2").await.unwrap();
        assert_eq!(bob, Some("Bob".to_string()));

        // Delete user
        let deleted = db_service.delete_user("1").await.unwrap();
        assert!(deleted);

        // Verify deletion
        let alice = db_service.get_user("1").await.unwrap();
        assert_eq!(alice, None);
    }

    #[test]
    async fn test_database_service_provider() {
        // Create a service registry
        let mut registry = ServiceRegistry::new();

        // Create a database
        let db = InMemoryDatabase::new(DatabaseConfig::default());

        // Register the database in the registry
        registry.register(db);

        // Create a database service using the registered database
        let db_service = DatabaseServiceProvider::create(DatabaseConfig::default(), &registry)
            .await
            .unwrap();

        // Create a user
        db_service.create_user("1", "Alice").await.unwrap();

        // Get the user
        let alice = db_service.get_user("1").await.unwrap();
        assert_eq!(alice, Some("Alice".to_string()));
    }
}
