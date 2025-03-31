use navius_core::error::AppError;
use navius_di::{Component, ComponentRegistry};
use std::sync::Arc;

#[cfg(feature = "database")]
use navius_db::PgPool;

#[cfg(feature = "cache")]
use navius_cache::CacheClient;

/// Service registry for managing application services and dependencies
pub struct ServiceRegistry {
    component_registry: ComponentRegistry,
    
    #[cfg(feature = "database")]
    db_pool: Option<Arc<PgPool>>,
    
    #[cfg(feature = "cache")]
    cache_client: Option<Arc<dyn CacheClient>>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            component_registry: ComponentRegistry::new(),
            
            #[cfg(feature = "database")]
            db_pool: None,
            
            #[cfg(feature = "cache")]
            cache_client: None,
        }
    }
    
    /// Register a component
    pub fn register<T: Component + 'static>(&mut self, component: T) -> &mut Self {
        self.component_registry.register(component);
        self
    }
    
    /// Get a component by type
    pub fn get<T: Component + 'static>(&self) -> Option<Arc<T>> {
        self.component_registry.get::<T>()
    }
    
    /// Set the database pool
    #[cfg(feature = "database")]
    pub fn with_db_pool(mut self, pool: Arc<PgPool>) -> Self {
        self.db_pool = Some(pool);
        self
    }
    
    /// Get the database pool
    #[cfg(feature = "database")]
    pub fn db_pool(&self) -> Option<Arc<PgPool>> {
        self.db_pool.clone()
    }
    
    /// Set the cache client
    #[cfg(feature = "cache")]
    pub fn with_cache_client(mut self, client: Arc<dyn CacheClient>) -> Self {
        self.cache_client = Some(client);
        self
    }
    
    /// Get the cache client
    #[cfg(feature = "cache")]
    pub fn cache_client(&self) -> Option<Arc<dyn CacheClient>> {
        self.cache_client.clone()
    }
}

/// Initialize database connection
#[cfg(feature = "database")]
pub async fn init_database(config: &crate::config::DatabaseConfig) -> Result<Arc<PgPool>, AppError> {
    use navius_db_postgres::PostgresConnectionManager;
    
    if !config.enabled {
        return Err(AppError::configuration_error("Database is not enabled in configuration"));
    }
    
    let pool = PostgresConnectionManager::new()
        .with_url(&config.url)
        .with_max_connections(config.max_connections)
        .with_min_connections(config.min_connections)
        .build()
        .await
        .map_err(|e| AppError::database_error(format!("Failed to initialize database pool: {}", e)))?;
    
    Ok(Arc::new(pool))
}

/// Initialize cache connection
#[cfg(feature = "cache")]
pub async fn init_cache(config: &crate::config::CacheConfig) -> Result<Arc<dyn CacheClient>, AppError> {
    use navius_cache_redis::RedisConnectionManager;
    
    if !config.enabled {
        return Err(AppError::configuration_error("Cache is not enabled in configuration"));
    }
    
    let client = RedisConnectionManager::new()
        .with_url(&config.url)
        .with_default_ttl(config.ttl_seconds)
        .build()
        .await
        .map_err(|e| AppError::cache_error(format!("Failed to initialize cache client: {}", e)))?;
    
    Ok(Arc::new(client))
}
