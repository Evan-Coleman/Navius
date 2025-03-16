use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};
use utoipa::ToSchema;

use crate::models::Pet;

/// Type alias for the pet cache
pub type PetCache = Arc<Cache<i64, Pet>>;

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: u64,
}

/// Initialize the pet cache
pub fn init_cache(max_capacity: u64, ttl_seconds: u64) -> PetCache {
    info!(
        "Initializing cache with TTL: {}s, capacity: {} items",
        ttl_seconds, max_capacity
    );

    Arc::new(
        Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build(),
    )
}

/// Get cache statistics
pub fn get_cache_stats(cache: &PetCache) -> CacheStats {
    let cache_ref = &**cache;
    CacheStats {
        hits: 0,   // We don't have direct access to hits/misses
        misses: 0, // We track these with our own counters
        size: cache_ref.entry_count(),
    }
}

/// Get pet from cache or use provided function to fetch it
pub async fn get_or_fetch<F, Fut>(cache: &PetCache, id: i64, fetch_fn: F) -> Result<Pet, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Pet, String>>,
{
    // Try to get from cache first
    if let Some(pet) = cache.get(&id).await {
        debug!("Cache hit for pet ID: {}", id);
        return Ok(pet);
    }

    // Cache miss, fetch from source
    debug!("Cache miss for pet ID: {}", id);

    // Fetch the pet
    let pet = fetch_fn().await?;

    // Store in cache
    cache.insert(id, pet.clone()).await;
    debug!("Added pet ID: {} to cache", id);

    Ok(pet)
}

/// Start metrics updater to track cache stats
pub fn start_metrics_updater(start_time: std::time::SystemTime, cache_opt: Option<PetCache>) {
    if let Some(cache) = cache_opt {
        let cache = cache.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15));
            loop {
                interval.tick().await;

                // Just log uptime and cache size
                if let Ok(uptime) = start_time.elapsed() {
                    debug!("App uptime: {} seconds", uptime.as_secs());
                    debug!("Cache size: {} items", cache.entry_count());
                }
            }
        });
    } else {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15));
            loop {
                interval.tick().await;

                // Just log uptime
                if let Ok(uptime) = start_time.elapsed() {
                    debug!("App uptime: {} seconds", uptime.as_secs());
                }
            }
        });
    }
}
