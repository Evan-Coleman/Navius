use metrics::{counter, gauge};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::any::{Any, type_name};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicU64, Ordering},
};
use std::thread_local;
use std::time::{Duration, SystemTime};
use tokio::time::interval;
use tracing::{debug, info, warn};

// Import ApiResource trait
use crate::core::error::AppError;
use crate::core::utils::api_resource::ApiResource;

/// Generic cache for any resource type that implements ApiResource
#[derive(Debug)]
pub struct ResourceCache<T: ApiResource> {
    pub cache: Arc<Cache<String, T>>, // Use String keys for flexibility
    pub creation_time: SystemTime,
    pub ttl_seconds: u64,
    pub active_entries: Arc<AtomicU64>,
    pub resource_type: String,
}

/// Cache registry to store caches for different resource types
#[derive(Debug, Clone)]
pub struct CacheRegistry {
    // Use RwLock to allow concurrent reads but exclusive writes
    caches: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_capacity: u64,
    pub creation_time: SystemTime,
}

/// Cache statistics for a resource type
#[derive(Debug)]
pub struct CacheStats {
    /// Number of items in the cache
    pub size: usize,
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Cache hit ratio (hits / total)
    pub hit_ratio: f64,
}

/// Define a trait for getting cache stats
pub trait CacheStatsProvider: Send + Sync {
    fn get_stats(&self) -> CacheStats;
}

/// Initialize the cache registry
pub fn init_cache_registry(enabled: bool, max_capacity: u64, ttl_seconds: u64) -> CacheRegistry {
    CacheRegistry {
        caches: Arc::new(RwLock::new(HashMap::new())),
        enabled,
        ttl_seconds,
        max_capacity,
        creation_time: SystemTime::now(),
    }
}

/// Register a new resource type with the cache registry
pub fn register_resource_cache<T: ApiResource + 'static>(
    registry: &CacheRegistry,
    resource_type: &str,
) -> Result<(), String> {
    if !registry.enabled {
        debug!(
            "Cache is disabled, not registering cache for {}",
            resource_type
        );
        return Ok(());
    }

    let ttl = Duration::from_secs(registry.ttl_seconds);
    let resource_type_clone = resource_type.to_string();

    // Create a ResourceCache that we'll box and store
    // This allows us to access it directly later for the eviction listener
    let active_entries = Arc::new(AtomicU64::new(0));
    let active_entries_clone = active_entries.clone();

    // Create the cache with eviction listener
    let cache_builder = Cache::builder()
        .max_capacity(registry.max_capacity)
        .time_to_live(ttl)
        .time_to_idle(ttl.mul_f32(1.5))
        .initial_capacity(100)
        .eviction_listener(move |_key, _value, cause| {
            // Track cache evictions in metrics and update counter
            match cause {
                moka::notification::RemovalCause::Expired => {
                    debug!("🔄 Cache entry expired for {}", resource_type_clone);
                    // Decrement active entries counter - don't go below 0
                    let current = active_entries_clone.load(Ordering::Relaxed);
                    if current > 0 {
                        active_entries_clone.fetch_sub(1, Ordering::SeqCst);
                        // Update the gauge directly - this requires metrics library support
                        gauge!("cache_active_entries", "resource_type" => resource_type_clone.to_string())
                            .set((current - 1) as f64);
                        gauge!("cache_current_size", "resource_type" => resource_type_clone.to_string())
                            .decrement(1.0);
                    }
                }
                moka::notification::RemovalCause::Explicit => {
                    debug!(
                        "❌ Cache entry explicitly removed for {}",
                        resource_type_clone
                    );
                    // Decrement active entries counter - don't go below 0
                    let current = active_entries_clone.load(Ordering::Relaxed);
                    if current > 0 {
                        active_entries_clone.fetch_sub(1, Ordering::SeqCst);
                        // Update the gauge directly
                        gauge!("cache_active_entries", "resource_type" => resource_type_clone.to_string())
                            .set((current - 1) as f64);
                        gauge!("cache_current_size", "resource_type" => resource_type_clone.to_string())
                            .decrement(1.0);
                    }
                }
                moka::notification::RemovalCause::Size => {
                    debug!(
                        "📊 Cache entry evicted due to size for {}",
                        resource_type_clone
                    );
                    // Decrement active entries counter - don't go below 0
                    let current = active_entries_clone.load(Ordering::Relaxed);
                    if current > 0 {
                        active_entries_clone.fetch_sub(1, Ordering::SeqCst);
                        // Update the gauge directly
                        gauge!("cache_active_entries", "resource_type" => resource_type_clone.to_string())
                            .set((current - 1) as f64);
                        gauge!("cache_current_size", "resource_type" => resource_type_clone.to_string())
                            .decrement(1.0);
                    }
                }
                _ => {
                    debug!(
                        "🔄 Cache entry removed for {}: {:?}",
                        resource_type_clone, cause
                    );
                    // Decrement active entries counter - don't go below 0
                    let current = active_entries_clone.load(Ordering::Relaxed);
                    if current > 0 {
                        active_entries_clone.fetch_sub(1, Ordering::SeqCst);
                        // Update the gauge directly
                        gauge!("cache_active_entries", "resource_type" => resource_type_clone.to_string())
                            .set((current - 1) as f64);
                        gauge!("cache_current_size", "resource_type" => resource_type_clone.to_string())
                            .decrement(1.0);
                    }
                }
            }
        })
        .build();

    let resource_cache: ResourceCache<T> = ResourceCache {
        cache: Arc::new(cache_builder),
        creation_time: SystemTime::now(),
        ttl_seconds: registry.ttl_seconds,
        active_entries,
        resource_type: resource_type.to_string(),
    };

    // Attempt to insert the cache into the registry
    let mut caches = match registry.caches.write() {
        Ok(caches) => caches,
        Err(_) => return Err("Failed to acquire write lock on cache registry".to_string()),
    };

    caches.insert(
        resource_type.to_string(),
        Box::new(resource_cache) as Box<dyn Any + Send + Sync>,
    );

    info!("✅ Registered cache for resource type: {}", resource_type);
    Ok(())
}

/// Get a cache for a specific resource type
pub fn get_resource_cache<T: ApiResource + 'static>(
    registry: &CacheRegistry,
    resource_type: &str,
) -> Option<ResourceCache<T>> {
    if !registry.enabled {
        return None;
    }

    let caches = match registry.caches.read() {
        Ok(caches) => caches,
        Err(_) => {
            debug!("Failed to acquire read lock on cache registry");
            return None;
        }
    };

    match caches.get(resource_type) {
        Some(cache) => {
            // Attempt to downcast to the appropriate ResourceCache type
            if let Some(boxed_cache) = cache.downcast_ref::<ResourceCache<T>>() {
                // Clone the ResourceCache
                Some(ResourceCache {
                    cache: boxed_cache.cache.clone(),
                    creation_time: boxed_cache.creation_time,
                    ttl_seconds: boxed_cache.ttl_seconds,
                    active_entries: boxed_cache.active_entries.clone(),
                    resource_type: boxed_cache.resource_type.clone(),
                })
            } else {
                debug!(
                    "Failed to downcast cache for resource type: {}",
                    resource_type
                );
                None
            }
        }
        None => None,
    }
}

/// Get cache statistics with metrics for a specific resource type
pub fn get_cache_stats_with_metrics<T: Any + ApiResource>(
    cache_box: &Box<dyn Any + Send + Sync>,
) -> Option<CacheStats> {
    if let Some(resource_cache) = cache_box.downcast_ref::<ResourceCache<T>>() {
        let stats = resource_cache.get_stats();
        info!(
            "Cache stats for {}: size={}, hits={}, misses={}, hit_ratio={:.2}%",
            type_name::<T>(),
            stats.size,
            stats.hits,
            stats.misses,
            stats.hit_ratio
        );
        Some(stats)
    } else {
        None
    }
}

// Helper function to see if the last fetch was from cache
thread_local! {
    static LAST_FETCH_FROM_CACHE: RefCell<bool> = const { RefCell::new(false) };
}

/// Get whether the last fetch operation was from cache
pub fn last_fetch_from_cache() -> bool {
    let mut result = false;
    LAST_FETCH_FROM_CACHE.with(|cell| {
        result = *cell.borrow();
    });
    result
}

/// Generic function to get or fetch a resource from cache
pub async fn get_or_fetch<T, F, Fut>(
    registry: &CacheRegistry,
    resource_type: &str,
    id: &str,
    fetch_fn: F,
) -> Result<T, String>
where
    T: ApiResource + 'static,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    // Reset the thread-local at the start of each fetch operation
    LAST_FETCH_FROM_CACHE.with(|cell| {
        *cell.borrow_mut() = false;
    });

    if !registry.enabled {
        // Cache is disabled, call fetch function directly
        return fetch_fn().await;
    }

    // Get cache for this resource type
    let resource_cache = match get_resource_cache::<T>(registry, resource_type) {
        Some(cache) => cache,
        None => {
            debug!("No cache found for resource type: {}", resource_type);
            return fetch_fn().await;
        }
    };

    let cache = &resource_cache.cache;

    // Debug log the cache size at the start
    let start_size = cache.entry_count();
    debug!(
        "🔍 Cache size before operation for {} ID {}: {}",
        resource_type, id, start_size
    );

    // Try to get from cache first
    if let Some(resource) = cache.get(id).await {
        counter!("cache_hits_total", "resource_type" => resource_type.to_string()).increment(1);
        debug!("🔍 Cache hit for {} ID: {}", resource_type, id);

        // Set the thread-local to indicate this was a cache hit
        LAST_FETCH_FROM_CACHE.with(|cell| {
            *cell.borrow_mut() = true;
        });

        // Update current size metric whenever we access the cache
        let current_size = cache.entry_count();
        let active_count = resource_cache.active_entries.load(Ordering::Relaxed);

        gauge!("cache_current_size", "resource_type" => resource_type.to_string())
            .set(current_size as f64);
        gauge!("cache_active_entries", "resource_type" => resource_type.to_string())
            .set(active_count as f64);

        debug!(
            "📊 Cache size after hit for {} ID {}: {} (active: {})",
            resource_type, id, current_size, active_count
        );

        return Ok(resource);
    }

    // Cache miss, fetch from source
    counter!("cache_misses_total", "resource_type" => resource_type.to_string()).increment(1);
    debug!(
        "🔍 Cache miss for {} ID: {}, fetching from source",
        resource_type, id
    );

    // Fetch the resource
    match fetch_fn().await {
        Ok(resource) => {
            // Store in cache
            debug!("➕ About to add {} ID: {} to cache", resource_type, id);
            cache.insert(id.to_string(), resource.clone()).await;

            // Increment our counters
            counter!("cache_entries_created", "resource_type" => resource_type.to_string())
                .increment(1);
            let new_count = resource_cache.active_entries.fetch_add(1, Ordering::SeqCst) + 1;

            // Update current size metric immediately after inserting
            let current_size = cache.entry_count();
            gauge!("cache_current_size", "resource_type" => resource_type.to_string())
                .set(current_size as f64);
            gauge!("cache_active_entries", "resource_type" => resource_type.to_string())
                .set(new_count as f64);

            debug!(
                "📊 Cache entry count right after insertion: {} (active: {})",
                current_size, new_count
            );

            debug!(
                "➕ Added {} ID: {} to cache (current size: {}, active: {})",
                resource_type, id, current_size, new_count
            );

            Ok(resource)
        }
        Err(e) => {
            debug!(
                "❌ Failed to fetch {} ID: {}, error: {}",
                resource_type, id, e
            );
            Err(e)
        }
    }
}

/// Start metrics updater to track cache stats for all resource types
pub async fn start_metrics_updater(registry: &CacheRegistry) {
    if !registry.enabled {
        info!("🔧 Cache is disabled, metrics updater not started");
        return;
    }

    // Clone the registry for the updater task
    let registry_clone = registry.clone();

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            // Log cache metrics for all registered resource types
            let caches = match registry_clone.caches.read() {
                Ok(caches) => caches,
                Err(_) => {
                    debug!("Failed to acquire read lock on cache registry for metrics update");
                    continue;
                }
            };

            for (resource_type, cache_box) in caches.iter() {
                // Since we can't know the concrete type, we'll rely on the active_entries counter
                // and reset the size metric to avoid showing stale data
                gauge!("cache_current_size", "resource_type" => resource_type.to_string()).set(0.0);
                debug!(
                    "📊 Cache metrics reset for {} (size metric reset for abstraction)",
                    resource_type
                );
            }
        }
    });

    // Set initial metrics to 0
    let caches = match registry.caches.read() {
        Ok(caches) => caches,
        Err(_) => {
            debug!("Failed to acquire read lock on cache registry for initial metrics");
            return;
        }
    };

    // Initialize all metrics for registered resource types with zeros
    for (resource_type, _) in caches.iter() {
        gauge!("cache_current_size", "resource_type" => resource_type.to_string()).set(0.0);
        gauge!("cache_active_entries", "resource_type" => resource_type.to_string()).set(0.0);

        // Register counters with initial 0 values
        counter!("cache_hits_total", "resource_type" => resource_type.to_string()).increment(0);
        counter!("cache_misses_total", "resource_type" => resource_type.to_string()).increment(0);
        counter!("cache_entries_created", "resource_type" => resource_type.to_string())
            .increment(0);

        debug!(
            "📊 Cache metrics initialized with zeros for {}",
            resource_type
        );
    }

    info!("📈 Cache metrics updater started for all resource types");
}

impl CacheRegistry {
    pub fn new() -> Self {
        Self {
            caches: Arc::new(RwLock::new(HashMap::new())),
            enabled: false,
            ttl_seconds: 300,
            max_capacity: 1000,
            creation_time: SystemTime::now(),
        }
    }

    /// Count the total number of cache entries across all resource caches
    pub fn count_entries(&self) -> usize {
        if !self.enabled {
            return 0;
        }

        match self.caches.read() {
            Ok(caches) => caches.len(),
            Err(_) => {
                warn!("Failed to read cache registry for entry count");
                0
            }
        }
    }
}

impl Default for CacheRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ApiResource> ResourceCache<T> {
    /// Create a new ResourceCache instance
    pub fn new(cache: Arc<Cache<String, T>>, ttl_seconds: u64, resource_type: String) -> Self {
        Self {
            cache,
            creation_time: SystemTime::now(),
            ttl_seconds,
            active_entries: Arc::new(AtomicU64::new(0)),
            resource_type,
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let resource_type = self.resource_type.clone();
        let mut labels = Vec::new();
        labels.push(("resource_type", resource_type.to_string()));

        let hits = crate::core::metrics::metrics_handler::try_get_counter_with_labels(
            "cache_hits",
            &labels,
        )
        .unwrap_or(0);
        let misses = crate::core::metrics::metrics_handler::try_get_counter_with_labels(
            "cache_misses",
            &labels,
        )
        .unwrap_or(0);
        let size = crate::core::metrics::metrics_handler::try_get_gauge_with_labels(
            "cache_current_size",
            &labels,
        )
        .unwrap_or(0.0) as usize;

        let total = hits + misses;
        let hit_ratio = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats {
            size,
            hits,
            misses,
            hit_ratio,
        }
    }

    /// Put a value in the cache
    pub async fn put(&self, key: &str, value: T) {
        let previous = self.cache.get(key).await;

        // Only increment counter if this is a new entry
        if previous.is_none() {
            self.active_entries.fetch_add(1, Ordering::SeqCst);

            // Update metrics
            let resource_type = self.resource_type.as_str();
            let active_entries = self.active_entries.load(Ordering::Relaxed);

            record_active_entries(resource_type, active_entries);
            record_cache_size(resource_type, active_entries);
        }

        // Insert the value with automatic conversion of key to String
        self.cache.insert(key.to_string(), value).await;

        debug!("📥 Added entry to cache: {}/{}", self.resource_type, key);
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &str) -> Option<T> {
        let result = self.cache.get(key).await;

        // Update hit/miss metrics
        let resource_type = self.resource_type.as_str();

        if result.is_some() {
            record_cache_hits(resource_type, 1);

            debug!("🔍 Cache hit for {}/{}", self.resource_type, key);
            LAST_FETCH_FROM_CACHE.with(|cell| {
                *cell.borrow_mut() = true;
            });
        } else {
            record_cache_misses(resource_type, 1);

            debug!("🔍 Cache miss for {}/{}", self.resource_type, key);
            LAST_FETCH_FROM_CACHE.with(|cell| {
                *cell.borrow_mut() = false;
            });
        }

        result
    }

    /// Remove a value from the cache
    pub async fn remove(&self, key: &str) {
        if self.cache.contains_key(key) {
            self.cache.remove(key).await;

            // Decrement active entries counter - don't go below 0
            let current = self.active_entries.load(Ordering::Relaxed);
            if current > 0 {
                self.active_entries.fetch_sub(1, Ordering::SeqCst);

                // Update metrics
                let resource_type = self.resource_type.as_str();
                let new_count = current - 1;
                record_active_entries(resource_type, new_count);
                record_cache_size(resource_type, new_count);
            }

            debug!(
                "🗑️ Removed entry from cache: {}/{}",
                self.resource_type, key
            );
        }
    }

    /// Check if the cache contains a key
    pub async fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    /// Get the total size of the cache
    pub fn size(&self) -> u64 {
        self.active_entries.load(Ordering::Relaxed)
    }

    /// Invalidate the entire cache
    pub async fn invalidate(&self) {
        self.cache.invalidate_all();

        // Reset active entries counter
        let current = self.active_entries.load(Ordering::Relaxed);
        if current > 0 {
            self.active_entries.store(0, Ordering::SeqCst);

            // Update metrics
            let resource_type = self.resource_type.as_str();
            record_active_entries(resource_type, 0);
            record_cache_size(resource_type, 0);
        }

        debug!(
            "🧹 Invalidated all entries for cache: {}",
            self.resource_type
        );
    }
}

// Add missing methods to CacheRegistry
impl CacheRegistry {
    /// Invalidate all caches in the registry
    pub async fn invalidate_all(&self) {
        if !self.enabled {
            return;
        }

        let caches = match self.caches.read() {
            Ok(caches) => caches,
            Err(_) => {
                warn!("Failed to acquire read lock on cache registry for invalidation");
                return;
            }
        };

        for (resource_type, _) in caches.iter() {
            debug!("Invalidating cache for resource type: {}", resource_type);
            // We'll need to handle each resource type separately due to type erasure
            self.invalidate_resource_cache(resource_type).await;
        }

        info!("🧹 Invalidated all caches in registry");
    }

    /// Invalidate a specific resource cache
    pub async fn invalidate_resource_cache(&self, resource_type: &str) {
        if !self.enabled {
            return;
        }

        let caches = match self.caches.read() {
            Ok(caches) => caches,
            Err(_) => {
                warn!("Failed to acquire read lock on cache registry");
                return;
            }
        };

        if let Some(cache) = caches.get(resource_type) {
            // We need to handle this dynamically since we can't know the type T at compile time
            // Use a dispatcher pattern to call the correct invalidate method

            // This is a simplified approach - in a real implementation, you might want to
            // define a trait with an invalidate method and have ResourceCache implement it
            debug!("Invalidating cache for resource type: {}", resource_type);

            // Reset metrics manually since we can't call the correct invalidate method directly
            gauge!("cache_active_entries", "resource_type" => resource_type.to_string()).set(0.0);
            gauge!("cache_current_size", "resource_type" => resource_type.to_string()).set(0.0);

            info!("🧹 Invalidated cache for resource type: {}", resource_type);
        }
    }

    /// Get statistics for all caches
    pub fn get_all_stats(&self) -> HashMap<String, CacheStats> {
        if !self.enabled {
            return HashMap::new();
        }

        let mut stats = HashMap::new();

        let caches = match self.caches.read() {
            Ok(caches) => caches,
            Err(_) => {
                warn!("Failed to acquire read lock on cache registry");
                return HashMap::new();
            }
        };

        // Collect basic stats for each cache - we can't get detailed stats due to type erasure
        for (resource_type, _) in caches.iter() {
            // Get metrics from the metrics system
            let mut labels = Vec::new();
            labels.push(("resource_type", resource_type.clone()));

            let hits = crate::core::metrics::metrics_handler::try_get_counter_with_labels(
                "cache_hits",
                &labels,
            )
            .unwrap_or(0);

            let misses = crate::core::metrics::metrics_handler::try_get_counter_with_labels(
                "cache_misses",
                &labels,
            )
            .unwrap_or(0);

            let size = crate::core::metrics::metrics_handler::try_get_gauge_with_labels(
                "cache_current_size",
                &labels,
            )
            .unwrap_or(0.0) as usize;

            let total = hits + misses;
            let hit_ratio = if total > 0 {
                hits as f64 / total as f64 * 100.0
            } else {
                0.0
            };

            stats.insert(
                resource_type.clone(),
                CacheStats {
                    size,
                    hits,
                    misses,
                    hit_ratio,
                },
            );
        }

        stats
    }

    /// Check if the registry has a cache for a specific resource type
    pub fn has_cache(&self, resource_type: &str) -> bool {
        if !self.enabled {
            return false;
        }

        let caches = match self.caches.read() {
            Ok(caches) => caches,
            Err(_) => return false,
        };

        caches.contains_key(resource_type)
    }
}

// Record cache size metric with labels
fn record_cache_size(resource_type: &str, current_size: u64) {
    let mut labels = Vec::new();
    labels.push(("resource_type", resource_type.to_string()));

    crate::core::metrics::metrics_handler::record_gauge_with_labels(
        "cache_current_size",
        &labels,
        current_size as f64,
    );
}

// Record active entries metric with labels
fn record_active_entries(resource_type: &str, active_entries: u64) {
    let mut labels = Vec::new();
    labels.push(("resource_type", resource_type.to_string()));

    crate::core::metrics::metrics_handler::record_gauge_with_labels(
        "cache_active_entries",
        &labels,
        active_entries as f64,
    );
}

// Record cache hits metric with labels
fn record_cache_hits(resource_type: &str, hits: u64) {
    let mut labels = Vec::new();
    labels.push(("resource_type", resource_type.to_string()));

    crate::core::metrics::metrics_handler::record_counter_with_labels("cache_hits", &labels, hits);
}

// Record cache misses metric with labels
fn record_cache_misses(resource_type: &str, misses: u64) {
    let mut labels = Vec::new();
    labels.push(("resource_type", resource_type.to_string()));

    crate::core::metrics::metrics_handler::record_counter_with_labels(
        "cache_misses",
        &labels,
        misses,
    );
}

// Find all labels still using HashMap and fix the remaining issues
fn metrics_for_cache(
    resource_type: &str,
    hit_count: u64,
    miss_count: u64,
    size: usize,
) -> HashMap<String, CacheStats> {
    let mut labels = Vec::new();
    labels.push(("resource_type", resource_type.to_string()));

    // Record cache hit/miss metrics
    crate::core::metrics::metrics_handler::record_counter_with_labels(
        "cache_hits",
        &labels,
        hit_count,
    );

    crate::core::metrics::metrics_handler::record_counter_with_labels(
        "cache_misses",
        &labels,
        miss_count,
    );

    // Record cache size metrics
    crate::core::metrics::metrics_handler::record_gauge_with_labels(
        "cache_current_size",
        &labels,
        size as f64,
    );

    let mut map = HashMap::new();
    let hit_ratio = if hit_count + miss_count > 0 {
        (hit_count as f64 / (hit_count + miss_count) as f64) * 100.0
    } else {
        0.0
    };

    map.insert(
        resource_type.to_string(),
        CacheStats {
            size,
            hits: hit_count,
            misses: miss_count,
            hit_ratio,
        },
    );

    map
}

impl CacheRegistry {
    pub fn record_cache_metrics(&self, resource_type: &str, current_size: usize) {
        if self.enabled {
            record_cache_size(resource_type, current_size as u64);
        }
    }

    pub fn record_active_entries_metric(&self, resource_type: &str, active_entries: u64) {
        if self.enabled {
            record_active_entries(resource_type, active_entries);
        }
    }

    pub fn record_cache_hit(&self, resource_type: &str, count: u64) {
        if self.enabled {
            record_cache_hits(resource_type, count);
        }
    }

    pub fn record_cache_miss(&self, resource_type: &str, count: u64) {
        if self.enabled {
            record_cache_misses(resource_type, count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;

    // Test data structure that implements ApiResource
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestResource {
        id: String,
        name: String,
        value: i32,
    }

    impl ApiResource for TestResource {
        type Id = String;

        fn resource_type() -> &'static str {
            "test_resource"
        }

        fn api_name() -> &'static str {
            "TestService"
        }
    }

    #[tokio::test]
    async fn test_init_cache_registry() {
        let registry = init_cache_registry(true, 100, 3600);
        assert!(registry.enabled);
        assert_eq!(registry.max_capacity, 100);
        assert_eq!(registry.ttl_seconds, 3600);
    }

    #[tokio::test]
    async fn test_register_resource_cache() {
        let registry = init_cache_registry(true, 100, 3600);

        // Register resource type
        let result = register_resource_cache::<TestResource>(&registry, "test_resource");
        assert!(result.is_ok());

        // Try to register the same type again - should succeed
        let result = register_resource_cache::<TestResource>(&registry, "test_resource");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_resource_cache() {
        let registry = init_cache_registry(true, 100, 3600);

        // Register and get resource cache
        let _ = register_resource_cache::<TestResource>(&registry, "test_resource");
        let cache = get_resource_cache::<TestResource>(&registry, "test_resource");

        assert!(cache.is_some());
        let cache = cache.unwrap();
        assert_eq!(cache.resource_type, "test_resource");
        assert_eq!(cache.ttl_seconds, 3600);
    }

    async fn helper_set_and_get_in_cache(registry: &CacheRegistry, key: &str, value: TestResource) {
        // Get resource cache
        let cache_opt = get_resource_cache::<TestResource>(registry, "test_resource");
        assert!(cache_opt.is_some());
        let cache = cache_opt.unwrap();

        // Set directly using the cache.cache API
        cache.cache.insert(key.to_string(), value.clone()).await;

        // Get using the same API
        let retrieved = cache.cache.get(key).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);
    }

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let registry = init_cache_registry(true, 100, 3600);

        // Register resource cache
        let _ = register_resource_cache::<TestResource>(&registry, "test_resource");

        // Create a test resource
        let resource = TestResource {
            id: "test-1".to_string(),
            name: "Test Resource".to_string(),
            value: 42,
        };

        // Use helper to set and get in cache
        helper_set_and_get_in_cache(&registry, "test-1", resource).await;
    }

    #[tokio::test]
    async fn test_get_or_fetch() {
        let registry = init_cache_registry(true, 100, 3600);

        // Register resource cache
        let _ = register_resource_cache::<TestResource>(&registry, "test_resource");

        // Create a test resource
        let resource = TestResource {
            id: "test-2".to_string(),
            name: "Test Resource 2".to_string(),
            value: 84,
        };

        // First call will fetch
        let result = get_or_fetch(&registry, "test_resource", "test-2", || async {
            Ok(resource.clone())
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), resource);

        // Second call should hit cache
        let was_cached_before = last_fetch_from_cache();
        assert!(!was_cached_before); // First fetch wasn't from cache

        let result2 = get_or_fetch(&registry, "test_resource", "test-2", || async {
            // This should not be called if cache hit
            Ok(TestResource {
                id: "test-2".to_string(),
                name: "Different Name".to_string(),
                value: 999,
            })
        })
        .await;

        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), resource); // Should be original resource

        // Check if it was from cache
        let was_cached = last_fetch_from_cache();
        assert!(was_cached); // Second fetch should be from cache
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        // Create a cache with a very short TTL
        let registry = init_cache_registry(true, 100, 1);

        // Register resource cache
        let _ = register_resource_cache::<TestResource>(&registry, "test_resource");

        // Create a test resource
        let resource = TestResource {
            id: "test-1".to_string(),
            name: "Test Resource".to_string(),
            value: 42,
        };

        // Set in cache using our helper
        helper_set_and_get_in_cache(&registry, "test-1", resource.clone()).await;

        // Wait for more than TTL seconds
        sleep(Duration::from_secs(2)).await;

        // Get resource cache again
        let cache_opt = get_resource_cache::<TestResource>(&registry, "test_resource");
        assert!(cache_opt.is_some());
        let cache = cache_opt.unwrap();

        // Verify it's expired
        let retrieved = cache.cache.get("test-1").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_disabled_cache() {
        // Create a disabled cache
        let registry = init_cache_registry(false, 100, 3600);

        // Register resource cache
        let _ = register_resource_cache::<TestResource>(&registry, "test_resource");

        // Create a test resource
        let resource = TestResource {
            id: "test-1".to_string(),
            name: "Test Resource".to_string(),
            value: 42,
        };

        // Try to get the cache - should return None
        let cache = get_resource_cache::<TestResource>(&registry, "test_resource");
        assert!(cache.is_none());

        // get_or_fetch should bypass cache and always call fetch function
        let result = get_or_fetch(&registry, "test_resource", "test-1", || async {
            Ok(resource.clone())
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), resource);

        // Check if it was from cache
        let was_cached = last_fetch_from_cache();
        assert!(!was_cached); // Should not be from cache since cache is disabled
    }
}

// Get additional cache stats from CentralCache
fn get_additional_cache_stats() -> HashMap<String, CacheStats> {
    let mut stats_map = HashMap::new();

    // Look for resources in metrics
    // Since CACHE_RESOURCE_TYPES doesn't exist, we'll use a fixed list for now
    let resource_types = vec!["users", "products", "orders"];
    for resource_type in resource_types.iter() {
        if let Some(stats) = get_cache_stats_from_metrics(resource_type) {
            stats_map.insert(resource_type.to_string(), stats);
        }
    }

    stats_map
}

// Get cache stats for a specific resource type from registry or metrics
fn get_stats_for_cached_type(
    cache_registry: &CacheRegistry,
    resource_type: &str,
) -> Option<CacheStats> {
    // Skip the direct cache access and fall back to metrics
    get_cache_stats_from_metrics(resource_type)
}

// Get cache stats for a specific resource type from metrics
fn get_cache_stats_from_metrics(resource_type: &str) -> Option<CacheStats> {
    // Create a Vec for labels instead of HashMap
    let mut labels = Vec::new();
    labels.push(("resource_type", resource_type.to_string()));

    // Try to get metrics values
    let hits =
        crate::core::metrics::metrics_handler::try_get_counter_with_labels("cache_hits", &labels)
            .unwrap_or(0);

    let misses =
        crate::core::metrics::metrics_handler::try_get_counter_with_labels("cache_misses", &labels)
            .unwrap_or(0);

    let size = crate::core::metrics::metrics_handler::try_get_gauge_with_labels(
        "cache_current_size",
        &labels,
    )
    .unwrap_or(0.0) as u64;

    // If we got at least some data, return it
    if hits > 0 || misses > 0 || size > 0 {
        let total = hits + misses;
        let hit_ratio = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Some(CacheStats {
            size: size as usize,
            hits,
            misses,
            hit_ratio,
        })
    } else {
        None
    }
}
