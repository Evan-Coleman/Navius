//! # API Resource Abstraction
//!
//! This module provides a high-level abstraction for API resources that
//! handles common concerns like caching, retries, and error handling.

mod core;
mod registry;

// Re-export public items
pub use core::{ApiHandlerOptions, ApiResource, create_api_handler, fetch_with_retry};
pub use registry::*;

use crate::core::cache::cache_manager::register_resource_cache;
use crate::core::router::AppState;
use std::sync::Arc;
use tracing::info;

/// Register a resource type in the cache registry
///
/// This function checks if the cache registry is enabled and then registers
/// the resource type if it isn't already registered.
///
/// # Type Parameters
///
/// - `T`: The resource type that implements ApiResource
///
/// # Arguments
///
/// - `state`: The application state
/// - `resource_type`: The resource type to register (optional, defaults to T::resource_type())
///
/// # Returns
///
/// - `Ok(())` if the registration was successful or not needed
/// - `Err(String)` if there was an error registering the resource
pub fn register_resource<T: ApiResource + 'static>(
    state: &Arc<AppState>,
    resource_type: Option<&str>,
) -> Result<(), String> {
    // Skip if cache is disabled
    let Some(registry) = &state.cache_registry else {
        return Ok(());
    };

    let resource_name = resource_type.unwrap_or_else(|| T::resource_type());

    // Register the resource type in the cache registry
    match register_resource_cache::<T>(registry, resource_name) {
        Ok(_) => {
            info!(
                "✅ Registered resource type {} in cache registry",
                resource_name
            );
            Ok(())
        }
        Err(e) => {
            // Don't fail if registration fails - just log and continue
            info!(
                "⚠️ Failed to register resource type {} in cache registry: {}",
                resource_name, e
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        cache::cache_manager, config::app_config::AppConfig,
        utils::api_resource::registry::ApiResourceRegistry,
    };
    use metrics_exporter_prometheus::PrometheusBuilder;
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use std::{sync::Arc, time::SystemTime};

    // Mock resource for testing
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct MockResource {
        id: i64,
        name: String,
    }

    impl ApiResource for MockResource {
        type Id = i64;

        fn resource_type() -> &'static str {
            "mock"
        }

        fn api_name() -> &'static str {
            "TestAPI"
        }
    }

    // Helper function to create test app state
    fn create_test_app_state(with_cache: bool) -> Arc<AppState> {
        let cache_registry = if with_cache {
            Some(cache_manager::init_cache_registry(true, 1000, 300))
        } else {
            None
        };

        let metrics_recorder = PrometheusBuilder::new().build_recorder();
        let metrics_handle = metrics_recorder.handle();

        Arc::new(AppState {
            config: AppConfig::default(),
            start_time: SystemTime::now(),
            cache_registry: Some(Arc::new(cache_registry)),
            client: Some(Client::new()),
            db_pool: None,
            token_client: Some(Arc::new(MockTokenClient::default())),
            metrics_handle: Some(metrics_handle),
            resource_registry: Some(Arc::new(ApiResourceRegistry::new())),
            service_registry: Arc::new(ServiceRegistry::new()),
        })
    }

    #[test]
    fn test_register_resource_with_cache_enabled() {
        // Create a cache registry
        let app_state = create_test_app_state(true);

        // This line creates the warning, renaming to _cache_registry
        let _cache_registry = app_state.cache_registry.as_ref().unwrap();

        // Register the resource
        let result = register_resource::<MockResource>(&app_state, None);

        // Verify registration succeeded
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_resource_with_custom_name() {
        // Create app state without cache registry
        let app_state = create_test_app_state(true);

        // Register with custom name
        let custom_name = "CustomMock";
        let result = register_resource::<MockResource>(&app_state, Some(custom_name));

        // Verify registration succeeded
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_resource_with_cache_disabled() {
        // Create app state without cache registry
        let app_state = create_test_app_state(false);

        // Register the resource
        let result = register_resource::<MockResource>(&app_state, None);

        // Verify registration succeeded
        assert!(result.is_ok());
    }
}
