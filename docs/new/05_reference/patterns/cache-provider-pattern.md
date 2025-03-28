---
title: "Cache Provider Pattern"
description: "Design and implementation of the cache provider pattern with pluggable providers"
category: patterns
tags:
  - patterns
  - cache
  - performance
  - architecture
  - providers
related:
  - reference/patterns/repository-pattern.md
  - reference/api/cache-api.md
  - examples/cache-provider-example.md
  - examples/two-tier-cache-example.md
last_updated: March 26, 2025
version: 1.0
---

# Cache Provider Pattern

## Overview

The Cache Provider Pattern is an architectural approach that abstracts caching operations behind a generic interface with pluggable provider implementations. This enables applications to work with different caching technologies through a consistent API while allowing for flexible switching between implementations.

## Problem Statement

Applications often need caching to improve performance and reduce load on backend systems, but direct coupling to specific caching technologies creates several challenges:

- Difficult to switch between caching providers (e.g., in-memory to Redis)
- Testing is complicated by dependencies on external caching systems
- Code becomes tightly coupled to specific caching APIs
- Difficult to implement advanced caching strategies like multi-level caching
- Limited ability to fine-tune caching based on resource characteristics

## Solution: Cache Provider Pattern with Pluggable Providers

The Cache Provider Pattern in Navius uses a provider-based architecture with these components:

1. **CacheOperations Trait**: Defines core caching operations
2. **CacheProvider Trait**: Creates cache instances
3. **CacheProviderRegistry**: Manages and selects appropriate providers
4. **CacheConfig**: Configures cache behavior and settings
5. **CacheService**: Orchestrates cache operations
6. **TwoTierCache**: Implements advanced multi-level caching

### Pattern Structure

```
┌─────────────────┐     creates     ┌───────────────────┐
│  CacheService   │─────────────────│CacheProviderRegistry│
└────────┬────────┘                 └─────────┬─────────┘
         │                                    │ selects
         │                                    ▼
         │                          ┌───────────────────┐
         │                          │   CacheProvider   │
         │                          └─────────┬─────────┘
         │                                    │ creates
         │                                    ▼
         │ uses                     ┌───────────────────┐
         └────────────────────────▶│  CacheOperations   │
                                    └───────────────────┘
```

### Implementation

#### 1. Cache Operations Interface

The `CacheOperations` trait defines the contract for all cache implementations:

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

#### 2. Cache Provider Interface

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

#### 3. Cache Service

The `CacheService` manages cache instances and provides access to them:

```rust
pub struct CacheService {
    provider_registry: Arc<RwLock<CacheProviderRegistry>>,
    config_by_resource: HashMap<String, CacheConfig>,
    default_config: CacheConfig,
}

impl CacheService {
    pub fn new(registry: CacheProviderRegistry) -> Self {
        Self {
            provider_registry: Arc::new(RwLock::new(registry)),
            config_by_resource: HashMap::new(),
            default_config: CacheConfig::default(),
        }
    }
    
    pub async fn create_cache<T: Send + Sync + Clone + 'static>(
        &self,
        resource_name: &str
    ) -> Result<Box<dyn CacheOperations<T>>, CacheError> {
        // Use registry to create appropriate cache instance for the resource
    }
    
    pub async fn create_two_tier_cache<T: Send + Sync + Clone + 'static>(
        &self,
        resource_name: &str,
        config: TwoTierCacheConfig
    ) -> Result<TwoTierCache<T>, CacheError> {
        // Create a two-tier cache with fast and slow caches
    }
}
```

## Benefits

1. **Abstraction**: Decouples application from specific caching technologies
2. **Testability**: Simplifies testing with in-memory cache implementations
3. **Flexibility**: Easy to switch between cache providers
4. **Multi-Level Caching**: Supports advanced caching strategies
5. **Type Safety**: Generic typing ensures type safety across cache operations
6. **Configuration**: Resource-specific cache configuration
7. **Metrics**: Consistent cache statistics and monitoring

## Implementation Considerations

### 1. Cache Key Management

Proper key management is essential for effective caching:

- Use consistent key generation strategies
- Include resource type in keys to prevent collisions
- Consider using prefixes for different resources
- Support key namespaces to isolate different parts of the application

### 2. Time-to-Live (TTL) Strategies

Different resources may need different TTL strategies:

- Critical, frequently changing data: shorter TTLs
- Static content: longer TTLs, potentially indefinite
- User-specific data: session-based TTLs
- Two-tier caching: different TTLs for each tier

### 3. Cache Eviction Policies

Implement appropriate eviction policies:

- LRU (Least Recently Used)
- LFU (Least Frequently Used)
- Size-based eviction
- Time-based expiration
- Custom eviction strategies

### 4. Cache Synchronization

In distributed environments, consider cache synchronization:

- Implement cache invalidation messaging
- Use versioning for cache entries
- Consider eventual consistency implications
- Use distributed cache systems (Redis) for shared state

### 5. Error Handling

Caching should not affect critical application flow:

- Fail gracefully when cache operations fail
- Log cache errors without disrupting user operations
- Consider cache-aside pattern for resilience
- Implement circuit breaker for cache operations

## Example Implementations

### In-Memory Cache

```rust
pub struct MemoryCache<T> {
    cache: Arc<Mutex<moka::sync::Cache<String, T>>>,
    config: CacheConfig,
    stats: Arc<CacheStats>,
}

#[async_trait]
impl<T: Send + Sync + Clone + 'static> CacheOperations<T> for MemoryCache<T> {
    async fn get(&self, key: &str) -> Option<T> {
        let result = self.cache.lock().unwrap().get(key).cloned();
        
        // Update stats
        if result.is_some() {
            self.stats.increment_hits();
        } else {
            self.stats.increment_misses();
        }
        
        result
    }
    
    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), CacheError> {
        let ttl = ttl.unwrap_or(self.config.ttl);
        
        let mut cache = self.cache.lock().unwrap();
        if let Some(ttl) = ttl {
            cache.insert_with_ttl(key.to_string(), value, ttl);
        } else {
            cache.insert(key.to_string(), value);
        }
        
        Ok(())
    }
    
    // Other methods implementation...
}
```

### Redis Cache

```rust
pub struct RedisCache<T> {
    client: redis::Client,
    serializer: Box<dyn Serializer<T>>,
    config: CacheConfig,
    stats: Arc<CacheStats>,
}

#[async_trait]
impl<T: Send + Sync + Clone + 'static> CacheOperations<T> for RedisCache<T> {
    async fn get(&self, key: &str) -> Option<T> {
        let mut conn = self.client.get_async_connection().await.ok()?;
        
        let result: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .ok()?;
            
        let value = result.and_then(|data| self.serializer.deserialize(&data).ok());
        
        // Update stats
        if value.is_some() {
            self.stats.increment_hits();
        } else {
            self.stats.increment_misses();
        }
        
        value
    }
    
    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await?;
        let data = self.serializer.serialize(&value)?;
        
        let ttl = ttl.unwrap_or(self.config.ttl);
        
        if let Some(ttl) = ttl {
            redis::cmd("SETEX")
                .arg(key)
                .arg(ttl.as_secs())
                .arg(data)
                .query_async(&mut conn)
                .await?;
        } else {
            redis::cmd("SET")
                .arg(key)
                .arg(data)
                .query_async(&mut conn)
                .await?;
        }
        
        Ok(())
    }
    
    // Other methods implementation...
}
```

### Two-Tier Cache Implementation

```rust
pub struct TwoTierCache<T> {
    fast_cache: Box<dyn CacheOperations<T>>,
    slow_cache: Box<dyn CacheOperations<T>>,
    promote_on_get: bool,
    fast_ttl: Option<Duration>,
}

#[async_trait]
impl<T: Send + Sync + Clone + 'static> CacheOperations<T> for TwoTierCache<T> {
    async fn get(&self, key: &str) -> Option<T> {
        // Try fast cache first
        if let Some(value) = self.fast_cache.get(key).await {
            return Some(value);
        }
        
        // If not in fast cache, try slow cache
        if let Some(value) = self.slow_cache.get(key).await {
            // Promote to fast cache if configured to do so
            if self.promote_on_get {
                let value_clone = value.clone();
                let _ = self.fast_cache.set(key, value_clone, self.fast_ttl).await;
            }
            
            return Some(value);
        }
        
        None
    }
    
    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), CacheError> {
        // Set in both caches
        let value_clone = value.clone();
        let fast_result = self.fast_cache.set(key, value_clone, self.fast_ttl).await;
        let slow_result = self.slow_cache.set(key, value, ttl).await;
        
        // Return error if either operation failed
        fast_result.and(slow_result)
    }
    
    // Other methods implementation...
}
```

## API Example

```rust
// Get the cache service
let cache_service = service_registry.get::<CacheService>();

// Create a typed cache for users
let user_cache: Box<dyn CacheOperations<UserDto>> = 
    cache_service.create_cache("users").await?;

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

// Create a two-tier cache
let config = TwoTierCacheConfig::new()
    .with_fast_provider("memory")
    .with_slow_provider("redis")
    .with_fast_ttl(Duration::from_secs(60))
    .with_slow_ttl(Duration::from_secs(3600))
    .with_promotion_enabled(true);

let two_tier_cache = cache_service
    .create_two_tier_cache::<ProductDto>("products", config)
    .await?;
    
// Use the two-tier cache
two_tier_cache.set("product-1", product, None).await?;
```

## Related Patterns

- **Repository Pattern**: Often used with Cache Provider Pattern for cached data access
- **Strategy Pattern**: Different cache providers implement different strategies
- **Adapter Pattern**: Adapts specific cache APIs to the common interface
- **Decorator Pattern**: Used in two-tier caching to layer cache functionality
- **Factory Pattern**: Used to create cache instances
- **Builder Pattern**: Used for configuration building

## References

- [Cache-Aside Pattern](https://docs.microsoft.com/en-us/azure/architecture/patterns/cache-aside)
- [Multi-Level Cache](https://mechanical-sympathy.blogspot.com/2013/07/multi-layer-caches.html)
- [Redis Documentation](https://redis.io/documentation)
- [Caffeine Cache](https://github.com/ben-manes/caffeine) 