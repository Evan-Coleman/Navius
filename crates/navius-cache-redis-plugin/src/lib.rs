/*!
Redis Cache Plugin for Navius

This crate provides a Redis implementation of the Navius cache system
using the well-established redis-rs crate. It integrates with the
navius-cache interfaces to provide a Redis-backed cache that can be
used with the Navius plugin system.

## Features

- Complete implementation of the CacheOperations trait with Redis
- Optimized operations using Redis's pipelining and batching
- Support for all collection types (lists, sets, hashes, sorted sets)
- Connection pooling and automatic reconnection
- TLS support options
- Optional Redis Cluster support
- Support for Redis transactions and Lua scripting

## Usage

```rust
use navius_cache::{CacheConfig, Cache};
use navius_cache_redis_plugin::{RedisCache, RedisCacheConfig};

// Create a Redis cache configuration
let config = RedisCacheConfig::new(
    "redis://127.0.0.1:6379".to_string(),
    "my-app:".to_string(),
    std::time::Duration::from_secs(3600),
);

// Create a Redis cache instance
let cache = RedisCache::new(config).await?;

// Use with standard cache operations
cache.set("user:123", &user, None).await?;
let user: Option<User> = cache.get("user:123").await?;
```

## Optional Features

- `cluster` - Enable Redis Cluster support
- `json` - Enable RedisJSON support
- `pubsub` - Enable Redis PubSub capabilities
- `tls-rustls` - Enable TLS support using RusTLS
- `tls-native` - Enable TLS support using native TLS
*/

mod config;
mod connection;
mod error;
mod operations;
mod plugin;
mod util;

// Public exports
pub use config::RedisCacheConfig;
pub use error::{RedisError, RedisResult};
pub use operations::RedisCache;
pub use plugin::RedisCachePlugin;

// Re-export from redis crate for convenience
pub use redis;

/// Redis Plugin module to be used in applications
pub mod redis_plugin {
    pub use super::*;
}

// Tests
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
