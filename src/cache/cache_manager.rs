use metrics::{counter, gauge};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: u64,
    pub uptime_seconds: u64,
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
    }
}

/// Get cache statistics with metrics data
pub fn get_cache_stats_with_metrics(
    cache_with_ttl: &CacheWithTTL,
    metrics_text: &str,
) -> CacheStats {
    let cache = &cache_with_ttl.cache;
    let cached_entries = cache.entry_count();

    // Calculate uptime
    let uptime_seconds = SystemTime::now()
        .duration_since(cache_with_ttl.creation_time)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Parse metrics text to extract hit and miss counts
    let mut hits = 0;
    let mut misses = 0;

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
        }
    }

    CacheStats {
        hits,
        misses,
        size: cached_entries,
        uptime_seconds,
    }
}

/// Get basic cache statistics (without metrics data)
pub fn get_cache_stats(cache: &PetCache, uptime_seconds: u64) -> CacheStats {
    let cached_entries = cache.entry_count();

    CacheStats {
        hits: 0,   // We track hits using metrics
        misses: 0, // We track misses using metrics
        size: cached_entries,
        uptime_seconds,
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

    // Try to get from cache first
    if let Some(pet) = cache.get(&id).await {
        counter!("pet_cache_hits_total").increment(1);
        debug!("🔍 Cache hit for pet ID: {} (name: {})", id, pet.name);
        return Ok(pet);
    }

    // Cache miss, fetch from source
    counter!("pet_cache_misses_total").increment(1);
    debug!("🔍 Cache miss for pet ID: {}, fetching from source", id);

    // Fetch the pet
    match fetch_fn().await {
        Ok(pet) => {
            // Store in cache
            cache.insert(id, pet.clone()).await;
            counter!("cache_entries_created").increment(1);
            debug!("➕ Added pet ID: {} (name: {}) to cache", id, pet.name);
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

            loop {
                tick_interval.tick().await;

                // Update cache metrics
                let size = cache.cache.entry_count();
                gauge!("cache_size").set(size as f64);

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
                        info!("📊 Cache size: {}", size);
                    }
                }
            }
        });
    }
}
