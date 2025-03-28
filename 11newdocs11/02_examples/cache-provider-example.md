---
title: "Cache Provider Example"
description: "Examples of using the generic cache service interfaces and providers"
category: examples
tags:
  - cache
  - service
  - generalization
  - providers
  - performance
related:
  - examples/two-tier-cache-example.md
  - roadmaps/25-generic-service-implementations.md
  - roadmaps/07-enhanced-caching.md
last_updated: March 27, 2025
version: 1.0
---

# Cache Provider Example

This example demonstrates how to use the generic cache service implementation, including working with different cache providers, configuring caches, and implementing custom providers.

## Overview

The Cache Service implementation follows a provider-based architecture that enables:

- Abstracting cache operations from specific implementations
- Supporting multiple cache types through providers (in-memory, Redis, etc.)
- Configuration-based selection of cache providers
- Layered caching through the two-tier cache implementation
- Consistent interface for all cache operations

## Core Components

The cache service architecture consists of several key components:

1. **CacheOperations Trait**: Defines core cache operations (get, set, delete)
2. **CacheProvider Trait**: Defines interface for creating cache instances
3. **CacheProviderRegistry**: Manages and creates cache instances
4. **CacheConfig**: Configures cache settings
5. **MemoryCacheProvider**: Default in-memory implementation
6. **RedisCacheProvider**: Redis-based implementation
7. **TwoTierCache**: Implementation that combines fast and slow caches

## Basic Usage

### Accessing the Cache Service

The cache service is accessible through the application's service registry:

```rust
use crate::core::services::ServiceRegistry;
use crate::core::services::cache_service::CacheService;

// Get the service from service registry
let cache_service = service_registry.get::<CacheService>();

// Create a typed cache for a specific resource
let user_cache = cache_service.create_cache::<UserDto>("users").await?;
```rust

### Performing Basic Cache Operations

Once you have a cache instance, you can perform operations:

```rust
use std::time::Duration;

// Set a value with 5 minute TTL
user_cache.set("user-123", user_dto, Some(Duration::from_secs(300))).await?;

// Get a value
if let Some(user) = user_cache.get("user-123").await {
    println!("Found user: {}", user.name);
}

// Delete a value
user_cache.delete("user-123").await?;

// Clear the entire cache
user_cache.clear().await?;
```rust

## Implementing a Custom Cache Provider

You can implement your own cache provider by implementing the `CacheProvider` trait:

```rust
use crate::core::services::cache_provider::{CacheOperations, CacheProvider, CacheError};
use crate::core::services::cache_service::CacheConfig;
use async_trait::async_trait;
use std::time::Duration;
use std::marker::PhantomData;

pub struct CustomCacheProvider;

#[async_trait]
impl CacheProvider for CustomCacheProvider {
    async fn create_cache<T: Send + Sync + Clone + 'static>(
        &self,
        config: CacheConfig
    ) -> Result<Box<dyn CacheOperations<T>>, CacheError> {
        Ok(Box::new(CustomCache::<T>::new(config)))
    }
    
    fn supports(&self, config: &CacheConfig) -> bool {
        config.provider_type == "custom"
    }
    
    fn name(&self) -> &str {
        "custom"
    }
}

pub struct CustomCache<T> {
    config: CacheConfig,
    _phantom: PhantomData<T>,
    // Your cache implementation details here
}

impl<T> CustomCache<T> {
    fn new(config: CacheConfig) -> Self {
        Self {
            config,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T: Send + Sync + Clone + 'static> CacheOperations<T> for CustomCache<T> {
    async fn get(&self, key: &str) -> Option<T> {
        // Implement get operation
        None
    }
    
    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), CacheError> {
        // Implement set operation
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        // Implement delete operation
        Ok(false)
    }
    
    async fn clear(&self) -> Result<(), CacheError> {
        // Implement clear operation
        Ok(())
    }
    
    fn stats(&self) -> crate::core::services::cache_provider::CacheStats {
        // Return cache statistics
        crate::core::services::cache_provider::CacheStats {
            hits: 0,
            misses: 0,
            size: 0,
            max_size: self.config.max_size,
        }
    }
}
```rust

## Registering a Provider

Register your custom provider with the cache service:

```rust
use crate::core::services::cache_provider::CacheProviderRegistry;
use crate::core::services::cache_service::CacheService;

// Setup cache service with custom provider
async fn setup_cache_service() -> CacheService {
    // Create a registry
    let mut registry = CacheProviderRegistry::new();
    
    // Register built-in providers
    registry.register(Box::new(MemoryCacheProvider::new()));
    registry.register(Box::new(RedisCacheProvider::new()));
    
    // Register custom provider
    registry.register(Box::new(CustomCacheProvider));
    
    // Create service with registry
    let cache_service = CacheService::new(registry);
    
    // Initialize the service
    cache_service.init().await.unwrap();
    
    cache_service
}
```rust

## Using the In-Memory Cache Provider

The in-memory cache provider is useful for local caching and testing:

```rust
use crate::core::services::memory_cache::MemoryCacheProvider;
use crate::core::services::cache_service::CacheConfig;
use std::time::Duration;

#[tokio::test]
async fn test_memory_cache() {
    // Create a provider and configuration
    let provider = MemoryCacheProvider::new();
    let config = CacheConfig::default()
        .with_name("test-cache")
        .with_ttl(Duration::from_secs(60))
        .with_max_size(1000);
    
    // Create a cache instance
    let cache = provider.create_cache::<String>(config).await.unwrap();
    
    // Set a test value
    cache.set("greeting", "Hello, world!".to_string(), None).await.unwrap();
    
    // Get the value back
    let value = cache.get("greeting").await;
    assert_eq!(value, Some("Hello, world!".to_string()));
}
```rust

## Using the Redis Cache Provider

The Redis cache provider is used for distributed caching:

```rust
use crate::core::services::redis_cache::RedisCacheProvider;
use crate::core::services::cache_service::CacheConfig;

async fn setup_redis_cache() {
    // Create a provider and configuration
    let provider = RedisCacheProvider::new("redis://localhost:6379");
    let config = CacheConfig::default()
        .with_provider("redis")
        .with_name("user-cache");
    
    // Create a cache instance
    let cache = provider.create_cache::<UserDto>(config).await.unwrap();
    
    // Use the cache
    // ...
}
```rust

## Two-Tier Cache Implementation

The two-tier cache combines a fast in-memory cache with a slower persistent cache:

```rust
use crate::core::services::cache_service::{TwoTierCache, CacheConfig};
use crate::core::services::memory_cache::MemoryCacheProvider;
use crate::core::services::redis_cache::RedisCacheProvider;
use std::time::Duration;

async fn setup_two_tier_cache() {
    // Create providers
    let memory_provider = MemoryCacheProvider::new();
    let redis_provider = RedisCacheProvider::new("redis://localhost:6379");
    
    // Create memory cache config (shorter TTL)
    let memory_config = CacheConfig::default()
        .with_provider("memory")
        .with_ttl(Duration::from_secs(60))
        .with_max_size(1000);
    
    // Create Redis cache config (longer TTL)
    let redis_config = CacheConfig::default()
        .with_provider("redis")
        .with_ttl(Duration::from_secs(3600));
    
    // Create individual caches
    let fast_cache = memory_provider.create_cache::<UserDto>(memory_config).await.unwrap();
    let slow_cache = redis_provider.create_cache::<UserDto>(redis_config).await.unwrap();
    
    // Create two-tier cache
    let two_tier_cache = TwoTierCache::new(fast_cache, slow_cache);
    
    // Use the cache - automatically manages both tiers
    two_tier_cache.get("user-123").await;
}
```rust

## Configuration

Configure the cache service in your application configuration:

```yaml
# In config/default.yaml
cache:
  default_provider: memory
  providers:
    memory:
      enabled: true
      max_size: 10000
      ttl_seconds: 300
    redis:
      enabled: true
      connection_string: redis://localhost:6379
      ttl_seconds: 3600
  resources:
    users:
      provider: memory
      ttl_seconds: 60
      max_size: 1000
    products:
      provider: redis
      ttl_seconds: 1800
```rust

Loading the configuration:

```rust
use crate::core::config::AppConfig;
use crate::core::services::cache_service::CacheConfig;

// Load from application config
let app_config = AppConfig::load()?;
let cache_config = CacheConfig::from_app_config(&app_config, "users");

// Or create it programmatically
let cache_config = CacheConfig::default()
    .with_provider("memory")
    .with_name("users")
    .with_ttl(Duration::from_secs(60))
    .with_max_size(1000);
```rust

## Complete Example

Here's a complete example showing how to set up and use the cache service:

```rust
use crate::core::services::cache_service::{
    CacheService, CacheConfig, CacheOperations
};
use crate::core::services::cache_provider::CacheProviderRegistry;
use crate::core::services::memory_cache::register_memory_cache_provider;
use crate::core::services::redis_cache::register_redis_cache_provider;
use std::time::Duration;

// Example user DTO
#[derive(Clone)]
struct UserDto {
    id: String,
    name: String,
    email: String,
}

async fn setup_cache_service() -> Result<CacheService, CacheError> {
    // Create a provider registry
    let mut registry = CacheProviderRegistry::new();
    
    // Register providers
    register_memory_cache_provider(&mut registry);
    register_redis_cache_provider(&mut registry, "redis://localhost:6379");
    
    // Create service
    let service = CacheService::new(registry);
    
    // Initialize the service
    service.init().await?;
    
    Ok(service)
}

async fn cache_example(service: &CacheService) -> Result<(), CacheError> {
    // Create a typed cache for users
    let user_cache = service.create_cache::<UserDto>("users").await?;
    
    // Create a user
    let user = UserDto {
        id: "user-123".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    // Cache the user with 5 minute TTL
    user_cache.set(&user.id, user.clone(), Some(Duration::from_secs(300))).await?;
    
    // Get the user from cache
    if let Some(cached_user) = user_cache.get(&user.id).await {
        println!("Found user: {}", cached_user.name);
    }
    
    // Get cache statistics
    let stats = user_cache.stats();
    println!("Cache stats - Hits: {}, Misses: {}, Size: {}", 
             stats.hits, stats.misses, stats.size);
    
    Ok(())
}
```rust

## Integration with Two-Tier Cache

The generic cache providers can be used with the existing two-tier cache system:

```rust
use crate::core::services::cache_service::{TwoTierCache, TwoTierCacheConfig};

async fn two_tier_example(service: &CacheService) -> Result<(), CacheError> {
    // Configure two-tier cache
    let config = TwoTierCacheConfig::new()
        .with_fast_provider("memory")
        .with_slow_provider("redis")
        .with_fast_ttl(Duration::from_secs(60))
        .with_slow_ttl(Duration::from_secs(3600))
        .with_promotion_enabled(true);
    
    // Create a two-tier cache
    let users_cache = service.create_two_tier_cache::<UserDto>("users", config).await?;
    
    // Use it like a regular cache - automatically manages both tiers
    let user = UserDto {
        id: "user-456".to_string(),
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };
    
    // Set in both tiers
    users_cache.set(&user.id, user.clone(), None).await?;
    
    // Get first tries memory, then Redis if not found
    if let Some(cached_user) = users_cache.get(&user.id).await {
        println!("Found user: {}", cached_user.name);
    }
    
    Ok(())
}
```rust

## Best Practices

1. **Provider Selection**: Choose the appropriate provider based on your requirements:
   - Memory cache for fast local caching
   - Redis cache for distributed caching
   - Two-tier cache for balance of performance and durability

2. **TTL Management**: Set appropriate time-to-live values:
   - Shorter TTLs for frequently changing data
   - Longer TTLs for relatively static data
   - Consider using different TTLs for different cache tiers

3. **Cache Invalidation**: Implement proper invalidation strategies:
   - Delete cache entries when the source data changes
   - Use version or timestamp-based invalidation
   - Consider using cache groups for bulk invalidation

4. **Error Handling**: Gracefully handle cache errors:
   - Don't let cache failures affect critical operations
   - Use fallbacks when cache is unavailable
   - Log cache errors for monitoring

5. **Performance**: Optimize cache usage for performance:
   - Cache expensive operations rather than simple lookups
   - Monitor cache hit rates and adjust strategies
   - Carefully select what to cache based on access patterns

6. **Security**: Consider security implications:
   - Don't cache sensitive information unless necessary
   - Encrypt sensitive cached data if required
   - Set appropriate permissions on Redis instances

## Related Documentation

- [Two-Tier Cache Example](two-tier-cache-example.md)
- [Generic Service Implementations Roadmap](../roadmaps/25-generic-service-implementations.md)
- [Enhanced Caching Roadmap](../roadmaps/07-enhanced-caching.md) 