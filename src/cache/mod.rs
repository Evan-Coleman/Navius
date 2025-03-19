pub mod cache_manager;
pub mod registry_stats;

// Re-export main types and functions from cache_manager
pub use cache_manager::{
    CacheRegistry, CacheStats, ResourceCache, get_cache_stats_with_metrics, get_or_fetch,
    get_resource_cache, init_cache_registry, register_resource_cache, start_metrics_updater,
};

// Re-export from registry_stats
pub use registry_stats::get_all_cache_stats_with_metrics;
