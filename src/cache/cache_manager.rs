use metrics::{counter, gauge};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, SystemTime};
use tokio::time::interval;
use tracing::{debug, info};

// Import ApiResource trait
use crate::utils::api_resource::ApiResource;

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

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub resource_type: String,
    pub uptime_seconds: u64,
    pub size: u64,
    pub active_entries: u64,
    pub entries_created: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_ratio: f64,
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
    let cache_builder = Cache::builder()
        .max_capacity(registry.max_capacity)
        .time_to_live(ttl)
        .time_to_idle(ttl.mul_f32(1.5))
        .initial_capacity(100)
        .build();

    let resource_cache: ResourceCache<T> = ResourceCache {
        cache: Arc::new(cache_builder),
        creation_time: SystemTime::now(),
        ttl_seconds: registry.ttl_seconds,
        active_entries: Arc::new(AtomicU64::new(0)),
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

/// Get cache statistics with metrics data
pub fn get_cache_stats_with_metrics(
    registry: &CacheRegistry,
    resource_type: &str,
    metrics_text: &str,
) -> Option<CacheStats> {
    if !registry.enabled {
        return None;
    }

    // Calculate uptime
    let uptime_seconds = SystemTime::now()
        .duration_since(registry.creation_time)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Parse metrics text to extract hit and miss counts for this resource type
    let mut hits = 0;
    let mut misses = 0;
    let mut entries_created = 0;
    let mut size = 0;
    let mut active_entries = 0;

    // Construct the metric name with label that we're looking for
    let metric_label = format!("resource_type=\"{}\"", resource_type);

    for line in metrics_text.lines() {
        // Skip comment lines and empty lines
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        // Process only lines with our resource type
        if !line.contains(&metric_label) {
            continue;
        }

        // Extract the metric value (should be the second item when split by whitespace)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let value = match parts[1].parse::<u64>() {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Update the appropriate counter based on metric name
        if line.contains("cache_hits_total") {
            hits = value;
        } else if line.contains("cache_misses_total") {
            misses = value;
        } else if line.contains("cache_entries_created") {
            entries_created = value;
        } else if line.contains("cache_current_size") {
            size = value;
        } else if line.contains("cache_active_entries") {
            active_entries = value;
        }
    }

    // Calculate hit ratio
    let hit_ratio = if hits + misses > 0 {
        (hits as f64 / (hits + misses) as f64) * 100.0
    } else {
        0.0
    };

    // Return the stats object with all the collected metrics
    Some(CacheStats {
        resource_type: resource_type.to_string(),
        uptime_seconds,
        size,
        active_entries,
        entries_created,
        hits,
        misses,
        hit_ratio,
    })
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

            for (resource_type, _cache_box) in caches.iter() {
                // We don't have a direct way to access the cache entry count from the boxed cache
                // So we'll at least ensure the resource type is registered in the metrics

                // These are placeholder values that will be properly updated by the cache operations
                // Simply setting a non-zero value (1.0) ensures the metrics exist for the health endpoints
                gauge!("cache_current_size", "resource_type" => resource_type.to_string()).set(1.0);
                gauge!("cache_active_entries", "resource_type" => resource_type.to_string())
                    .set(1.0);
                counter!("cache_hits_total", "resource_type" => resource_type.to_string())
                    .increment(0);
                counter!("cache_misses_total", "resource_type" => resource_type.to_string())
                    .increment(0);
                counter!("cache_entries_created", "resource_type" => resource_type.to_string())
                    .increment(0);

                info!("📊 Cache metrics registered for {}", resource_type);
            }
        }
    });

    info!("📈 Cache metrics updater started for all resource types");
}
