use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use super::CacheProvider;
use crate::core::cache::{
    CacheRegistry, ResourceCache, get_cache_stats_with_metrics, get_or_fetch, get_resource_cache,
    init_cache_registry, register_resource_cache,
};
use crate::core::metrics::try_record_metrics;
use crate::utils::api_resource::ApiResource;

/// In-memory cache provider using Moka cache
pub struct MemoryCacheProvider {
    registry: Arc<CacheRegistry>,
}

impl MemoryCacheProvider {
    /// Create a new memory cache provider
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        let registry = init_cache_registry(true, max_capacity, ttl_seconds);
        Self {
            registry: Arc::new(registry),
        }
    }

    /// Get the underlying cache registry
    pub fn registry(&self) -> Arc<CacheRegistry> {
        self.registry.clone()
    }
}

#[async_trait]
impl CacheProvider for MemoryCacheProvider {
    fn init(&self) -> Result<(), String> {
        // Already initialized with the constructor
        Ok(())
    }

    async fn set<T: ApiResource>(
        &self,
        key: &str,
        value: T,
        _ttl_seconds: u64,
    ) -> Result<(), String> {
        let resource_type = T::resource_type();

        // Ensure the cache is registered for this resource type
        let _ = register_resource_cache::<T>(&self.registry, &resource_type);

        // Get the cache for this resource type
        if let Some(resource_cache) = get_resource_cache::<T>(&self.registry, &resource_type) {
            // Insert the value with the provided key
            resource_cache.cache.insert(key.to_string(), value).await;
            Ok(())
        } else {
            Err(format!(
                "Failed to get cache for resource type: {}",
                resource_type
            ))
        }
    }

    async fn get<T: ApiResource>(&self, key: &str) -> Result<Option<T>, String> {
        let resource_type = T::resource_type();

        // Get the cache for this resource type
        if let Some(resource_cache) = get_resource_cache::<T>(&self.registry, &resource_type) {
            // Get the value with the provided key
            let value = resource_cache.cache.get(&key.to_string()).await;
            Ok(value)
        } else {
            Ok(None) // Cache not found for this resource type
        }
    }

    async fn delete(&self, _key: &str) -> Result<(), String> {
        // Since we need the resource type to find the right cache,
        // and we only have the key, we would need to keep track of
        // which cache contains which key. For simplicity, we'll just
        // report that delete is not supported in this basic implementation.
        Err("Delete by key only is not supported in memory provider. Use get_resource_cache directly.".to_string())
    }

    async fn clear(&self) -> Result<(), String> {
        // This is a basic implementation - for a real app, you'd want to
        // iterate through all registered caches and clear them
        Err("Clear all caches is not supported in this basic implementation. Use get_resource_cache directly.".to_string())
    }

    async fn exists(&self, _key: &str) -> Result<bool, String> {
        // Similar issue as with delete - we need the resource type
        Err("Exists by key only is not supported in memory provider. Use get_resource_cache directly.".to_string())
    }

    async fn get_stats(&self) -> Result<serde_json::Value, String> {
        // Use the metrics recorder to get stats for all caches
        let _metrics_text = match try_record_metrics() {
            Ok(text) => text,
            Err(_) => return Err("Failed to record metrics".to_string()),
        };

        // Get stats for all resource types from the registry
        let stats: Vec<serde_json::Value> = Vec::new();

        // TODO: Implement a more comprehensive approach to get all stats
        // This is just a placeholder
        let result = json!({
            "provider": "memory",
            "enabled": self.registry.enabled,
            "max_capacity": self.registry.max_capacity,
            "ttl_seconds": self.registry.ttl_seconds,
            "stats": stats,
        });

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::api_resource::ApiResource;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    // Test data structure that implements ApiResource
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestResource {
        id: String,
        name: String,
        value: i32,
    }

    impl ApiResource for TestResource {
        type Id = String;

        fn resource_type() -> &'static str {
            "test_resource"
        }

        fn api_name() -> &'static str {
            "TestService"
        }
    }

    #[tokio::test]
    async fn test_memory_cache_provider_creation() {
        let provider = MemoryCacheProvider::new(100, 3600);
        assert!(provider.registry().enabled);
        assert_eq!(provider.registry().max_capacity, 100);
        assert_eq!(provider.registry().ttl_seconds, 3600);
    }

    #[tokio::test]
    async fn test_memory_cache_init() {
        let provider = MemoryCacheProvider::new(100, 3600);
        let result = provider.init();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_memory_cache_set_and_get() {
        let provider = MemoryCacheProvider::new(100, 3600);

        // Create a test resource
        let resource = TestResource {
            id: "test-1".to_string(),
            name: "Test Resource".to_string(),
            value: 42,
        };

        // Register the resource type directly in the cache registry
        let registry = provider.registry();
        // Instead of using register_resource, directly register the cache with our resource type
        let _ = crate::core::cache::cache_manager::register_resource_cache::<TestResource>(
            &registry,
            "test_resource",
        );

        // Set the resource in the cache
        let set_result = provider.set("test-1", resource.clone(), 3600).await;
        assert!(set_result.is_ok());

        // Get the resource from the cache
        let get_result = provider.get::<TestResource>("test-1").await;
        assert!(get_result.is_ok());

        let retrieved_resource = get_result.unwrap();
        assert!(retrieved_resource.is_some());

        let retrieved_resource = retrieved_resource.unwrap();
        assert_eq!(retrieved_resource.id, "test-1");
        assert_eq!(retrieved_resource.name, "Test Resource");
        assert_eq!(retrieved_resource.value, 42);
    }

    #[tokio::test]
    async fn test_memory_cache_get_nonexistent() {
        let provider = MemoryCacheProvider::new(100, 3600);

        // Register the resource type directly in the cache registry
        let registry = provider.registry();
        // Instead of using register_resource, directly register the cache with our resource type
        let _ = crate::core::cache::cache_manager::register_resource_cache::<TestResource>(
            &registry,
            "test_resource",
        );

        // Try to get a resource that doesn't exist
        let get_result = provider.get::<TestResource>("nonexistent").await;
        assert!(get_result.is_ok());

        let retrieved_resource = get_result.unwrap();
        assert!(retrieved_resource.is_none());
    }

    #[tokio::test]
    async fn test_memory_cache_unimplemented_methods() {
        let provider = MemoryCacheProvider::new(100, 3600);

        // Test delete method (should return error since it's not implemented)
        let delete_result = provider.delete("test-key").await;
        assert!(delete_result.is_err());

        // Test clear method (should return error since it's not implemented)
        let clear_result = provider.clear().await;
        assert!(clear_result.is_err());

        // Test exists method (should return error since it's not implemented)
        let exists_result = provider.exists("test-key").await;
        assert!(exists_result.is_err());
    }

    #[tokio::test]
    async fn test_memory_cache_get_stats() {
        let provider = MemoryCacheProvider::new(100, 3600);

        // Get stats (basic implementation should return a JSON object)
        let stats_result = provider.get_stats().await;
        assert!(stats_result.is_ok());

        let stats = stats_result.unwrap();
        assert_eq!(stats["provider"], "memory");
        assert_eq!(stats["enabled"], true);
        assert_eq!(stats["max_capacity"], 100);
        assert_eq!(stats["ttl_seconds"], 3600);
        assert!(stats["stats"].is_array());
    }
}
