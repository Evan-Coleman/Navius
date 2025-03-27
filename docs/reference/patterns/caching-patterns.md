---
title: "Caching Patterns in Navius"
description: "Technical reference documentation for common caching patterns used throughout the Navius framework"
category: reference
tags:
  - reference
  - patterns
  - caching
  - performance
  - best-practices
related:
  - ../../guides/caching-strategies.md
  - ../configuration/cache-config.md
  - ../../examples/two-tier-cache-example.md
last_updated: March 26, 2024
version: 1.0
---

# Caching Patterns in Navius

This reference document describes the common caching patterns used throughout the Navius framework and how to implement them effectively.

## Core Caching Patterns

### Cache-Aside Pattern

The most common caching pattern used in Navius is the Cache-Aside (Lazy Loading) pattern:

```rust
async fn get_data(&self, key: &str) -> Result<Data, Error> {
    // Try to get from cache first
    if let Some(cached_data) = self.cache.get(key).await? {
        return Ok(cached_data);
    }
    
    // Not in cache, fetch from the source
    let data = self.data_source.fetch(key).await?;
    
    // Store in cache for future requests
    self.cache.set(key, &data, Some(Duration::from_secs(300))).await?;
    
    Ok(data)
}
```

This pattern:
- Checks the cache first
- Falls back to the source only if needed
- Updates the cache with the fetched data
- Returns the data to the client

### Write-Through Cache

For data consistency when writing, Navius recommends the Write-Through pattern:

```rust
async fn save_data(&self, key: &str, data: &Data) -> Result<(), Error> {
    // Write to the source first
    self.data_source.save(key, data).await?;
    
    // Then update the cache
    self.cache.set(key, data, Some(Duration::from_secs(300))).await?;
    
    Ok(())
}
```

This pattern:
- Ensures data is safely stored in the primary source first
- Updates the cache to maintain consistency
- Provides fast reads while ensuring write durability

### Cache Invalidation

For invalidation, Navius uses the direct invalidation pattern:

```rust
async fn delete_data(&self, key: &str) -> Result<(), Error> {
    // Delete from the source
    self.data_source.delete(key).await?;
    
    // Invalidate cache
    self.cache.delete(key).await?;
    
    Ok(())
}
```

## Multi-Level Caching

### Two-Tier Cache

Navius implements the Two-Tier Cache pattern for optimal performance:

```rust
// Two-tier cache implementation
pub struct TwoTierCache {
    fast_cache: Box<dyn CacheOperations>, // In-memory cache
    slow_cache: Box<dyn CacheOperations>, // Redis or other distributed cache
}

impl CacheOperations for TwoTierCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Try fast cache first
        if let Some(data) = self.fast_cache.get(key).await? {
            return Ok(Some(data));
        }
        
        // Then try slow cache
        if let Some(data) = self.slow_cache.get(key).await? {
            // Promote to fast cache
            self.fast_cache.set(key, &data, None).await?;
            return Ok(Some(data));
        }
        
        Ok(None)
    }
}
```

This pattern:
- Provides extremely fast access for frequently used data
- Maintains resiliency through the slower but more durable cache
- Automatically promotes items to the faster cache when accessed

## Specialized Caching Patterns

### Time-Based Expiration

For caches that need automatic expiration:

```rust
async fn get_with_ttl(&self, key: &str, ttl: Duration) -> Result<Option<Data>, Error> {
    let data = self.cache.get(key).await?;
    
    // Set with TTL when storing
    self.cache.set(key, &data, Some(ttl)).await?;
    
    Ok(data)
}
```

### Collection Caching

For caching collections of items:

```rust
async fn get_collection(&self, collection_key: &str) -> Result<Vec<Item>, Error> {
    // Try to get collection from cache
    if let Some(items) = self.cache.get(collection_key).await? {
        return Ok(items);
    }
    
    // Fetch collection
    let items = self.repository.get_all().await?;
    
    // Cache the collection
    self.cache.set(collection_key, &items, Some(Duration::from_secs(60))).await?;
    
    // Optionally, cache individual items too
    for item in &items {
        let item_key = format!("item:{}", item.id);
        self.cache.set(&item_key, item, Some(Duration::from_secs(300))).await?;
    }
    
    Ok(items)
}
```

### Read-Through Cache

For simplicity in some scenarios:

```rust
// Read-through cache implementation
pub struct ReadThroughCache<T> {
    cache: Box<dyn CacheOperations>,
    data_source: Arc<dyn DataSource<T>>,
}

impl<T> ReadThroughCache<T> {
    async fn get(&self, key: &str) -> Result<Option<T>, Error> {
        // Try to get from cache
        if let Some(data) = self.cache.get(key).await? {
            return Ok(Some(data));
        }
        
        // Not in cache, fetch from source
        let data = self.data_source.get(key).await?;
        
        if let Some(data_ref) = &data {
            // Store in cache for future
            self.cache.set(key, data_ref, None).await?;
        }
        
        Ok(data)
    }
}
```

## Cache Eviction Strategies

Navius caching system supports various eviction strategies:

1. **LRU (Least Recently Used)**: Evicts the least recently accessed items first
2. **LFU (Least Frequently Used)**: Evicts the least frequently accessed items first
3. **FIFO (First In First Out)**: Evicts the oldest entries first
4. **TTL (Time To Live)**: Evicts entries that have expired based on a set duration

Example configuration:

```yaml
cache:
  providers:
    - name: "memory"
      eviction_policy: "LRU"  # Options: LRU, LFU, FIFO, TTL
      capacity: 10000
```

## Cache Key Design Patterns

### Prefix-Based Keys

Navius recommends prefix-based keys for organization:

```rust
// User-related keys
let user_key = format!("user:{}", user_id);
let user_prefs_key = format!("user:{}:preferences", user_id);
let user_sessions_key = format!("user:{}:sessions", user_id);

// Content-related keys
let content_key = format!("content:{}", content_id);
let content_views_key = format!("content:{}:views", content_id);
```

### Composite Keys

For complex lookups:

```rust
// Composite key for filtered search results
let search_key = format!("search:{}:filter:{}:page:{}", 
    query, filter_hash, page_number);
```

## Cache Implementation Pattern

Navius follows this pattern for integrating caches with services:

```rust
pub struct EntityService<T> {
    cache: Arc<dyn TypedCache<T>>,
    repository: Arc<dyn EntityRepository<T>>,
}

impl<T> EntityService<T> {
    // Constructor with cache
    pub fn new(cache: Arc<dyn TypedCache<T>>, repository: Arc<dyn EntityRepository<T>>) -> Self {
        Self { cache, repository }
    }
    
    // Get entity by ID with caching
    pub async fn get_by_id(&self, id: &str) -> Result<Option<T>, Error> {
        let cache_key = format!("entity:{}", id);
        
        // Try cache first
        if let Some(entity) = self.cache.get(&cache_key).await? {
            return Ok(Some(entity));
        }
        
        // Not in cache, get from repository
        if let Some(entity) = self.repository.find_by_id(id).await? {
            // Cache for future
            self.cache.set(&cache_key, &entity, None).await?;
            return Ok(Some(entity));
        }
        
        Ok(None)
    }
}
```

## Best Practices

1. **Cache Appropriate Data**: Cache data that is:
   - Frequently accessed
   - Expensive to compute or retrieve
   - Relatively static (doesn't change often)

2. **Set Appropriate TTLs**: Consider:
   - How frequently data changes
   - Tolerance for stale data
   - Memory constraints

3. **Cache Consistency**:
   - Update or invalidate cache entries when the source data changes
   - Consider using event-based cache invalidation for distributed systems

4. **Error Handling**:
   - Treat cache failures as non-critical
   - Implement fallbacks when cache is unavailable
   - Log cache errors but continue operation

5. **Monitoring**:
   - Track cache hit/miss ratios
   - Monitor memory usage
   - Set alerts for unusual patterns

## Anti-Patterns to Avoid

1. **Caching Everything**: Not all data benefits from caching
2. **No Eviction Policy**: Always implement some form of eviction
3. **Unbounded Cache Growth**: Set capacity limits
4. **Cache Stampede**: Use techniques like request coalescing to prevent multiple identical fetches
5. **Ignoring Cache Errors**: Implement proper error handling

## Related Components

- [Two-Tier Cache Implementation](../../guides/caching-strategies.md#two-tier-cache-fallback-strategy)
- [Cache Provider Interface](../../guides/caching-strategies.md#cache-provider-interface)
- [Caching Configuration](../../guides/caching-strategies.md#configuration) 