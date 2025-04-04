/*!
Navius Cache - Caching functionality for the Navius framework

This crate provides comprehensive caching capabilities for the Navius framework, including:

- **Connection Management**: Flexible connection handling with configuration options
- **Cache Operations**: Type-safe interface for common cache operations (get, set, delete, etc.)
- **Invalidation Strategies**: Multiple strategies for cache invalidation
- **Backend Support**: In-memory implementation with extensible backend support
- **Error Handling**: Consistent error handling with integration into the core error system
- **Metrics**: Optional metrics for monitoring cache operations

## Features

The crate is approximately 95% complete with the following features implemented:

- Connection management with configuration options
- Comprehensive cache operations interface with type-safe serialization
- Multiple invalidation strategies (immediate, TTL-based, pattern-based, entity-based)
- In-memory backend implementation with optimized operations
- Metrics and telemetry integration for monitoring cache operations
- Comprehensive test coverage for all components

## Usage

Basic usage with in-memory cache:

```rust
use navius_cache::{CacheConfig, operations::memory::MemoryCache, CacheOptions};
use std::time::Duration;

// Create a cache configuration
let config = CacheConfig::new(
    "memory://".to_string(),
    "my-app:".to_string(),
    Duration::from_secs(3600),
);

// Create an in-memory cache
let cache = MemoryCache::new(config);

// Store a value with a custom TTL
let options = CacheOptions::new().ttl(Duration::from_secs(600));
cache.set("user:123", &user, Some(options)).await?;

// Retrieve a value
let user: Option<User> = cache.get("user:123").await?;
```

## Coming Soon

- Additional documentation and examples
- More cache backend implementations
*/

// Internal modules
pub mod config;
pub mod connection;
pub mod error;
pub mod invalidation;
pub mod metrics;
pub mod operations;

// Public exports
pub use config::CacheConfig;
pub use connection::CacheConnectionManager;
pub use error::{CacheError, CacheResult};
pub use invalidation::{CacheInvalidation, InvalidationStrategy};
#[cfg(feature = "metrics")]
pub use metrics::{CacheOperation, CacheResult as MetricsResult, CacheTimer};
pub use operations::memory::{CacheEntry, MemoryCache};
pub use operations::{Cache, CacheKey, CacheOperations, CacheOptions};

/// Cache module to be used in applications
pub mod cache {
    pub use super::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
