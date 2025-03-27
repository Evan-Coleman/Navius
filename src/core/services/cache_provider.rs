use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

use async_trait::async_trait;
use bincode::{Decode, Encode};
use serde::{Serialize, de::DeserializeOwned};

use crate::core::services::error::ServiceError;

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of items in the cache
    pub size: usize,
    /// Hit count
    pub hits: u64,
    /// Miss count
    pub misses: u64,
    /// Eviction count
    pub evictions: u64,
    /// Total capacity
    pub capacity: Option<usize>,
    /// Provider-specific metrics
    pub custom_metrics: HashMap<String, String>,
}

/// Eviction policy for cache
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// First In First Out
    FIFO,
    /// Least Frequently Used
    LFU,
    /// Time To Live based eviction
    TTL,
    /// Random eviction
    Random,
    /// No eviction (error on full)
    None,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Cache name
    pub name: String,
    /// Provider type
    pub provider: String,
    /// Max capacity (items)
    pub capacity: Option<usize>,
    /// Default TTL
    pub default_ttl: Option<Duration>,
    /// Eviction policy
    pub eviction_policy: EvictionPolicy,
    /// Provider-specific configuration
    pub provider_config: HashMap<String, String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            provider: "memory".to_string(),
            capacity: Some(1000),
            default_ttl: Some(Duration::from_secs(3600)),
            eviction_policy: EvictionPolicy::LRU,
            provider_config: HashMap::new(),
        }
    }
}

/// Cache error types for operation failures
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Capacity error (cache full)
    #[error("Cache capacity reached: {0}")]
    Capacity(String),

    /// Operation error
    #[error("Operation error: {0}")]
    Operation(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Type error
    #[error("Type error: {0}")]
    Type(String),

    /// Key error
    #[error("Key error: {0}")]
    Key(String),

    /// Other error
    #[error("Cache error: {0}")]
    Other(String),
}

impl From<CacheError> for ServiceError {
    fn from(error: CacheError) -> Self {
        match error {
            CacheError::Configuration(msg) => ServiceError::configuration_error(msg),
            CacheError::Connection(msg) => ServiceError::unavailable(msg),
            _ => ServiceError::other(error.to_string()),
        }
    }
}

/// TypedCache trait for type-specific cache operations
#[async_trait]
pub trait TypedCache<T>: Send + Sync + 'static
where
    T: Encode + Decode<()> + Send + Sync + 'static,
{
    /// Get a value from the cache
    async fn get(&self, key: &str) -> Result<Option<T>, CacheError>;

    /// Set a value in the cache with optional TTL
    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), CacheError>;

    /// Set multiple values in the cache
    async fn set_many(
        &self,
        items: HashMap<String, T>,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError>;

    /// Get multiple values from the cache
    async fn get_many(&self, keys: &[&str]) -> Result<HashMap<String, Option<T>>, CacheError>;
}

/// Helper trait to create TypedCache instances for a specific type
/// This trait is parameterized to avoid object safety issues
pub trait TypedCacheFactory<T>: Send + Sync + 'static
where
    T: Encode + Decode<()> + Send + Sync + 'static,
{
    /// Get a typed cache for the specified type
    fn create_typed_cache(&self) -> Box<dyn TypedCache<T>>;
}

/// Object-safe factory for getting TypedCacheFactory instances
pub trait CacheFactory: Send + Sync + 'static {
    /// Create a factory for a specific type
    fn for_type<T>(&self) -> Box<dyn TypedCacheFactory<T>>
    where
        T: Encode + Decode<()> + Send + Sync + 'static;
}

/// Base cache operations that are object-safe (no generic methods)
#[async_trait]
pub trait DynCacheOperations: Send + Sync + 'static {
    /// Delete a value from the cache
    async fn delete(&self, key: &str) -> Result<bool, CacheError>;

    /// Clear the entire cache
    async fn clear(&self) -> Result<(), CacheError>;

    /// Check if a key exists in the cache
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// Delete multiple values from the cache
    async fn delete_many(&self, keys: &[&str]) -> Result<usize, CacheError>;

    /// Increment a counter
    async fn increment(&self, key: &str, delta: i64) -> Result<i64, CacheError>;

    /// Get cache statistics
    fn stats(&self) -> Result<CacheStats, CacheError>;

    /// Get the cache name
    fn name(&self) -> &str;

    /// Get the cache configuration
    fn config(&self) -> &CacheConfig;

    /// Cast to Any for dynamic type conversion
    fn as_any(&self) -> &dyn Any;
}

/// Complete cache operations trait - combines DynCacheOperations and CacheFactory
/// Not object-safe due to generic methods from CacheFactory
pub trait CacheOperations: DynCacheOperations + CacheFactory {}

// Implement CacheOperations for anything that implements both required traits
impl<T> CacheOperations for T where T: DynCacheOperations + CacheFactory {}

/// Cache provider trait - for creating and managing cache instances
#[async_trait]
pub trait CacheProvider: Send + Sync + 'static {
    /// Create a new cache with the given configuration
    async fn create_cache(
        &self,
        config: CacheConfig,
    ) -> Result<Box<dyn DynCacheOperations>, CacheError>;

    /// Check if this provider supports the given configuration
    fn supports(&self, config: &CacheConfig) -> bool;

    /// Get the provider name
    fn name(&self) -> &str;

    /// Get provider capabilities
    fn capabilities(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Cast to Any for dynamic type conversion
    fn as_any(&self) -> &dyn Any;
}

/// Cache provider registry - for registering and retrieving cache providers
#[derive(Default)]
pub struct CacheProviderRegistry {
    providers: HashMap<String, Box<dyn CacheProvider>>,
}

impl CacheProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a cache provider
    pub fn register<P>(&mut self, provider: P)
    where
        P: CacheProvider + 'static,
    {
        let name = provider.name().to_string();
        self.providers.insert(name, Box::new(provider));
    }

    /// Get a provider by name
    pub fn get(&self, name: &str) -> Option<&dyn CacheProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// Get all provider names
    pub fn provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Create a cache with the given configuration
    pub async fn create_cache(
        &self,
        config: CacheConfig,
    ) -> Result<Box<dyn DynCacheOperations>, CacheError> {
        // Find a provider that supports this configuration
        if let Some(provider) = self.get(&config.provider) {
            if provider.supports(&config) {
                provider.create_cache(config).await
            } else {
                Err(CacheError::Configuration(format!(
                    "Provider '{}' does not support this configuration",
                    config.provider
                )))
            }
        } else {
            // Find any provider that supports this configuration
            for (_name, provider) in &self.providers {
                if provider.supports(&config) {
                    return provider.create_cache(config).await;
                }
            }

            Err(CacheError::Configuration(format!(
                "No provider found for cache configuration with provider '{}'",
                config.provider
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock cache for testing
    struct MockCache {
        name: String,
        config: CacheConfig,
    }

    // Mock typed cache
    struct MockTypedCache<T> {
        cache: Arc<MockCache>,
        _marker: std::marker::PhantomData<T>,
    }

    #[async_trait]
    impl<T> TypedCache<T> for MockTypedCache<T>
    where
        T: Encode + Decode<()> + Send + Sync + 'static,
    {
        async fn get(&self, _key: &str) -> Result<Option<T>, CacheError> {
            Err(CacheError::Other("Not implemented".to_string()))
        }

        async fn set(
            &self,
            _key: &str,
            _value: T,
            _ttl: Option<Duration>,
        ) -> Result<(), CacheError> {
            Ok(())
        }

        async fn get_many(&self, _keys: &[&str]) -> Result<HashMap<String, Option<T>>, CacheError> {
            Ok(HashMap::new())
        }

        async fn set_many(
            &self,
            _items: HashMap<String, T>,
            _ttl: Option<Duration>,
        ) -> Result<(), CacheError> {
            Ok(())
        }
    }

    // Type-specific factory implementation
    impl<T> TypedCacheFactory<T> for MockTypedCache<T>
    where
        T: Encode + Decode<()> + Send + Sync + 'static,
    {
        fn create_typed_cache(&self) -> Box<dyn TypedCache<T>> {
            Box::new(MockTypedCache {
                cache: self.cache.clone(),
                _marker: std::marker::PhantomData,
            })
        }
    }

    impl CacheFactory for MockCache {
        fn for_type<T>(&self) -> Box<dyn TypedCacheFactory<T>>
        where
            T: Encode + Decode<()> + Send + Sync + 'static,
        {
            Box::new(MockTypedCache {
                cache: Arc::new(self.clone()),
                _marker: std::marker::PhantomData,
            })
        }
    }

    #[async_trait]
    impl DynCacheOperations for MockCache {
        async fn delete(&self, _key: &str) -> Result<bool, CacheError> {
            Ok(true)
        }

        async fn clear(&self) -> Result<(), CacheError> {
            Ok(())
        }

        async fn exists(&self, _key: &str) -> Result<bool, CacheError> {
            Ok(false)
        }

        async fn delete_many(&self, _keys: &[&str]) -> Result<usize, CacheError> {
            Ok(0)
        }

        async fn increment(&self, _key: &str, _delta: i64) -> Result<i64, CacheError> {
            Ok(1)
        }

        fn stats(&self) -> Result<CacheStats, CacheError> {
            Ok(CacheStats {
                size: 0,
                hits: 0,
                misses: 0,
                evictions: 0,
                capacity: None,
                custom_metrics: HashMap::new(),
            })
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn config(&self) -> &CacheConfig {
            &self.config
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl Clone for MockCache {
        fn clone(&self) -> Self {
            Self {
                name: self.name.clone(),
                config: self.config.clone(),
            }
        }
    }

    // Mock provider for testing
    struct MockProvider;

    #[async_trait]
    impl CacheProvider for MockProvider {
        async fn create_cache(
            &self,
            config: CacheConfig,
        ) -> Result<Box<dyn DynCacheOperations>, CacheError> {
            Ok(Box::new(MockCache {
                name: config.name.clone(),
                config,
            }))
        }

        fn supports(&self, _config: &CacheConfig) -> bool {
            true
        }

        fn name(&self) -> &str {
            "mock"
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn test_cache_provider_registry() {
        // Create registry
        let mut registry = CacheProviderRegistry::new();

        // Register mock provider
        registry.register(MockProvider);

        // Verify provider is registered
        assert_eq!(registry.provider_names(), vec!["mock"]);
        assert!(registry.get("mock").is_some());
        assert!(registry.get("nonexistent").is_none());

        // Create cache with configuration
        let config = CacheConfig {
            name: "test-cache".to_string(),
            provider: "mock".to_string(),
            ..Default::default()
        };

        let cache = registry.create_cache(config.clone()).await.unwrap();
        assert_eq!(cache.name(), "test-cache");
        assert_eq!(cache.config().name, config.name);
    }

    #[tokio::test]
    async fn test_cache_provider_not_found() {
        // Create registry without providers
        let registry = CacheProviderRegistry::new();

        // Try to create cache
        let config = CacheConfig {
            name: "test-cache".to_string(),
            provider: "nonexistent".to_string(),
            ..Default::default()
        };

        let result = registry.create_cache(config).await;
        assert!(result.is_err());
        if let Err(CacheError::Configuration(msg)) = result {
            assert!(msg.contains("No provider found"));
        } else {
            panic!("Expected Configuration error");
        }
    }
}
