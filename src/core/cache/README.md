# Core Cache System

This directory contains the core implementation of the caching system used by the application. It provides a generic in-memory caching mechanism based on the [Moka](https://github.com/moka-rs/moka) library.

## Components

- `cache_manager.rs`: Main implementation of the caching system
- `registry_stats.rs`: Functions for retrieving cache statistics
- `mod.rs`: Module definitions and exports

## Design

The caching system is designed around these key components:

1. `ResourceCache<T>`: A generic cache for any resource type that implements the `ApiResource` trait
2. `CacheRegistry`: A registry that stores caches for different resource types
3. `CacheStats`: Statistics about a specific cache

## Features

- Generic caching for any type that implements `ApiResource`
- TTL-based cache expiration
- Thread-safe concurrency with Arc/RwLock
- Metrics tracking for hits, misses, evictions
- Cache statistics
- Support for async operations

## Usage

The core cache is not meant to be used directly by application code. Instead, use the application-level cache module which provides a more user-friendly interface and allows for different cache providers.

If you need to interact with the core cache system, use the following approach:

```rust
use crate::core::cache::{CacheRegistry, init_cache_registry, register_resource_cache, get_or_fetch};

// Initialize the cache registry
let registry = init_cache_registry(true, 10000, 3600);

// Register a cache for a specific resource type
register_resource_cache::<MyResource>(&registry, "my_resource");

// Get or fetch a resource
let result = get_or_fetch(
    &registry, 
    "my_resource", 
    "resource_id", 
    || async { /* fetch the resource if not in cache */ }
).await;
```

## Implementation Details

The core caching system includes the following features:

- **Automatic TTL**: Resources are automatically removed from the cache after their TTL expires
- **Metrics**: Cache hits, misses, and other statistics are tracked and exposed through metrics
- **Eviction Listener**: A listener that updates metrics when resources are evicted from the cache
- **Thread Safety**: The cache is thread-safe and can be used from multiple threads concurrently
- **Async Support**: All operations are async-compatible
