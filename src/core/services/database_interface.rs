use crate::core::services::error::ServiceError;
use async_trait::async_trait;
use std::sync::Arc;

/// Trait defining database operations
#[async_trait]
pub trait DatabaseOperations: Send + Sync + 'static {
    /// Get a value from the database
    async fn get(&self, collection: &str, key: &str) -> Result<Option<String>, ServiceError>;

    /// Set a value in the database
    async fn set(&self, collection: &str, key: &str, value: &str) -> Result<(), ServiceError>;

    /// Delete a value from the database
    async fn delete(&self, collection: &str, key: &str) -> Result<bool, ServiceError>;

    /// Query the database with a filter
    async fn query(&self, collection: &str, filter: &str) -> Result<Vec<String>, ServiceError>;
}

/// Trait for database providers
#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    /// The type of database this provider creates
    type Database: DatabaseOperations;

    /// Create a new database instance
    async fn create_database(
        &self,
        config: Arc<DatabaseConfig>,
    ) -> Result<Self::Database, ServiceError>;

    /// Check if this provider supports the given configuration
    fn supports(&self, config: &DatabaseConfig) -> bool;
}

/// Configuration for database connections
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Provider name
    pub provider: String,

    /// Database URL
    pub url: String,

    /// Maximum connection pool size
    pub max_connections: u32,

    /// Connection timeout in seconds
    pub timeout_seconds: u32,

    /// Whether to use SSL for connections
    pub use_ssl: bool,

    /// Provider-specific configuration
    pub provider_config: std::collections::HashMap<String, String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            provider: "memory".to_string(),
            url: "memory://".to_string(),
            max_connections: 10,
            timeout_seconds: 30,
            use_ssl: false,
            provider_config: std::collections::HashMap::new(),
        }
    }
}

/// Registry for database providers
pub struct DatabaseProviderRegistry {
    providers: std::collections::HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl DatabaseProviderRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    /// Register a provider
    pub fn register<P: DatabaseProvider + 'static>(&mut self, name: &str, provider: P) {
        self.providers.insert(name.to_string(), Box::new(provider));
    }

    /// Get a provider by name with type
    pub fn get<P: DatabaseProvider + 'static>(&self, name: &str) -> Option<&P> {
        self.providers.get(name).and_then(|p| p.downcast_ref::<P>())
    }

    /// Create a database with a specific provider type
    pub async fn create_database<P: DatabaseProvider + 'static>(
        &self,
        provider_name: &str,
        config: Arc<DatabaseConfig>,
    ) -> Result<P::Database, ServiceError> {
        let provider = self.get::<P>(provider_name).ok_or_else(|| {
            ServiceError::not_found(format!("Provider not found: {}", provider_name))
        })?;

        if !provider.supports(&config) {
            return Err(ServiceError::configuration_error(format!(
                "Provider {} does not support the given configuration",
                provider_name
            )));
        }

        provider.create_database(config).await
    }
}
