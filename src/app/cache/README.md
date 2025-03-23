# Application Caching

This directory contains user-facing caching functionality for the Navius application. Use this module to extend and customize the core caching system for your specific needs.

## Usage

To use caching in your application code:

```rust
use crate::app::cache;
use crate::core::error::Result;

async fn example_caching() -> Result<()> {
    // Create a cache for a specific use case
    let user_cache = cache::create_user_cache()?;
    
    // Cache operations
    user_cache.set("user:123", serialize_user(user).await?).await?;
    let user_data = user_cache.get("user:123").await?;
    
    // Use core cache manager directly
    let product_cache = cache::cache_manager::create_cache::<String, Vec<u8>>(
        "product_cache",
        cache::cache_manager::CacheProvider::Memory,
        Some(std::time::Duration::from_secs(1800)),
    )?;
    
    Ok(())
}
```

## Extending Caching Functionality

### Creating Custom Cache Providers

You can create custom cache providers that implement the `Cache` trait:

```rust
// src/app/cache/providers/two_tier_cache.rs
use crate::core::cache::{Cache, CacheError};
use crate::core::error::Result;
use async_trait::async_trait;
use std::marker::PhantomData;
use std::time::Duration;

pub struct TwoTierCache<K, V> {
    name: String,
    fast_cache: Box<dyn Cache<K, V>>,
    slow_cache: Box<dyn Cache<K, V>>,
    _marker: PhantomData<(K, V)>,
}

impl<K, V> TwoTierCache<K, V> 
where
    K: Send + Sync + Clone + 'static,
    V: Send + Sync + Clone + 'static,
{
    pub fn new(
        name: &str,
        fast_cache: Box<dyn Cache<K, V>>,
        slow_cache: Box<dyn Cache<K, V>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            fast_cache,
            slow_cache,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for TwoTierCache<K, V>
where
    K: Send + Sync + Clone + 'static,
    V: Send + Sync + Clone + 'static,
{
    async fn get(&self, key: K) -> Result<Option<V>> {
        // Try fast cache first
        if let Some(value) = self.fast_cache.get(key.clone()).await? {
            return Ok(Some(value));
        }
        
        // If not in fast cache, try slow cache
        if let Some(value) = self.slow_cache.get(key.clone()).await? {
            // Promote to fast cache
            self.fast_cache.set(key, value.clone()).await?;
            return Ok(Some(value));
        }
        
        Ok(None)
    }
    
    async fn set(&self, key: K, value: V) -> Result<()> {
        // Set in both caches
        self.fast_cache.set(key.clone(), value.clone()).await?;
        self.slow_cache.set(key, value).await?;
        Ok(())
    }
    
    // Implement other methods...
}
```

### Creating Custom Cache Factories

Create factory functions for your custom cache implementations:

```rust
// In src/app/cache/mod.rs
pub fn create_two_tier_cache<K, V>(
    name: &str,
    ttl: Option<Duration>,
) -> Result<Box<dyn Cache<K, V>>>
where
    K: Send + Sync + Clone + 'static,
    V: Send + Sync + Clone + 'static,
{
    let fast_cache = cache_manager::create_cache::<K, V>(
        &format!("{}_fast", name),
        cache_manager::CacheProvider::Memory,
        Some(Duration::from_secs(300)), // 5 minutes
    )?;
    
    let slow_cache = cache_manager::create_cache::<K, V>(
        &format!("{}_slow", name),
        cache_manager::CacheProvider::Redis,
        ttl.or(Some(Duration::from_secs(3600))), // 1 hour default
    )?;
    
    Ok(Box::new(providers::two_tier_cache::TwoTierCache::new(
        name,
        fast_cache,
        slow_cache,
    )))
}
```

## Best Practices

1. Choose appropriate cache TTLs based on data volatility
2. Use meaningful cache names for better monitoring
3. Consider cache invalidation strategies
4. Handle cache failures gracefully
5. Add appropriate logging for cache operations
6. Consider cache warming for critical data
7. Monitor cache hit/miss rates
8. Use serialization for complex objects
9. Consider memory usage when caching large objects

## Core Caching System

The core caching system is provided by `crate::core::cache` and includes:

- Memory-based caching with Moka
- Redis-based caching
- Fallback caching strategies
- Cache statistics and monitoring
- Base Cache trait and implementations

Do not modify the core caching system directly. Instead, use this directory to extend and customize caching for your specific application needs. 