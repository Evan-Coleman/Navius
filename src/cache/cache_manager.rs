use metrics::{counter, gauge};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time::interval;
use tracing::{debug, info};
use utoipa::ToSchema;

use crate::generated_apis::petstore_api::models::Upet;

/// Type alias for the pet cache
pub type PetCache = Arc<Cache<i64, Upet>>;

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: u64,
    pub uptime_seconds: u64,
}

/// Initialize the pet cache
pub fn init_cache(max_capacity: u64, ttl_seconds: u64) -> PetCache {
    info!(
        "🔧 Initializing cache with TTL: {}s, capacity: {} items",
        ttl_seconds, max_capacity
    );

    Arc::new(
        Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build(),
    )
}

/// Get cache statistics with metrics data
pub fn get_cache_stats_with_metrics(
    cache: &PetCache,
    uptime_seconds: u64,
    metrics_text: &str,
) -> CacheStats {
    let cached_entries = cache.entry_count();

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
pub async fn get_or_fetch<F, Fut>(cache: &PetCache, id: i64, fetch_fn: F) -> Result<Upet, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Upet, String>>,
{
    // Try to get from cache first
    if let Some(pet) = cache.get(&id).await {
        counter!("pet_cache_hits_total").increment(1);
        debug!("Cache hit for pet ID: {}", id);
        return Ok(pet);
    }

    // Cache miss, fetch from source
    counter!("pet_cache_misses_total").increment(1);
    debug!("Cache miss for pet ID: {}", id);

    // Fetch the pet
    let pet = fetch_fn().await?;

    // Store in cache
    cache.insert(id, pet.clone()).await;
    counter!("cache_entries_created").increment(1);
    debug!("Added pet ID: {} to cache", id);

    Ok(pet)
}

/// Start metrics updater to track cache stats
pub fn start_metrics_updater(start_time: SystemTime, pet_cache: Option<PetCache>) {
    if let Some(cache) = pet_cache {
        tokio::spawn(async move {
            let mut tick_interval = interval(Duration::from_secs(15));

            info!("🔄 Metrics updater started for cache");

            loop {
                tick_interval.tick().await;

                // Update cache metrics
                let entry_count = cache.entry_count() as f64;

                // Update Prometheus metrics - no labels
                gauge!("pet_cache_size").set(entry_count);

                // Calculate uptime
                if let Ok(duration) = SystemTime::now().duration_since(start_time) {
                    let uptime_secs = duration.as_secs() as f64;
                    gauge!("app_uptime_seconds").set(uptime_secs);
                }

                info!("📊 Cache stats: size={}", entry_count as u64);
            }
        });
    }
}
