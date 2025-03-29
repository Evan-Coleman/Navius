use crate::error::CacheResult;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Display;
use std::time::Duration;

/// Cache key trait for converting types to cache keys
pub trait CacheKey: Display + Send + Sync {
    /// Convert the value to a string representation
    fn to_string(&self) -> String;
}

// Implement CacheKey for String
impl CacheKey for String {
    fn to_string(&self) -> String {
        self.clone()
    }
}

// Implement CacheKey for &str
impl CacheKey for &str {
    fn to_string(&self) -> String {
        self.to_string()
    }
}

/// Cache options for controlling cache behavior
#[derive(Debug, Clone)]
pub struct CacheOptions {
    /// Time to live for cache entries
    pub ttl: Option<Duration>,
}

impl Default for CacheOptions {
    fn default() -> Self {
        Self { ttl: None }
    }
}

impl CacheOptions {
    /// Create new cache options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the TTL
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

/// Cache operations trait for interacting with the cache
#[async_trait]
pub trait CacheOperations: Send + Sync + 'static {
    /// Get a value from the cache
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Get multiple values from the cache
    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + 'static,
        V: DeserializeOwned + 'static;

    /// Set a value in the cache
    async fn set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Set multiple values in the cache
    async fn set_many<K, V>(
        &self,
        entries: Vec<(K, V)>,
        options: Option<CacheOptions>,
    ) -> CacheResult<()>
    where
        K: CacheKey + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Delete a value from the cache
    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static;

    /// Delete multiple values from the cache
    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + 'static;

    /// Check if a key exists in the cache
    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + 'static;

    /// Increment a counter in the cache
    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static;

    /// Expire a key in the cache
    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + 'static;

    /// Clear the entire cache
    async fn clear(&self) -> CacheResult<()>;

    /// Get the health status of the cache
    async fn health_check(&self) -> CacheResult<()>;
}

/// Cache trait combining cache operations and cloning
pub trait Cache: CacheOperations + Clone {}

// Export Redis module
#[cfg(feature = "redis")]
pub mod redis;
