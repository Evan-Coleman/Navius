// Check that the cache implementation correctly updates metrics
// and that start_metrics_updater is properly implemented

// Example fix if start_metrics_updater is not passing the correct parameters:
use metrics::{counter, gauge};
use moka::future::Cache;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time::interval;
use tracing::info;

use crate::generated_apis::petstore_api::models::Upet;

/// Cache for pet data
pub type PetCache = Arc<Cache<i64, Upet>>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_size: u64,
    pub uptime_seconds: u64,
}

/// Initialize the pet cache
pub fn init_cache(max_capacity: u64, ttl_seconds: u64) -> PetCache {
    let cache = Cache::builder()
        .max_capacity(max_capacity)
        .time_to_live(Duration::from_secs(ttl_seconds))
        .build();

    Arc::new(cache)
}

/// Start the metrics updater
pub fn start_metrics_updater(start_time: SystemTime, pet_cache: Option<PetCache>) {
    if let Some(cache) = pet_cache {
        tokio::spawn(async move {
            let mut tick_interval = interval(Duration::from_secs(15));
            let mut previous_hit_count = 0;
            let mut previous_miss_count = 0;

            loop {
                tick_interval.tick().await;

                // Update cache metrics
                let hit_count = cache.stats().hit_count();
                let miss_count = cache.stats().miss_count();
                let entry_count = cache.entry_count() as u64;

                // Calculate the delta hits/misses since last check
                let new_hits = hit_count.saturating_sub(previous_hit_count);
                let new_misses = miss_count.saturating_sub(previous_miss_count);

                // Increment counters for new hits/misses (fixed format)
                if new_hits > 0 {
                    counter!("pet_cache_hits_total").increment(new_hits);
                }
                if new_misses > 0 {
                    counter!("pet_cache_misses_total").increment(new_misses);
                }

                // Update previous values
                previous_hit_count = hit_count;
                previous_miss_count = miss_count;

                // Update Prometheus metrics
                gauge!("pet_cache_size", entry_count as f64);
                gauge!("pet_cache_hit_count", hit_count as f64);
                gauge!("pet_cache_miss_count", miss_count as f64);

                // Calculate uptime
                if let Ok(duration) = SystemTime::now().duration_since(start_time) {
                    gauge!("app_uptime_seconds", duration.as_secs() as f64);
                }

                info!(
                    "📊 Cache stats: hits={}, misses={}, size={}",
                    hit_count, miss_count, entry_count
                );
            }
        });
    }
}

impl PetCache {
    /// Get stats for the cache
    pub async fn stats(&self) -> CacheStats {
        let uptime = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        CacheStats {
            cache_hits: self.stats().hit_count(),
            cache_misses: self.stats().miss_count(),
            cache_size: self.entry_count() as u64,
            uptime_seconds: uptime,
        }
    }
}
