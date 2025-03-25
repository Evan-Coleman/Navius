//! Caching functionality
//!
//! This module provides caching implementations and management:
//! - Cache registry for tracking cached resources
//! - Statistics collection and reporting
//! - Cache eviction policies

pub mod cache_manager;
pub mod registry_stats;

// Re-export main types and functions
pub use cache_manager::{
    CacheRegistry, CacheStats, ResourceCache, get_cache_stats_with_metrics, get_or_fetch,
    get_resource_cache, init_cache_registry, last_fetch_from_cache, register_resource_cache,
    start_metrics_updater,
};
pub use registry_stats::get_all_cache_stats_with_metrics;
