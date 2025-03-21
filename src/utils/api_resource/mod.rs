//! # API Resource Abstraction
//!
//! This module provides a high-level abstraction for API resources that
//! handles common concerns like caching, retries, and error handling.
//!
//! See the [README.md](./README.md) for detailed usage examples and guidelines.

mod core;
mod registry;
#[cfg(test)]
mod tests;

// Re-export public items
pub use core::{ApiHandlerOptions, ApiResource, create_api_handler};
pub use registry::*;

use crate::core::router::AppState;
use crate::cache::cache_manager::register_resource_cache;
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
