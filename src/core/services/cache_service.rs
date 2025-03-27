use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use bincode::{Decode, Encode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tracing::{error, info};

use crate::core::services::cache_provider::{
    CacheConfig, CacheError, CacheFactory, CacheOperations, CacheProvider, CacheProviderRegistry,
    CacheStats, DynCacheOperations, TypedCache, TypedCacheFactory,
};
use crate::core::services::error::ServiceError;
use crate::core::services::memory_cache::ClonableDynCacheOperations;

/// Service for managing caches using provider registry
pub struct CacheService {
    /// Provider registry for creating caches
    provider_registry: Arc<RwLock<CacheProviderRegistry>>,
    /// Cache instances
    caches: Arc<RwLock<HashMap<String, Box<dyn DynCacheOperations>>>>,
}

impl CacheService {
    /// Create a new cache service
    pub fn new(provider_registry: Arc<RwLock<CacheProviderRegistry>>) -> Self {
        Self {
            provider_registry,
            caches: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a cache with the specified configuration
    pub async fn get_cache(
        &self,
        config: CacheConfig,
    ) -> Result<Arc<Box<dyn DynCacheOperations>>, CacheError> {
        let name = config.name.clone();

        // Check if we already have this cache
        {
            let caches = self.caches.read().unwrap();
            if let Some(cache) = caches.get(&name) {
                // Now we can clone the Box<dyn DynCacheOperations>
                return Ok(Arc::new(cache.clone()));
            }
        }

        // Create the cache
        let cache = {
            let registry = self.provider_registry.read().unwrap();
            registry.create_cache(config).await?
        };

        // Store the cache
        {
            let mut caches = self.caches.write().unwrap();
            caches.insert(name.clone(), cache.clone());
            Ok(Arc::new(cache))
        }
    }

    /// Get an existing cache by name
    pub fn get_existing_cache(&self, name: &str) -> Option<Arc<Box<dyn DynCacheOperations>>> {
        let caches = self.caches.read().unwrap();
        caches.get(name).map(|c| Arc::new(c.clone()))
    }

    /// Get list of available provider names
    pub fn available_providers(&self) -> Vec<String> {
        let registry = self.provider_registry.read().unwrap();
        registry.provider_names()
    }

    /// Register a new cache provider
    pub fn register_provider<P>(&self, provider: P)
    where
        P: CacheProvider + 'static,
    {
        let mut registry = self.provider_registry.write().unwrap();
        registry.register(provider);
    }

    /// Get list of cache names
    pub fn cache_names(&self) -> Vec<String> {
        let caches = self.caches.read().unwrap();
        caches.keys().cloned().collect()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> HashMap<String, Result<CacheStats, CacheError>> {
        let caches = self.caches.read().unwrap();
        let mut stats = HashMap::new();

        for (name, cache) in caches.iter() {
            stats.insert(name.clone(), cache.stats());
        }

        stats
    }

    /// Clear a specific cache
    pub async fn clear_cache(&self, name: &str) -> Result<(), CacheError> {
        if let Some(cache) = self.get_existing_cache(name) {
            cache.clear().await?;
            Ok(())
        } else {
            Err(CacheError::Key(format!("Cache {} not found", name)))
        }
    }

    /// Clear all caches
    pub async fn clear_all_caches(&self) -> Result<(), CacheError> {
        let cache_names = self.cache_names();

        for name in cache_names {
            // Ignore errors for individual caches
            let _ = self.clear_cache(&name).await;
        }

        Ok(())
    }
}

/// Helper trait to simplify caching operations
#[async_trait]
pub trait CacheHelpers {
    /// Get a typed cache for working with a specific type
    fn get_typed_cache<T>(&self) -> Box<dyn TypedCache<T>>
    where
        T: Encode + Decode<()> + Send + Sync + 'static;

    /// Get a value from the cache
    async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: Encode + Decode<()> + Send + Sync + 'static;

    /// Set a value in the cache with optional TTL
    async fn set<T>(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), CacheError>
    where
        T: Encode + Decode<()> + Send + Sync + 'static;

    /// Get multiple values from the cache
    async fn get_many<T>(&self, keys: &[&str]) -> Result<HashMap<String, Option<T>>, CacheError>
    where
        T: Encode + Decode<()> + Send + Sync + 'static;

    /// Set multiple values in the cache
    async fn set_many<T>(
        &self,
        items: HashMap<String, T>,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError>
    where
        T: Encode + Decode<()> + Send + Sync + 'static,
    {
        let typed = self.get_typed_cache::<T>();
        typed.set_many(items, ttl).await
    }
}

// Helper function to get the typed cache factory
fn get_object_factory<T>(cache: &dyn DynCacheOperations) -> Option<Box<dyn TypedCacheFactory<T>>>
where
    T: Encode + Decode<()> + Send + Sync + 'static,
{
    // Try to downcast to known cache types
    let any = cache.as_any();

    if let Some(memory_cache) =
        any.downcast_ref::<crate::core::services::memory_cache::InMemoryCache>()
    {
        return Some(memory_cache.for_type::<T>());
    }

    if let Some(redis_cache) = any.downcast_ref::<crate::core::services::redis_cache::RedisCache>()
    {
        return Some(redis_cache.for_type::<T>());
    }

    // Add more cache types here as needed

    None
}

#[async_trait]
impl CacheHelpers for dyn DynCacheOperations {
    fn get_typed_cache<T>(&self) -> Box<dyn TypedCache<T>>
    where
        T: Encode + Decode<()> + Send + Sync + 'static,
    {
        if let Some(factory) = get_object_factory::<T>(self) {
            factory.create_typed_cache()
        } else {
            // Fallback to an error-returning implementation
            struct ErrorCache<T> {
                cache_name: String,
                _marker: std::marker::PhantomData<T>,
            }

            #[async_trait]
            impl<T> TypedCache<T> for ErrorCache<T>
            where
                T: Encode + Decode<()> + Send + Sync + 'static,
            {
                async fn get(&self, _key: &str) -> Result<Option<T>, CacheError> {
                    Err(CacheError::Operation(format!(
                        "Cache {} does not support typed operations for type {}",
                        self.cache_name,
                        std::any::type_name::<T>()
                    )))
                }

                async fn set(
                    &self,
                    _key: &str,
                    _value: T,
                    _ttl: Option<Duration>,
                ) -> Result<(), CacheError> {
                    Err(CacheError::Operation(format!(
                        "Cache {} does not support typed operations for type {}",
                        self.cache_name,
                        std::any::type_name::<T>()
                    )))
                }

                async fn get_many(
                    &self,
                    _keys: &[&str],
                ) -> Result<HashMap<String, Option<T>>, CacheError> {
                    Err(CacheError::Operation(format!(
                        "Cache {} does not support typed operations for type {}",
                        self.cache_name,
                        std::any::type_name::<T>()
                    )))
                }

                async fn set_many(
                    &self,
                    _items: HashMap<String, T>,
                    _ttl: Option<Duration>,
                ) -> Result<(), CacheError> {
                    Err(CacheError::Operation(format!(
                        "Cache {} does not support typed operations for type {}",
                        self.cache_name,
                        std::any::type_name::<T>()
                    )))
                }
            }

            Box::new(ErrorCache {
                cache_name: self.name().to_string(),
                _marker: std::marker::PhantomData,
            })
        }
    }

    async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: Encode + Decode<()> + Send + Sync + 'static,
    {
        let typed = self.get_typed_cache::<T>();
        typed.get(key).await
    }

    async fn set<T>(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), CacheError>
    where
        T: Encode + Decode<()> + Send + Sync + 'static,
    {
        let typed = self.get_typed_cache::<T>();
        typed.set(key, value, ttl).await
    }

    async fn get_many<T>(&self, keys: &[&str]) -> Result<HashMap<String, Option<T>>, CacheError>
    where
        T: Encode + Decode<()> + Send + Sync + 'static,
    {
        let typed = self.get_typed_cache::<T>();
        typed.get_many(keys).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::services::memory_cache::InMemoryCacheProvider;

    // Simple test struct that implements Encode and Decode
    #[derive(Debug, PartialEq, Clone, Encode, Decode)]
    struct TestData {
        id: u32,
        value: String,
    }

    #[tokio::test]
    async fn test_cache_service() {
        // Create a provider registry
        let registry = Arc::new(RwLock::new(CacheProviderRegistry::new()));

        // Register a provider
        {
            let mut reg = registry.write().unwrap();
            reg.register(InMemoryCacheProvider::new());
        }

        // Create the cache service
        let service = CacheService::new(Arc::clone(&registry));

        // Check available providers
        let providers = service.available_providers();
        assert_eq!(providers, vec!["memory"]);

        // Create a cache
        let config = CacheConfig {
            name: "test-cache".to_string(),
            provider: "memory".to_string(),
            ..Default::default()
        };

        let cache = service.get_cache(config).await.unwrap();

        // Test basic operations
        let test_data = TestData {
            id: 1,
            value: "value1".to_string(),
        };

        cache.set("key1", test_data.clone(), None).await.unwrap();
        let value: Option<TestData> = cache.get("key1").await.unwrap();
        assert_eq!(value, Some(test_data));

        // Test get existing cache
        let existing = service.get_existing_cache("test-cache").unwrap();
        let value: Option<TestData> = existing.get("key1").await.unwrap();
        assert_eq!(value.unwrap().id, 1);

        // Test cache names
        let names = service.cache_names();
        assert_eq!(names, vec!["test-cache"]);

        // Test cache stats
        let stats = service.cache_stats();
        assert!(stats.contains_key("test-cache"));

        // Test clearing cache
        service.clear_cache("test-cache").await.unwrap();
        let value: Option<TestData> = cache.get("key1").await.unwrap();
        assert_eq!(value, None);
    }
}
