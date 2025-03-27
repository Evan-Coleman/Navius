---
title: "Advanced Caching Strategies in Navius"
description: "Implementation guide for advanced caching strategies including two-tier cache fallback in Navius applications"
category: guides
tags:
  - features
  - caching
  - redis
  - performance
  - two-tier
  - memory
  - fallback
related:
  - features/caching.md
  - ../reference/configuration/cache-config.md
  - ../reference/patterns/caching-patterns.md
  - ../examples/two-tier-cache-example.md
last_updated: March 26, 2024
version: 1.0
---

# Caching Strategies in Navius

This guide describes the caching strategies available in Navius, including basic caching, Redis integration, and the two-tier cache fallback implementation.

## Overview

Navius provides a flexible caching system with multiple providers and strategies:

1. **In-Memory Cache**: Fast, local cache that lives in the application's memory
2. **Redis Cache**: Distributed cache for use across multiple application instances
3. **Two-Tier Cache**: Combines both memory and Redis caches for optimal performance

## Cache Provider Interface

Navius uses a provider-based approach for caching, allowing different implementations to be used with the same interface:

```rust
pub trait CacheOperations {
    fn name(&self) -> &str;
    
    async fn set<T: AsRef<[u8]>>(&self, key: &str, value: T, ttl_seconds: Option<u64>) -> Result<()>;
    
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    async fn delete(&self, key: &str) -> Result<bool>;
    
    async fn clear(&self) -> Result<()>;
    
    async fn get_many(&self, keys: &[&str]) -> Result<HashMap<String, Vec<u8>>>;
}
```

## Two-Tier Cache Fallback Strategy

The Two-Tier Cache provides a cache fallback strategy that combines the speed of an in-memory cache with the persistence of a Redis cache.

### How It Works

1. **Reads**:
   - Try to get from the fast cache (in-memory) first
   - If not found, try to get from the slow cache (Redis)
   - If found in the slow cache, promote the item to the fast cache automatically
   - Return the result from whichever cache it was found in

2. **Writes**:
   - Write to both caches simultaneously
   - Failures are logged but only return an error if both caches fail

3. **Deletes**:
   - Delete from both caches simultaneously
   - Consider successful if either cache delete succeeds

### Benefits

- **Optimal Performance**: Most reads hit the fast in-memory cache
- **Resilience**: If the in-memory cache is cleared or restarted, data can still be retrieved from Redis
- **Automatic Warming**: Frequently accessed items are automatically promoted to the fast cache
- **Configurable TTLs**: Different time-to-live settings for each cache layer

### Implementation

The `TwoTierCache` struct manages the coordination between the two caches:

```rust
pub struct TwoTierCache {
    name: String,
    fast_cache: Box<dyn DynCacheOperations>,
    slow_cache: Box<dyn DynCacheOperations>,
}
```

It implements the `CacheOperations` trait to provide the fallback strategy and the `DynCacheOperations` trait to ensure compatibility with the rest of the caching system.

### Usage

You can create a two-tier cache using the factory functions provided in the `app::cache` module:

```rust
// Create a two-tier cache with default TTLs
let cache = create_two_tier_cache("user-data", &cache_service, None, None).await?;

// Create a typed two-tier cache for a specific data type
let user_cache: Box<dyn TypedCache<User>> = 
    create_typed_two_tier_cache::<User>("users", &cache_service, 
        Some(Duration::from_secs(60)),      // 1 minute for fast cache
        Some(Duration::from_secs(3600))     // 1 hour for slow cache
    ).await?;

// For development environments without Redis
let dev_cache = create_memory_only_two_tier_cache("dev-cache", &cache_service, 
    Some(Duration::from_secs(30)),      // 30 seconds for small fast cache
    Some(Duration::from_secs(300))      // 5 minutes for larger slow cache
).await?;
```

## Configuration

The cache system can be configured in your application config:

```yaml
cache:
  default_provider: "memory"
  providers:
    - name: "memory"
      enabled: true
      default_ttl: 300  # 5 minutes
      capacity: 1000    # items
      eviction_policy: "LRU"
    - name: "redis"
      enabled: true
      default_ttl: 3600  # 1 hour
      connection_string: "redis://localhost:6379"
```

## Best Practices

1. **Choose the Right Cache**: Use two-tier cache for frequently accessed data that is expensive to compute
2. **Set Appropriate TTLs**: Fast cache TTL should be shorter than slow cache TTL
3. **Consider Memory Usage**: Set reasonable capacity limits for in-memory caches
4. **Handle Failures Gracefully**: Always have fallback logic for when cache fails
5. **Use Typed Caches**: For type safety and cleaner code, use the `TypedCache` interface

## Monitoring and Maintenance

The caching system exposes statistics and metrics for monitoring:

- Hit/miss rates for each cache layer
- Cache size and capacity
- Average operation latency
- Promotion rate (items moved from slow to fast cache)

You can access these metrics through the `stats()` method on the cache providers or through the application's metrics endpoint.

## Implementation Details

The Two-Tier Cache implementation carefully handles concurrent operations and error scenarios:

- **Concurrent Writes**: Uses `tokio::join!` to perform operations on both caches simultaneously
- **Error Handling**: Only returns errors when both caches fail
- **Promotion Logic**: Automatically promotes items from slow to fast cache on reads
- **Type Safety**: Supports type-specific caches with serialization/deserialization

## Conclusion

The Two-Tier Cache Fallback Strategy provides an optimal balance between performance and resilience. By combining a fast in-memory cache with a persistent distributed cache, applications can achieve near-memory speeds while maintaining data durability even when instances are restarted. 