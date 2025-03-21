pub mod cache_manager;
pub mod registry_stats;

// Re-export main types and functions from cache_manager
pub use cache_manager::{
    CacheRegistry, CacheStats, ResourceCache, get_cache_stats_with_metrics, get_or_fetch,
    get_resource_cache, init_cache_registry, register_resource_cache, start_metrics_updater,
};

// Re-export from registry_stats
pub use registry_stats::get_all_cache_stats_with_metrics;

// Re-export all core cache functionality
pub use crate::core::cache::*;

// Import and re-export provider modules
pub mod providers;
pub mod registry_stats;
pub use providers::*;

/// User-defined cache adapters and extensions
pub mod registry_stats;

// Default to in-memory cache but allow users to implement their own providers
pub use providers::memory::MemoryCacheProvider as DefaultCacheProvider;

// Export named providers for clarity
pub use providers::fallback::FallbackCacheProvider;
pub use providers::memory::MemoryCacheProvider;
pub use providers::redis::RedisCacheProvider;
