// Navius Cache Redis Implementation
//
// This crate provides a Redis-specific implementation of the Navius cache system.

#![deny(missing_docs)]
#![warn(clippy::all)]

/// Redis cache configuration
pub mod config;
/// Redis cache connection management
pub mod connection;
/// Redis cache error types
pub mod error;
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
pub use connection::{ConnectionHealth, PoolStats, RedisConnectionManager};
pub use error::{RedisCacheError, RedisCacheResult};
pub use invalidation::RedisInvalidator;
pub use lua::{initialize_common_scripts, RedisLuaManager, RedisLuaScripting};
pub use metrics::{
    record_connection_acquisition, record_connection_health, record_connection_pool_stats,
    TimedOperation,
};
pub use operations::{JsonSerializer, RedisCache};
pub use pipeline::{Pipeline, RedisCommandPipeline, RedisPipelineBuilder};
