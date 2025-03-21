pub mod fallback;
pub mod memory;
pub mod redis;

use crate::utils::api_resource::ApiResource;
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

/// Cache provider interface for implementing different caching backends
#[async_trait]
pub trait CacheProvider: Send + Sync + 'static {
    /// Initialize the cache provider with configuration
    fn init(&self) -> Result<(), String>;

    /// Store a value in the cache
    async fn set<T: ApiResource>(
        &self,
        key: &str,
        value: T,
        ttl_seconds: u64,
    ) -> Result<(), String>;

    /// Retrieve a value from the cache
    async fn get<T: ApiResource>(&self, key: &str) -> Result<Option<T>, String>;

    /// Remove a value from the cache
    async fn delete(&self, key: &str) -> Result<(), String>;

    /// Clear all values from the cache
    async fn clear(&self) -> Result<(), String>;

    /// Check if a key exists in the cache
    async fn exists(&self, key: &str) -> Result<bool, String>;

    /// Get cache statistics
    async fn get_stats(&self) -> Result<serde_json::Value, String>;
}
