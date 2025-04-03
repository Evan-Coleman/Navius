//! Redis implementation of the Navius cache interface
//!
//! This module provides a Redis cache implementation

// Temporary allowances during development
#![allow(missing_docs)]
#![deny(unsafe_code)]
#![allow(incomplete_features)]
#![allow(unused_imports)]

// Re-export the Redis cache
pub mod config;
pub mod connection;
pub mod error;
pub mod key;
pub mod lua;
pub mod metrics;
pub mod operations;
pub mod redis_cache;
pub mod serialization;

// Declare the new modules
pub mod redis_basic_ops;
pub mod redis_hash_ops;
pub mod redis_list_ops;
pub mod redis_set_ops;

// Re-export the key types
pub use config::RedisCacheConfig;
pub use error::{RedisCacheError, RedisCacheResult};
pub use redis_cache::{RedisCache, check_redis_connection};

// Re-export the cache interface
pub use navius_cache::{Cache, CacheOperations, CacheOptions, CacheResult};

/// Create a new Redis cache with the given configuration
///
/// # Arguments
///
/// * `config` - The Redis cache configuration
///
/// # Returns
///
/// A new `RedisCache` instance wrapped in a `Result`
///
/// # Errors
///
/// Returns an error if the Redis server is not available or if the configuration is invalid
pub async fn new(config: RedisCacheConfig) -> Result<RedisCache, RedisCacheError> {
    redis_cache::RedisCache::new(config).await
}

/// Check if a Redis server is available at the given URL
///
/// # Arguments
///
/// * `url` - The Redis URL to check
///
/// # Returns
///
/// `true` if the Redis server is available, `false` otherwise
pub async fn check(url: &str) -> bool {
    redis_cache::check_redis_connection(url).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
