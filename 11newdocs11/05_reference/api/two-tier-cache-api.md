---
title: "Two-Tier Cache API Reference"
description: "Comprehensive API reference for the Two-Tier Cache implementation in Navius"
category: reference
tags:
  - api
  - caching
  - two-tier
  - redis
  - memory-cache
  - reference
related:
  - guides/caching-strategies.md
  - reference/configuration/cache-config.md
  - reference/patterns/caching-patterns.md
  - examples/two-tier-cache-example.md
last_updated: March 27, 2025
version: 1.0
---

# Two-Tier Cache API Reference

This document provides a comprehensive reference for the Two-Tier Cache API in the Navius framework, including core interfaces, implementation details, and usage patterns.

## Table of Contents
- [Overview](#overview)
- [Core Interfaces](#core-interfaces)
- [TwoTierCache Implementation](#twotiercache-implementation)
- [Factory Functions](#factory-functions)
- [Configuration Options](#configuration-options)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)
- [Examples](#examples)

## Overview

The Two-Tier Cache implementation provides a caching solution that combines a fast in-memory cache (primary) with a distributed Redis cache (secondary). This approach offers:

- High-speed access to frequently used data
- Persistence across application restarts
- Automatic promotion of items from slow to fast cache
- Configurable time-to-live (TTL) settings for both cache levels
- Graceful handling of Redis connection issues

## Core Interfaces

### `CacheOperations` Trait

The core interface for all cache implementations:

```rust
pub trait CacheOperations: Send + Sync {
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<(), AppError>;
    async fn get(&self, key: &str) -> Result<Vec<u8>, AppError>;
    async fn delete(&self, key: &str) -> Result<(), AppError>;
    async fn clear(&self) -> Result<(), AppError>;
    async fn get_many(&self, keys: &[&str]) -> Result<HashMap<String, Vec<u8>>, AppError>;
    fn name(&self) -> &'static str;
}
```

### `DynCacheOperations` Trait

An extension of `CacheOperations` that provides typed cache access:

```rust
pub trait DynCacheOperations: CacheOperations {
    fn get_typed_cache<T: 'static>(&self) -> Box<dyn TypedCache<T>>;
}
```

### `TypedCache` Trait

A generic interface for type-safe cache operations:

```rust
pub trait TypedCache<T: 'static>: Send + Sync {
    async fn set(&self, key: &str, value: T, ttl: Option<Duration>) -> Result<(), AppError>;
    async fn get(&self, key: &str) -> Result<T, AppError>;
    async fn delete(&self, key: &str) -> Result<(), AppError>;
    async fn clear(&self) -> Result<(), AppError>;
}
```

## TwoTierCache Implementation

The `TwoTierCache` struct implements the core cache interfaces and provides the two-tier caching functionality:

```rust
pub struct TwoTierCache {
    fast_cache: Box<dyn DynCacheOperations>,
    slow_cache: Box<dyn DynCacheOperations>,
    promote_on_get: bool,
    fast_ttl: Option<Duration>,
    slow_ttl: Option<Duration>,
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `fast_cache` | `Box<dyn DynCacheOperations>` | The primary in-memory cache (typically Moka-based) |
| `slow_cache` | `Box<dyn DynCacheOperations>` | The secondary distributed cache (typically Redis-based) |
| `promote_on_get` | `bool` | Whether to promote items from slow to fast cache on cache hits |
| `fast_ttl` | `Option<Duration>` | TTL for items in the fast cache (None = use default) |
| `slow_ttl` | `Option<Duration>` | TTL for items in the slow cache (None = use default) |

### Methods

#### Constructor

```rust
pub fn new(
    fast_cache: Box<dyn DynCacheOperations>,
    slow_cache: Box<dyn DynCacheOperations>,
    promote_on_get: bool,
    fast_ttl: Option<Duration>,
    slow_ttl: Option<Duration>,
) -> Self
```

Creates a new `TwoTierCache` instance with the specified parameters.

#### Core Cache Operations

```rust
// Set a value in both fast and slow caches
async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<(), AppError>

// Get a value, trying fast cache first, then slow cache with promotion
async fn get(&self, key: &str) -> Result<Vec<u8>, AppError>

// Delete a value from both caches
async fn delete(&self, key: &str) -> Result<(), AppError>

// Clear both caches
async fn clear(&self) -> Result<(), AppError>

// Get multiple values, optimizing for batch operations
async fn get_many(&self, keys: &[&str]) -> Result<HashMap<String, Vec<u8>>, AppError>
```

#### TypedCache Support

```rust
// Get a typed cache wrapper for working with specific types
fn get_typed_cache<T: 'static>(&self) -> Box<dyn TypedCache<T>>
```

## Factory Functions

The framework provides several convenience functions for creating cache instances:

### `create_two_tier_cache`

```rust
pub async fn create_two_tier_cache(
    config: &CacheConfig,
    metrics: Option<Arc<MetricsHandler>>,
) -> Result<Arc<Box<dyn DynCacheOperations>>, AppError>
```

Creates a standard two-tier cache with:
- Fast in-memory Moka cache
- Slow Redis-based cache
- Automatic promotion from slow to fast
- Configurable TTLs for both layers

### `create_memory_only_two_tier_cache`

```rust
pub async fn create_memory_only_two_tier_cache(
    config: &CacheConfig,
    metrics: Option<Arc<MetricsHandler>>,
) -> Arc<Box<dyn DynCacheOperations>>
```

Creates a memory-only two-tier cache (for development/testing) with:
- Fast in-memory Moka cache
- Slow in-memory Moka cache (simulating Redis)
- Suitable for local development without Redis dependency

### `is_redis_available`

```rust
pub async fn is_redis_available(redis_url: &str) -> bool
```

Helper function to check if Redis is available at the specified URL.

## Configuration Options

The `CacheConfig` struct provides configuration options for the caching system:

```rust
pub struct CacheConfig {
    pub redis_url: String,
    pub redis_pool_size: usize,
    pub redis_timeout_ms: u64,
    pub redis_namespace: String,
    pub moka_max_capacity: u64,
    pub moka_time_to_live_ms: u64,
    pub moka_time_to_idle_ms: u64,
    pub redis_ttl_seconds: u64,
    pub cache_promotion: bool,
}
```

### Configuration Properties

| Property | Type | Description |
|----------|------|-------------|
| `redis_url` | `String` | URL for the Redis connection (e.g., "redis://localhost:6379") |
| `redis_pool_size` | `usize` | Number of connections in the Redis connection pool |
| `redis_timeout_ms` | `u64` | Timeout for Redis operations in milliseconds |
| `redis_namespace` | `String` | Namespace prefix for all Redis keys |
| `moka_max_capacity` | `u64` | Maximum number of items in the Moka cache |
| `moka_time_to_live_ms` | `u64` | Default TTL for Moka cache items in milliseconds |
| `moka_time_to_idle_ms` | `u64` | Time after which idle items are evicted from Moka cache |
| `redis_ttl_seconds` | `u64` | Default TTL for Redis cache items in seconds |
| `cache_promotion` | `bool` | Whether to promote items from slow to fast cache on get |

## Error Handling

The Two-Tier Cache handles the following error scenarios:

### Cache Miss
When an item is not found in either cache, a `CacheError::Miss` is returned, which is then converted to an `AppError::NotFound`.

### Redis Connection Issues
If Redis is unavailable, operations on the slow cache will fail gracefully:
- `get` operations will only use the fast cache
- `set` operations will only update the fast cache
- Error details are logged for diagnostics

### Serialization Errors
If an item cannot be serialized or deserialized, a `CacheError::Serialization` is returned, which is converted to an `AppError::BadRequest`.

## Best Practices

### Optimal Usage

1. **Choose Appropriate TTLs**
   - Fast cache TTL should be shorter than slow cache TTL
   - Critical data should have shorter TTLs to ensure freshness
   - Less critical data can have longer TTLs for performance

2. **Key Design**
   - Use consistent key naming conventions
   - Include type information in keys to prevent type confusion
   - Use namespaces to avoid key collisions between features

3. **Cache Size Management**
   - Set appropriate Moka cache size limits based on memory constraints
   - Monitor cache hit/miss ratios to optimize size

4. **Error Handling**
   - Always handle cache errors gracefully in application code
   - Implement fallback mechanisms for cache misses

### Performance Considerations

1. **Batch Operations**
   - Use `get_many` for retrieving multiple items when possible
   - Group related cache operations to reduce network overhead

2. **Serialization**
   - Use efficient serialization formats (e.g., bincode for binary data)
   - Consider compression for large objects

3. **Promotion Strategy**
   - Enable promotion for frequently accessed items
   - Disable promotion for large items that would consume significant memory

## Examples

### Basic Usage

```rust
// Create a two-tier cache
let cache_config = CacheConfig {
    redis_url: "redis://localhost:6379".to_string(),
    redis_pool_size: 10,
    redis_timeout_ms: 100,
    redis_namespace: "app:".to_string(),
    moka_max_capacity: 10_000,
    moka_time_to_live_ms: 300_000, // 5 minutes
    moka_time_to_idle_ms: 600_000, // 10 minutes
    redis_ttl_seconds: 3600, // 1 hour
    cache_promotion: true,
};

let cache = create_two_tier_cache(&cache_config, None).await?;

// Get typed cache for a specific type
let user_cache = cache.get_typed_cache::<User>();

// Store a user
let user = User { id: "user1".to_string(), name: "Alice".to_string() };
user_cache.set("user:user1", user, None).await?;

// Retrieve the user
let retrieved_user = user_cache.get("user:user1").await?;
```

### Error Handling

```rust
match user_cache.get("user:unknown").await {
    Ok(user) => {
        // Use the user
    },
    Err(err) if err.is_not_found() => {
        // Handle cache miss
        let user = fetch_user_from_database("unknown").await?;
        user_cache.set("user:unknown", user.clone(), None).await?;
        // Use the user
    },
    Err(err) => {
        // Handle other errors
        log::error!("Cache error: {}", err);
        // Fallback strategy
    }
}
```

### Custom Cache Configuration

```rust
// Create a two-tier cache with custom TTLs
let fast_ttl = Duration::from_secs(60); // 1 minute
let slow_ttl = Duration::from_secs(3600); // 1 hour

let moka_cache = create_moka_cache(&cache_config, None);
let redis_cache = create_redis_cache(&cache_config, None).await?;

let custom_cache = TwoTierCache::new(
    moka_cache,
    redis_cache,
    true, // promote_on_get
    Some(fast_ttl),
    Some(slow_ttl),
);
```

## Related Documentation

- [Caching Strategies Guide](../guides/caching-strategies.md) - Advanced caching concepts and strategies
- [Cache Configuration](../reference/configuration/cache-config.md) - Configuration options for the caching system
- [Caching Patterns](../reference/patterns/caching-patterns.md) - Common caching patterns and best practices
- [Two-Tier Cache Example](../examples/two-tier-cache-example.md) - Example implementation and usage 