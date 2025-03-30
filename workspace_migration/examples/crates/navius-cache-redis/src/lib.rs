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
/// Redis cache operations
pub mod operations;
/// Redis cache pipelining
pub mod pipeline;

// Re-export important types
pub use config::RedisCacheConfig;
pub use invalidation::RedisInvalidator;
pub use lua::{RedisLuaManager, RedisLuaScripting, initialize_common_scripts};
pub use operations::RedisCache;
pub use pipeline::{Pipeline, RedisPipeline};

// Error types
pub mod error;
pub use error::{RedisCacheError, RedisCacheResult};

// Connection management
pub mod connection;
pub use connection::{ConnectionHealth, PoolStats, RedisConnectionManager};
