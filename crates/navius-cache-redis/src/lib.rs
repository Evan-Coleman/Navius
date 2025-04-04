//! Redis implementation of the Navius cache interface
//!
//! This module provides a Redis cache implementation of the Navius cache interface.
//! It offers full support for Redis operations including basic operations, hash, list,
//! set, and sorted set (zset) operations.
//!
//! ## Type Requirements
//!
//! All Redis cache operations have specific trait bounds requirements:
//!
//! - `K: CacheKey + 'static`: Keys must implement the `CacheKey` trait
//! - `V: Serialize + Send + Sync + 'static`: Values must be serializable, thread-safe, and have a static lifetime
//! - `V: DeserializeOwned + Send + 'static`: Values retrieved from cache must be deserializable without borrowing
//!
//! Most operations have a `Debug` bound on keys for better error reporting.
//!
//! ## Redis Data Types
//!
//! This implementation supports all Redis data structures:
//!
//! - **Basic operations**: get, set, delete, exists, expire, increment
//! - **Hash operations**: hash_get, hash_set, hash_delete, hash_get_all, etc.
//! - **List operations**: list_push_left, list_push_right, list_pop_left, etc.
//! - **Set operations**: set_add, set_remove, set_contains, set_members, etc.
//! - **Sorted Set operations**: zset_add, zset_remove, zset_score, zset_range, etc.

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
///
/// # Example
///
/// ```
/// # use navius_cache_redis::{new, RedisCacheConfig};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = RedisCacheConfig::new("redis://localhost:6379");
/// let cache = new(config).await?;
/// # Ok(())
/// # }
/// ```
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
///
/// # Example
///
/// ```
/// # use navius_cache_redis::check;
/// # async fn example() {
/// let is_available = check("redis://localhost:6379").await;
/// # }
/// ```
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
