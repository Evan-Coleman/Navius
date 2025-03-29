/*!
Navius Cache - Caching functionality for the Navius framework

This crate provides caching capabilities, including connection management,
cache operations, and invalidation logic for the Navius framework.
It currently supports Redis as the primary cache backend.
*/

// Internal modules
mod config;
mod connection;
mod error;
mod invalidation;
mod operations;

// Public exports
pub use config::CacheConfig;
pub use connection::CacheConnectionManager;
pub use error::{CacheError, CacheResult};
pub use invalidation::{CacheInvalidation, InvalidationStrategy};
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
