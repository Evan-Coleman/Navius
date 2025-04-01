// Navius Cache Redis Implementation
//
// This crate provides a Redis-specific implementation of the Navius cache system.

#![deny(missing_docs)]
#![warn(clippy::all)]

/// Redis cache configuration
pub mod config;
/// Redis cache invalidation
pub mod invalidation;
/// Redis cache lua scripting
pub mod lua;
/// Redis cache metrics
pub mod metrics;
/// Redis cache operations
pub mod operations;
/// Redis cache pipelining
pub mod pipeline;

// Re-export important types
pub use config::RedisCacheConfig;
pub use invalidation::RedisInvalidator;
pub use lua::{RedisLuaManager, RedisLuaScripting, initialize_common_scripts};
pub use metrics::{
    TimedOperation, record_connection_acquisition, record_connection_health,
    record_connection_pool_stats,
};
pub use operations::RedisCache;
pub use pipeline::{Pipeline, RedisPipeline};

// Error types
pub mod error;
pub use error::{RedisCacheError, RedisCacheResult};

// Connection management
pub mod connection;
pub use connection::{ConnectionHealth, PoolStats, RedisConnectionManager};

use navius_cache::{
    CacheKey,
    CacheOperations,
    CacheOptions,
    error::{CacheError, CacheResult},
};

// Remove stricter requirements where V: Send + Sync
async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
where
    K: CacheKey + 'static,
    V: DeserializeOwned + 'static,
{
    // ... existing code ...
}
