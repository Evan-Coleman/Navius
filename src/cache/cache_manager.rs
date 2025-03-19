use metrics::{counter, gauge};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, SystemTime};
use tokio::time::interval;
use tracing::{debug, info};

use crate::generated_apis::petstore_api::models::Upet;

/// Type alias for the pet cache
pub type PetCache = Arc<Cache<i64, Upet>>;

/// Wrapper for cache with TTL information
#[derive(Debug, Clone)]
pub struct CacheWithTTL {
    pub cache: PetCache,
    pub creation_time: SystemTime,
    pub ttl_seconds: u64,
    // Add an atomic counter for current active entries
    pub active_entries: Arc<AtomicU64>,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub uptime_seconds: u64,
    pub size: u64,
    pub active_entries: u64,
    pub entries_created: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_ratio: f64,
}

/// Initialize the pet cache with TTL
pub fn init_cache(max_capacity: u64, ttl_seconds: u64) -> CacheWithTTL {
    let ttl = Duration::from_secs(ttl_seconds);

    let cache = Arc::new(
        Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            // Add additional configuration to make the cache more resilient
            .time_to_idle(ttl.mul_f32(1.5)) // Set idle time to 1.5x the TTL
            .initial_capacity(100) // Pre-allocate some capacity
            .build(),
    );

    CacheWithTTL {
        cache,
        creation_time: SystemTime::now(),
        ttl_seconds,
        active_entries: Arc::new(AtomicU64::new(0)),
    }
}

/// Get cache statistics with metrics data
pub fn get_cache_stats_with_metrics(
    cache_with_ttl: &CacheWithTTL,
    metrics_text: &str,
) -> CacheStats {
    let cache = &cache_with_ttl.cache;
    let cached_entries = cache.entry_count();
    let active_entries = cache_with_ttl
        .active_entries
        .load(std::sync::atomic::Ordering::Relaxed);

    // Calculate uptime
    let uptime_seconds = SystemTime::now()
        .duration_since(cache_with_ttl.creation_time)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Parse metrics text to extract hit and miss counts
    let mut hits = 0;
    let mut misses = 0;
    let mut entries_created = 0;

    for line in metrics_text.lines() {
        if line.contains("pet_cache_hits_total") && !line.starts_with('#') {
            if let Some(value) = line.split_whitespace().nth(1) {
                if let Ok(count) = value.parse::<u64>() {
                    hits = count;
                }
            }
        } else if line.contains("pet_cache_misses_total") && !line.starts_with('#') {
            if let Some(value) = line.split_whitespace().nth(1) {
                if let Ok(count) = value.parse::<u64>() {
                    misses = count;
                }
            }
        } else if line.contains("cache_entries_created") && !line.starts_with('#') {
            if let Some(value) = line.split_whitespace().nth(1) {
                if let Ok(count) = value.parse::<u64>() {
                    entries_created = count;
                }
            }
        }
    }

    // Calculate hit ratio
    let hit_ratio = if hits + misses > 0 {
        (hits as f64 / (hits + misses) as f64) * 100.0
    } else {
        0.0
    };

    CacheStats {
        uptime_seconds,
        size: cached_entries,
        active_entries,
        entries_created,
        hits,
        misses,
        hit_ratio,
    }
}

/// Get basic cache statistics (without metrics data)
pub fn get_cache_stats(cache: &PetCache, uptime_seconds: u64) -> CacheStats {
    let cached_entries = cache.entry_count();

    CacheStats {
        uptime_seconds,
        size: cached_entries,
        active_entries: 0,  // We track active entries using our atomic counter
        entries_created: 0, // We track entries created using metrics
        hits: 0,            // We track hits using metrics
        misses: 0,          // We track misses using metrics
        hit_ratio: 0.0,     // We calculate the hit ratio from hits and misses
    }
}

/// Get pet from cache or use provided function to fetch it
pub async fn get_or_fetch<F, Fut>(
    cache_with_ttl: &CacheWithTTL,
    id: i64,
    fetch_fn: F,
) -> Result<Upet, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Upet, String>>,
{
    let cache = &cache_with_ttl.cache;

    // Debug log the cache size at the start
    let start_size = cache.entry_count();
    debug!(
        "🔍 Cache size before operation for pet ID {}: {}",
        id, start_size
    );

    // Try to get from cache first
    if let Some(pet) = cache.get(&id).await {
        counter!("pet_cache_hits_total").increment(1);
        debug!("🔍 Cache hit for pet ID: {} (name: {})", id, pet.name);

        // Update current size metric whenever we access the cache
        let current_size = cache.entry_count();
        let active_count = cache_with_ttl.active_entries.load(Ordering::Relaxed);

        gauge!("cache_current_size").set(current_size as f64);
        gauge!("cache_active_entries").set(active_count as f64);

        debug!(
            "📊 Cache size after hit for pet ID {}: {} (active: {})",
            id, current_size, active_count
        );

        return Ok(pet);
    }

    // Cache miss, fetch from source
    counter!("pet_cache_misses_total").increment(1);
    debug!("🔍 Cache miss for pet ID: {}, fetching from source", id);

    // Fetch the pet
    match fetch_fn().await {
        Ok(pet) => {
            // Store in cache
            debug!("➕ About to add pet ID: {} to cache", id);
            cache.insert(id, pet.clone()).await;

            // Increment our counters
            counter!("cache_entries_created").increment(1);
            let new_count = cache_with_ttl.active_entries.fetch_add(1, Ordering::SeqCst) + 1;

            // Update current size metric immediately after inserting
            let current_size = cache.entry_count();
            gauge!("cache_current_size").set(current_size as f64);
            gauge!("cache_active_entries").set(new_count as f64);

            // Additional debugging for what's in the cache
            debug!(
                "📊 Cache entry count right after insertion: {} (active: {})",
                current_size, new_count
            );

            // Try to read back the value to confirm it's there
            let check = cache.get(&id).await;
            debug!(
                "🔍 Verification check - pet {} is in cache: {}",
                id,
                check.is_some()
            );

            debug!(
                "➕ Added pet ID: {} (name: {}) to cache (current size: {}, active: {})",
                id, pet.name, current_size, new_count
            );

            Ok(pet)
        }
        Err(e) => {
            debug!("❌ Failed to fetch pet ID: {}, error: {}", id, e);
            Err(e)
        }
    }
}

/// Start metrics updater to track cache stats
pub fn start_metrics_updater(cache_with_ttl: Option<CacheWithTTL>) {
    if let Some(cache) = cache_with_ttl {
        tokio::spawn(async move {
            let mut tick_interval = interval(Duration::from_secs(15));

            info!("🔄 Metrics updater started for cache");

            // Get an initial size to help diagnose initial state
            let initial_size = cache.cache.entry_count();
            let initial_active = cache.active_entries.load(Ordering::Relaxed);
            info!(
                "📊 Initial cache size on startup: {} (active: {})",
                initial_size, initial_active
            );

            loop {
                tick_interval.tick().await;

                // Update cache metrics
                let size = cache.cache.entry_count();
                let active_count = cache.active_entries.load(Ordering::Relaxed);

                // Update metrics directly from our atomic counter
                gauge!("cache_size").set(active_count as f64);
                gauge!("cache_active_entries").set(active_count as f64);

                // Calculate cache TTL percentage used
                if let Ok(elapsed) = SystemTime::now().duration_since(cache.creation_time) {
                    let ttl_percentage =
                        (elapsed.as_secs() as f64 / cache.ttl_seconds as f64) * 100.0;
                    gauge!("cache_ttl_percentage").set(ttl_percentage);
                }

                // Log cache stats every 5 minutes (20 ticks at 15 second intervals)
                static mut TICK_COUNT: u64 = 0;
                unsafe {
                    TICK_COUNT += 1;
                    if TICK_COUNT % 20 == 0 {
                        info!("📊 Cache size: {} (active: {})", size, active_count);
                    }
                }
            }
        });
    }
}
