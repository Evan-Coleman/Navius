// Copyright (c) 2025 Navius Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Application Caching
//!
//! This module contains user-facing caching functionality that can be customized.
//! Core caching capabilities are provided by `crate::core::cache`.

// Temporarily commenting out the problematic module
// pub mod providers;

use crate::core::error::{AppError, Result};
use crate::core::services::cache_provider::{
    CacheConfig, CacheProviderRegistry, DynCacheOperations, EvictionPolicy, TypedCache,
    TypedCacheFactory,
};
use crate::core::services::cache_service::CacheService;
use crate::core::services::memory_cache::InMemoryCacheProvider;
use crate::core::services::redis_cache::RedisCacheProvider;

// Re-export core cache functionality
pub use crate::core::cache::*;

use bincode::{Decode, Encode};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

// Temporarily commenting out
// pub use providers::two_tier_cache::TwoTierCache;

/// Example: Create a cache factory for a specific use case
///
/// Note: In a real application, this would create an actual cache implementation
/// based on the core cache API
pub fn create_user_cache() -> Result<String> {
    // This is just a placeholder until we have access to the actual cache API
    // In a real implementation, this would create and return a cache instance

    Ok("Example cache".to_string())
}

/*
/// Create a two-tier cache with a fast in-memory cache and a slower persistent cache
///
/// This implements the cache fallback strategy, where:
/// - Reads check the fast cache first, then the slow cache if not found
/// - Items found in the slow cache are automatically promoted to the fast cache
/// - Writes go to both caches simultaneously
/// - Different TTLs can be set for each cache level
pub async fn create_two_tier_cache(
    name: &str,
    cache_service: &CacheService,
    fast_ttl: Option<Duration>,
    slow_ttl: Option<Duration>,
) -> Result<Box<dyn DynCacheOperations>> {
    // Create fast cache (in-memory)
    let fast_config = CacheConfig {
        name: format!("{}_fast", name),
        provider: "memory".to_string(),
        capacity: Some(1000),
        default_ttl: fast_ttl.or(Some(Duration::from_secs(300))), // 5 minutes default
        eviction_policy: EvictionPolicy::LRU,
        provider_config: HashMap::new(),
    };

    let fast_cache = cache_service.get_cache(fast_config).await?;

    // Create slow cache (Redis)
    let slow_config = CacheConfig {
        name: format!("{}_slow", name),
        provider: "redis".to_string(),
        capacity: None, // Redis doesn't need capacity
        default_ttl: slow_ttl.or(Some(Duration::from_secs(3600))), // 1 hour default
        eviction_policy: EvictionPolicy::TTL,
        provider_config: HashMap::new(),
    };

    let slow_cache = cache_service.get_cache(slow_config).await?;

    // Create two-tier cache
    let two_tier_cache = TwoTierCache::new(name.to_string(), fast_cache, slow_cache);

    info!("Created two-tier cache '{}' with memory+redis layers", name);
    Ok(Box::new(two_tier_cache))
}

/// Create a two-tier cache for a specific type
pub async fn create_typed_two_tier_cache<T>(
    name: &str,
    cache_service: &CacheService,
    fast_ttl: Option<Duration>,
    slow_ttl: Option<Duration>,
) -> Result<Box<dyn TypedCache<T>>>
where
    T: Encode + Decode<()> + Send + Sync + 'static,
{
    let cache = create_two_tier_cache(name, cache_service, fast_ttl, slow_ttl).await?;
    Ok(cache.get_typed_cache::<T>())
}

/// Fallback to a memory-only two-tier cache for development environments
pub async fn create_memory_only_two_tier_cache(
    name: &str,
    cache_service: &CacheService,
    fast_ttl: Option<Duration>,
    slow_ttl: Option<Duration>,
) -> Result<Box<dyn DynCacheOperations>> {
    // Create fast cache (small memory cache)
    let fast_config = CacheConfig {
        name: format!("{}_fast", name),
        provider: "memory".to_string(),
        capacity: Some(100),
        default_ttl: fast_ttl.or(Some(Duration::from_secs(60))), // 1 minute default
        eviction_policy: EvictionPolicy::LRU,
        provider_config: HashMap::new(),
    };

    let fast_cache = cache_service.get_cache(fast_config).await?;

    // Create slow cache (larger memory cache)
    let slow_config = CacheConfig {
        name: format!("{}_slow", name),
        provider: "memory".to_string(),
        capacity: Some(1000),
        default_ttl: slow_ttl.or(Some(Duration::from_secs(300))), // 5 minutes default
        eviction_policy: EvictionPolicy::LRU,
        provider_config: HashMap::new(),
    };

    let slow_cache = cache_service.get_cache(slow_config).await?;

    // Create two-tier cache
    let two_tier_cache = TwoTierCache::new(name.to_string(), fast_cache, slow_cache);

    debug!(
        "Created memory-only two-tier cache '{}' for development",
        name
    );
    Ok(Box::new(two_tier_cache))
}
*/

/// Helper function for detecting if Redis is available
pub async fn is_redis_available(cache_service: &CacheService) -> bool {
    cache_service
        .available_providers()
        .contains(&"redis".to_string())
}
