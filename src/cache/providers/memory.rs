use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use super::CacheProvider;
use crate::core::cache::{
    CacheRegistry, ResourceCache, get_cache_stats_with_metrics, get_or_fetch, get_resource_cache,
    init_cache_registry, register_resource_cache,
};
use crate::metrics::metrics_service::try_record_metrics;
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
