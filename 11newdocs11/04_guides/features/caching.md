---
title: Redis Caching Guide
description: "Implementation guide for basic Redis caching in Navius applications"
category: guides
tags:
  - features
  - caching
  - redis
  - performance
related:
  - ../caching-strategies.md
  - ../../reference/configuration/cache-config.md
  - ../../reference/patterns/caching-patterns.md
  - ../../examples/two-tier-cache-example.md
last_updated: March 27, 2025
version: 1.1
---

# Redis Caching Guide

This guide covers the implementation of basic Redis caching in Navius applications.

## Overview

Caching is an essential strategy for improving application performance and reducing database load. Navius provides built-in support for Redis caching, allowing you to easily implement efficient caching in your application.

## Basic Redis Caching Setup

### Installation

First, ensure Redis is installed and running:

```bash
# On macOS using Homebrew
brew install redis
brew services start redis

# On Ubuntu
sudo apt install redis-server
sudo systemctl start redis-server
```

### Configuration

Configure Redis in your application:

```yaml
# config/default.yaml
cache:
  enabled: true
  default_provider: "redis"
  providers:
    - name: "redis"
      type: "redis"
      connection_string: "redis://localhost:6379"
      ttl_seconds: 300  # 5 minutes
```

See the [Cache Configuration Reference](../../reference/configuration/cache-config.md) for a complete list of options.

### Basic Usage

Here's a simple example of using Redis caching in your application:

```rust
use navius::core::services::cache_service::{CacheService, TypedCache};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    id: String,
    name: String,
    email: String,
}

async fn cache_example(cache_service: &CacheService) -> Result<(), AppError> {
    // Create a typed cache for User objects
    let user_cache = cache_service.get_typed_cache::<User>("users").await?;
    
    // Store a user in the cache
    let user = User {
        id: "123".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    // Cache with TTL of 5 minutes
    user_cache.set("user:123", &user, Some(Duration::from_secs(300))).await?;
    
    // Retrieve from cache
    if let Some(cached_user) = user_cache.get("user:123").await? {
        println!("Found user: {:?}", cached_user);
    }
    
    Ok(())
}
```

## Cache-Aside Pattern

The most common caching pattern is the cache-aside pattern:

```rust
async fn get_user(&self, id: &str) -> Result<Option<User>, AppError> {
    let cache_key = format!("user:{}", id);
    
    // Try to get from cache first
    if let Some(user) = self.cache.get(&cache_key).await? {
        return Ok(Some(user));
    }
    
    // Not in cache, get from database
    if let Some(user) = self.repository.find_by_id(id).await? {
        // Store in cache for future requests
        self.cache.set(&cache_key, &user, Some(Duration::from_secs(300))).await?;
        return Ok(Some(user));
    }
    
    Ok(None)
}
```

For more caching patterns, see the [Caching Patterns Reference](../../reference/patterns/caching-patterns.md).

## Performance Considerations

- **TTL (Time To Live)**: Set appropriate TTL values based on how frequently your data changes
- **Cache Keys**: Use consistent and descriptive cache key formats
- **Cache Size**: Monitor memory usage in production environments
- **Cache Invalidation**: Implement proper cache invalidation strategies

## Advanced Caching

For more complex caching needs, Navius provides advanced caching options:

- **Two-Tier Caching**: Combines in-memory and Redis caching for optimal performance
- **Typed Caching**: Type-safe caching with automatic serialization/deserialization
- **Cache Eviction Policies**: Various strategies for cache eviction

For these advanced features, see the [Advanced Caching Strategies Guide](../caching-strategies.md).

## Advanced Caching Options

For more advanced caching scenarios, Navius provides a sophisticated Two-Tier Cache implementation that combines in-memory and Redis caching for optimal performance:

- [Advanced Caching Strategies](../caching-strategies.md) - Learn about the Two-Tier Cache implementation
- [Two-Tier Cache API](../../reference/api/two-tier-cache-api.md) - API reference for the Two-Tier Cache
- [Two-Tier Cache Example](../../examples/two-tier-cache-example.md) - Implementation examples

## Redis Clustering

For high-availability production environments, consider using Redis clustering:

```yaml
cache:
  providers:
    - name: "redis"
      type: "redis"
      cluster_mode: true
      nodes:
        - "redis://node1:6379"
        - "redis://node2:6379"
        - "redis://node3:6379"
```

## Monitoring

Monitor your Redis cache using:

```rust
// Get cache statistics
let stats = cache_service.stats().await?;
println!("Hit ratio: {}%", stats.hit_ratio * 100.0);
println!("Miss count: {}", stats.miss_count);
println!("Size: {} items", stats.size);
```

## Troubleshooting

Common issues and solutions:

- **Connection Failures**: Check Redis server status and connection settings
- **Serialization Errors**: Ensure all cached objects implement Serialize/Deserialize
- **Memory Issues**: Configure maxmemory and eviction policies in Redis configuration
- **Slow Performance**: Consider using the Two-Tier Cache implementation for improved performance

## Next Steps

- Explore [Advanced Caching Strategies](../caching-strategies.md) for implementing two-tier caching
- Check the [Two-Tier Cache Example](../../examples/two-tier-cache-example.md) for implementation details
- Review the [Caching Patterns](../../reference/patterns/caching-patterns.md) reference for best practices
- Configure your cache using the [Cache Configuration Reference](../../reference/configuration/cache-config.md)

## Related Resources

- [Redis Documentation](https://redis.io/documentation)
- [Caching Best Practices](https://aws.amazon.com/caching/best-practices/)
- [Navius Performance Guide](../performance-optimization.md)
