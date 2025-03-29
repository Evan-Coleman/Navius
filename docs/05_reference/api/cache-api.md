---
title: "Cache API Reference"
description: "API documentation for Navius caching service and operations"
category: api
tags:
  - api
  - cache
  - performance
  - memory
  - redis
related:
  - ../patterns/cache-provider-pattern.md
  - ../../02_examples/cache-provider-example.md
  - ../../02_examples/two-tier-cache-example.md
last_updated: March 31, 2025
version: 1.0
---


# Cache API Reference

## Overview

The Cache API provides a generic interface for interacting with caching systems through the Cache Service. This reference documents the core interfaces, operations, and usage patterns for working with the Cache Service, including single-tier and two-tier caching strategies.

## Core Interfaces

### CacheOperations

The `CacheOperations` trait defines the core operations available for all cache implementations:

```rust
#[async_trait]
pub trait CacheOperations<T: Send + Sync + Clone + 'static>: Send + Sync {
    /// Get a value from the cache
    async fn get(&self, key: &str) -> Option<T>;
    
    /// Set a value in the cache with optional TTL
    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), CacheError>;
    
    /// Delete a value from the cache
    async fn delete(&self, key: &str) -> Result<bool, CacheError>;
    
    /// Clear the entire cache
    async fn clear(&self) -> Result<(), CacheError>;
    
    /// Get cache statistics
    fn stats(&self) -> CacheStats;
}
```

### CacheProvider

The `CacheProvider` trait enables creating cache instances:

```rust
#[async_trait]
pub trait CacheProvider: Send + Sync {
    /// Create a new cache instance
    async fn create_cache<T: Send + Sync + Clone + 'static>(
        &self, 
        config: CacheConfig
    ) -> Result<Box<dyn CacheOperations<T>>, CacheError>;
    
    /// Check if this provider supports the given configuration
    fn supports(&self, config: &CacheConfig) -> bool;
    
    /// Get the name of this provider
    fn name(&self) -> &str;
}
```

### CacheService

The `CacheService` manages cache instances:

```rust
pub struct CacheService {
    provider_registry: Arc<RwLock<CacheProviderRegistry>>,
    config_by_resource: HashMap<String, CacheConfig>,
    default_config: CacheConfig,
}

impl CacheService {
    pub fn new(registry: CacheProviderRegistry) -> Self {
        // Implementation details...
    }
    
    pub async fn create_cache<T: Send + Sync + Clone + 'static>(
        &self,
        resource_name: &str
    ) -> Result<Box<dyn CacheOperations<T>>, CacheError> {
        // Implementation details...
    }
    
    pub async fn create_two_tier_cache<T: Send + Sync + Clone + 'static>(
        &self,
        resource_name: &str,
        config: TwoTierCacheConfig
    ) -> Result<TwoTierCache<T>, CacheError> {
        // Implementation details...
    }
}
```

### TwoTierCache

The `TwoTierCache` provides multi-level caching capabilities:

```rust
pub struct TwoTierCache<T> {
    fast_cache: Box<dyn CacheOperations<T>>,
    slow_cache: Box<dyn CacheOperations<T>>,
    promote_on_get: bool,
    fast_ttl: Option<Duration>,
}

#[async_trait]
impl<T: Send + Sync + Clone + 'static> CacheOperations<T> for TwoTierCache<T> {
    // Implementation of CacheOperations...
}
```

## Using the Cache API

### Accessing the Cache Service

The cache service is accessible through the application's service registry:

```rust
use crate::core::services::ServiceRegistry;
use crate::core::services::cache_service::CacheService;

// Get the service from service registry
let cache_service = service_registry.get::<CacheService>();

// Create a typed cache for users
let user_cache = cache_service.create_cache::<UserDto>("users").await?;
```

### Basic Cache Operations

#### Setting Values

```rust
use std::time::Duration;

// Create a user
let user = UserDto {
    id: "user-123".to_string(),
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
};

// Cache with 5 minute TTL
user_cache.set(&user.id, user.clone(), Some(Duration::from_secs(300))).await?;

// Cache with default TTL
user_cache.set(&user.id, user.clone(), None).await?;
```

#### Getting Values

```rust
// Get a user from cache
if let Some(user) = user_cache.get("user-123").await {
    println!("Found user: {}", user.name);
} else {
    println!("User not in cache");
    
    // Fetch from database and cache
    let user = db.get_user("user-123").await?;
    user_cache.set("user-123", user.clone(), None).await?;
}
```

#### Deleting Values

```rust
// Delete a cached user
let deleted = user_cache.delete("user-123").await?;
if deleted {
    println!("User removed from cache");
} else {
    println!("User was not in cache");
}
```

#### Clearing Cache

```rust
// Clear the entire cache
user_cache.clear().await?;
```

#### Getting Cache Statistics

```rust
// Get cache statistics
let stats = user_cache.stats();
println!("Cache stats - Hits: {}, Misses: {}, Size: {}", 
         stats.hits, stats.misses, stats.size);
```

### Using Two-Tier Cache

```rust
use crate::core::services::cache_service::TwoTierCacheConfig;

// Configure two-tier cache
let config = TwoTierCacheConfig::new()
    .with_fast_provider("memory")
    .with_slow_provider("redis")
    .with_fast_ttl(Duration::from_secs(60))
    .with_slow_ttl(Duration::from_secs(3600))
    .with_promotion_enabled(true);

// Create a two-tier cache for products
let product_cache = cache_service
    .create_two_tier_cache::<ProductDto>("products", config)
    .await?;
    
// Use it like a regular cache
product_cache.set("product-123", product, None).await?;

// This will check fast cache first, then slow cache, and promote if found
if let Some(product) = product_cache.get("product-123").await {
    println!("Found product: {}", product.name);
}
```

## Available Cache Providers

### MemoryCacheProvider

The in-memory cache provider uses Moka for high-performance caching:

```rust
use crate::core::services::memory_cache::MemoryCacheProvider;

// Create a provider
let provider = MemoryCacheProvider::new();

// Create a cache instance
let config = CacheConfig::default()
    .with_name("user-cache")
    .with_ttl(Duration::from_secs(300))
    .with_max_size(10000);
    
let cache = provider.create_cache::<UserDto>(config).await?;
```

### RedisCacheProvider

The Redis provider is used for distributed caching:

```rust
use crate::core::services::redis_cache::RedisCacheProvider;

// Create a provider with connection string
let provider = RedisCacheProvider::new("redis://localhost:6379");

// Create a cache instance
let config = CacheConfig::default()
    .with_name("product-cache")
    .with_ttl(Duration::from_secs(3600));
    
let cache = provider.create_cache::<ProductDto>(config).await?;
```

## Configuration

The Cache Service can be configured in `config/default.yaml`:

```yaml
# Cache configuration
cache:
  # Default provider to use
  default_provider: memory
  
  # Provider-specific configurations
  providers:
    memory:
      enabled: true
      max_size: 10000
      ttl_seconds: 300
      
    redis:
      enabled: true
      connection_string: redis://localhost:6379
      ttl_seconds: 3600
  
  # Resource-specific cache configurations
  resources:
    users:
      provider: memory
      ttl_seconds: 60
      max_size: 1000
      
    products:
      provider: redis
      ttl_seconds: 1800
      
  # Two-tier cache configurations
  two_tier:
    users:
      fast_provider: memory
      slow_provider: redis
      fast_ttl_seconds: 60
      slow_ttl_seconds: 3600
      promote_on_get: true
```

## Error Handling

The Cache API uses `CacheError` for error handling:

```rust
// Example error handling
match user_cache.set("user-123", user, None).await {
    Ok(_) => {
        println!("User cached successfully");
    },
    Err(e) => {
        match e {
            CacheError::ConnectionError { message, .. } => {
                // Handle connection error
                println!("Cache connection error: {}", message);
            },
            CacheError::SerializationError { message } => {
                // Handle serialization error
                println!("Cache serialization error: {}", message);
            },
            _ => {
                // Handle other errors
                println!("Cache error: {}", e);
            }
        }
    }
}
```

## Cache-Aside Pattern

The cache-aside pattern is a common caching strategy:

```rust
async fn get_user_with_cache(
    id: &str,
    cache: &Box<dyn CacheOperations<UserDto>>,
    db: &Database
) -> Result<UserDto, ServiceError> {
    // Try to get from cache first
    if let Some(user) = cache.get(id).await {
        return Ok(user);
    }
    
    // If not in cache, get from database
    let user_json = db.get("users", id).await?
        .ok_or_else(|| ServiceError::not_found(format!("User not found: {}", id)))?;
        
    let user: UserDto = serde_json::from_str(&user_json)?;
    
    // Store in cache for next time with 5 minute TTL
    let _ = cache.set(id, user.clone(), Some(Duration::from_secs(300))).await;
    
    Ok(user)
}
```

## Implementing a Custom Provider

You can implement your own cache provider by implementing the `CacheProvider` trait:

```rust
use crate::core::services::cache_provider::{
    CacheOperations, CacheProvider, CacheError, CacheStats
};
use crate::core::services::cache_service::CacheConfig;
use async_trait::async_trait;
use std::time::Duration;
use std::marker::PhantomData;

// Custom cache implementation
pub struct CustomCache<T> {
    // Implementation details...
    config: CacheConfig,
    _phantom: PhantomData<T>,
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
        // Implementation...
        None
    }
    
    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), CacheError> {
        // Implementation...
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        // Implementation...
        Ok(false)
    }
    
    async fn clear(&self) -> Result<(), CacheError> {
        // Implementation...
        Ok(())
    }
    
    fn stats(&self) -> CacheStats {
        // Implementation...
        CacheStats {
            hits: 0,
            misses: 0,
            size: 0,
            max_size: self.config.max_size,
        }
    }
}

// Custom provider
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
```

Register your custom provider:

```rust
let mut registry = CacheProviderRegistry::new();
registry.register(Box::new(CustomCacheProvider));

let service = CacheService::new(registry);
```

## Best Practices

### Cache Key Management

- Use consistent key generation strategies
- Include resource type in keys to prevent collisions
- Consider using prefixed keys (e.g., `user:123`)
- Keep keys short but descriptive

### Time-to-Live (TTL) Strategies

- Use shorter TTLs for frequently changing data
- Use longer TTLs for relatively static data
- Set appropriate TTLs for different resources
- Consider different TTLs for different cache tiers

### Cache Invalidation

- Invalidate cache entries when source data changes
- Use versioned keys for complex invalidation scenarios
- Consider bulk invalidation for related data
- Implement cache consistency strategies

### Error Handling

- Fail gracefully when cache operations fail
- Don't let cache failures affect critical operations
- Log cache errors without disrupting user operations
- Add circuit breakers for unreliable cache systems

### Cache Size Management

- Set appropriate maximum sizes for memory caches
- Monitor cache usage and adjust limits as needed
- Use eviction policies that match your access patterns
- Consider separate caches for different resource types

## Performance Considerations

- Use bulk operations when possible
- Select appropriate serialization formats
- Balance cache TTLs against freshness requirements
- Monitor cache hit/miss rates
- Use two-tier caching for frequently accessed data
- Consider data compression for large cached items

## Related Documentation

- [Cache Provider Pattern](../patterns/cache-provider-pattern.md)
- [Two-Tier Cache Example](../../examples/two-tier-cache-example.md)
- [Cache Provider Example](../../examples/cache-provider-example.md) 
