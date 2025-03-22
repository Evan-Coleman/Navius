//! Property-based tests for cache providers
//! These tests verify properties that should hold true regardless of input values

use crate::cache::providers::CacheProvider;
use crate::cache::providers::memory::MemoryCacheProvider;
use crate::utils::api_resource::ApiResource;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Builder;

/// Test data structure that implements ApiResource
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

/// A wrapper to run async functions in proptest - creates a fresh runtime for each call
/// to avoid issues with runtime being used after being shut down
fn run_async<F: std::future::Future>(future: F) -> F::Output {
    // Use a basic single-threaded runtime to avoid thread pool issues
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(future)
}

proptest! {
    /// Test that cache get after set always returns the same value
    #[test]
    fn memory_cache_get_after_set_returns_same_value(
        id in "[a-z0-9]{1,10}",
        name in "[a-zA-Z ]{1,20}",
        value in 0..1000,
        ttl in 1..3600u64
    ) {
        let provider = MemoryCacheProvider::new(100, 3600);
        let resource = TestResource {
            id: id.clone(),
            name,
            value,
        };

        // Initialize provider
        let _ = provider.init();

        // Register the resource type
        let registry = provider.registry();
        let _ = crate::core::cache::cache_manager::register_resource_cache::<TestResource>(
            &registry,
            "test_resource",
        );

        // Set the resource in the cache
        let set_result = run_async(provider.set(&id, resource.clone(), ttl));
        prop_assert!(set_result.is_ok());

        // Get the resource from the cache
        let get_result = run_async(provider.get::<TestResource>(&id));
        prop_assert!(get_result.is_ok());

        let retrieved_resource = get_result.unwrap();
        prop_assert!(retrieved_resource.is_some());

        let retrieved = retrieved_resource.unwrap();
        prop_assert_eq!(retrieved.id, resource.id);
        prop_assert_eq!(retrieved.name, resource.name);
        prop_assert_eq!(retrieved.value, resource.value);
    }

    /// Test that get returns None for nonexistent keys
    #[test]
    fn memory_cache_get_nonexistent_returns_none(
        id in "[a-z0-9]{1,10}"
    ) {
        let provider = MemoryCacheProvider::new(100, 3600);

        // Initialize provider
        let _ = provider.init();

        // Register the resource type
        let registry = provider.registry();
        let _ = crate::core::cache::cache_manager::register_resource_cache::<TestResource>(
            &registry,
            "test_resource",
        );

        // Try to get a resource that doesn't exist
        let get_result = run_async(provider.get::<TestResource>(&id));
        prop_assert!(get_result.is_ok());
        prop_assert!(get_result.unwrap().is_none());
    }

    /// Test that cache provider returns stats
    #[test]
    fn memory_cache_get_stats_returns_valid_json(
        max_capacity in 10..1000u64,
        ttl in 1..3600u64
    ) {
        let provider = MemoryCacheProvider::new(max_capacity, ttl);

        // Initialize provider
        let _ = provider.init();

        // Get stats
        let stats_result = run_async(provider.get_stats());
        prop_assert!(stats_result.is_ok());

        let stats = stats_result.unwrap();

        // Verify the stats contains the expected fields
        prop_assert!(stats.is_object());
        prop_assert!(stats.get("provider").is_some());
        prop_assert_eq!(stats.get("provider").unwrap().as_str().unwrap(), "memory");
        prop_assert!(stats.get("enabled").is_some());
        prop_assert_eq!(stats.get("enabled").unwrap().as_bool().unwrap(), true);
        prop_assert!(stats.get("max_capacity").is_some());
        prop_assert_eq!(stats.get("max_capacity").unwrap().as_u64().unwrap(), max_capacity);
        prop_assert!(stats.get("ttl_seconds").is_some());
        prop_assert_eq!(stats.get("ttl_seconds").unwrap().as_u64().unwrap(), ttl);
    }
}
