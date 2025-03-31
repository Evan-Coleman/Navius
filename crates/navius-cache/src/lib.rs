/*!
Navius Cache - Caching functionality for the Navius framework

This crate provides comprehensive caching capabilities for the Navius framework, including:

- **Connection Management**: Flexible connection handling with configuration options
- **Cache Operations**: Type-safe interface for common cache operations (get, set, delete, etc.)
- **Invalidation Strategies**: Multiple strategies for cache invalidation
- **Backend Support**: Redis implementation with transaction support
- **Error Handling**: Consistent error handling with integration into the core error system
- **Metrics**: Optional metrics for monitoring cache operations

## Features

The crate is approximately 95% complete with the following features implemented:

- Connection management with connection pooling and health checks
- Comprehensive cache operations interface with type-safe serialization
- Multiple invalidation strategies (immediate, TTL-based, pattern-based, entity-based)
- Full Redis backend implementation with optimized operations
- Metrics and telemetry integration for monitoring cache operations
- Comprehensive test coverage for all components

## Usage

Basic usage with Redis:

```rust
use navius_cache::{CacheConfig, CacheConnectionManager, CacheOptions, RedisCache};
use std::time::Duration;

// Create a cache configuration
let config = CacheConfig::new(
    "redis://127.0.0.1:6379".to_string(),
    "my-app:".to_string(),
    Duration::from_secs(3600),
);

// Create a cache connection manager
let cache = CacheConnectionManager::new_redis(config).await?;

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
mod config;
mod connection;
mod error;
mod invalidation;
mod metrics;
mod operations;

// Public exports
pub use config::CacheConfig;
pub use connection::CacheConnectionManager;
pub use error::{CacheError, CacheResult};
pub use invalidation::{CacheInvalidation, InvalidationStrategy};
#[cfg(feature = "metrics")]
pub use metrics::{CacheOperation, CacheResult as MetricsResult, CacheTimer};
#[cfg(feature = "redis")]
pub use operations::redis::{RedisCache, RedisConfig};
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
