//! # API Resource Abstraction
//!
//! This module provides a high-level abstraction for API resources that
//! handles common concerns like caching, retries, and error handling.

pub mod core;
pub mod registry;

#[cfg(feature = "auth")]
use crate::core::auth::TokenClient;
#[cfg(feature = "auth")]
use crate::core::auth::mock::MockTokenClient;
use crate::core::cache;
use crate::core::cache::CacheRegistry;
use crate::core::error::AppError;
use crate::core::models::DependencyStatus;
use crate::core::router::AppState;
use crate::core::services::health::HealthIndicator;

use axum::extract::State;
use axum::http::header::{AUTHORIZATION, HeaderMap};
use axum::middleware::Next;
use axum::response::Response;
use axum::{Json, extract::Request};
use futures::FutureExt;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

// Re-export public items
pub use core::{ApiHandlerOptions, ApiResource, create_api_handler, fetch_with_retry};
pub use registry::*;

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
    let resource_name = resource_type.unwrap_or_else(|| T::resource_type());

    // Check if the cache registry is available
    if let Some(registry) = &state.cache_registry {
        // Register the resource type in the cache registry
        match cache::register_resource_cache::<T>(registry, resource_name) {
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
    } else {
        // Cache registry is not available, just log and continue
        info!(
            "⚠️ Cache registry not available, skipping registration of {}",
            resource_name
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "auth")]
    use crate::core::auth::MockTokenClient;
    use crate::core::cache::CacheRegistry;
    use crate::core::router::ServiceRegistry;
    use std::sync::Arc;

    // ... existing code ...
}
