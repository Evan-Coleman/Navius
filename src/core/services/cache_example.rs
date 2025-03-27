use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::core::services::cache_provider::{CacheConfig, CacheProviderRegistry, EvictionPolicy};
use crate::core::services::cache_service::{CacheHelpers, CacheService};
use crate::core::services::memory_cache::InMemoryCacheProvider;
use crate::core::services::redis_cache::RedisCacheProvider;

/// Example usage of the cache service
pub async fn cache_service_example() -> Result<(), String> {
    // Create provider registry
    let registry = Arc::new(RwLock::new(CacheProviderRegistry::new()));

    // Register providers
    {
        let mut reg = registry.write().unwrap();
        reg.register(InMemoryCacheProvider::new());
        reg.register(RedisCacheProvider::new());
    }

    // Create cache service
    let cache_service = CacheService::new(Arc::clone(&registry));

    // List available providers
    let providers = cache_service.available_providers();
    println!("Available providers: {:?}", providers);

    // Create configuration
    let config = CacheConfig {
        name: "example-cache".to_string(),
        provider: "memory".to_string(),
        capacity: Some(1000),
        default_ttl: Some(Duration::from_secs(300)), // 5 minutes
        eviction_policy: EvictionPolicy::LRU,
        provider_config: HashMap::new(),
    };

    // Get or create cache
    let cache = cache_service
        .get_cache(config)
        .await
        .map_err(|e| e.to_string())?;

    // Store a simple string value
    cache
        .set("greeting", "Hello, world!", None)
        .await
        .map_err(|e| e.to_string())?;

    // Store a value with specific TTL
    cache
        .set(
            "temporary",
            "This will expire soon",
            Some(Duration::from_secs(10)),
        )
        .await
        .map_err(|e| e.to_string())?;

    // Store a complex value (using serde)
    let complex_data = vec![
        ("item1".to_string(), 100),
        ("item2".to_string(), 200),
        ("item3".to_string(), 300),
    ];
    cache
        .set("complex", complex_data, None)
        .await
        .map_err(|e| e.to_string())?;

    // Retrieve a string value
    let greeting: Option<String> = cache.get("greeting").await.map_err(|e| e.to_string())?;
    println!("Greeting: {:?}", greeting);

    // Retrieve a complex value
    let complex: Option<Vec<(String, i32)>> =
        cache.get("complex").await.map_err(|e| e.to_string())?;
    println!("Complex data: {:?}", complex);

    // Direct use of typed cache
    let typed_cache = cache.get_typed_cache::<i32>();
    typed_cache
        .set("direct", 42, None)
        .await
        .map_err(|e| e.to_string())?;
    let value = typed_cache.get("direct").await.map_err(|e| e.to_string())?;
    println!("Direct value: {:?}", value);

    // Batch operations
    let mut batch_data = HashMap::new();
    batch_data.insert("batch1".to_string(), "Value 1");
    batch_data.insert("batch2".to_string(), "Value 2");
    batch_data.insert("batch3".to_string(), "Value 3");

    // Store multiple values at once
    cache
        .set_many(batch_data, None)
        .await
        .map_err(|e| e.to_string())?;

    // Retrieve multiple values at once
    let batch_results: HashMap<String, Option<String>> = cache
        .get_many(&["batch1", "batch2", "missing"])
        .await
        .map_err(|e| e.to_string())?;

    println!("Batch results: {:?}", batch_results);

    // Get cache statistics
    let stats = cache.stats().map_err(|e| e.to_string())?;
    println!("Cache stats: {:?}", stats);

    // Increment a counter (for visitor counts, etc.)
    let count = cache
        .increment("visitor_count", 1)
        .await
        .map_err(|e| e.to_string())?;
    println!("Visitor count: {}", count);

    // Clear the cache
    cache.clear().await.map_err(|e| e.to_string())?;
    println!("Cache cleared");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_example() {
        let result = cache_service_example().await;
        assert!(result.is_ok());
    }
}
