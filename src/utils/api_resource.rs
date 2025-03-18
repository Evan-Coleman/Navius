use axum::{
    Json,
    extract::{Path, State},
};
use metrics::counter;
use serde::{Serialize, de::DeserializeOwned};
use std::{fmt::Debug, sync::Arc};
use tracing::{debug, info, warn};

use crate::{
    app::AppState,
    cache::cache_manager::PetCache,
    error::{AppError, Result},
    utils::api_logger,
};

/// Trait for API resources that can be retrieved by ID
pub trait ApiResource:
    Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static
{
    /// The ID type of the resource
    type Id: ToString + Clone + Send + Sync + 'static;

    /// The string representation of the resource type (e.g., "pet", "user")
    fn resource_type() -> &'static str;

    /// The API name used for logging (e.g., "Petstore")
    fn api_name() -> &'static str;
}

/// Options for the API handler
pub struct ApiHandlerOptions {
    /// Whether to use caching
    pub use_cache: bool,
    /// Whether to use retries
    pub use_retries: bool,
}

impl Default for ApiHandlerOptions {
    fn default() -> Self {
        Self {
            use_cache: true,
            use_retries: true,
        }
    }
}

/// Creates a handler function that handles caching, retries, and error handling
pub fn create_api_handler<R, F, Fut>(
    fetch_fn: F,
    options: ApiHandlerOptions,
) -> impl Fn(
    State<Arc<AppState>>,
    Path<String>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Json<R>>> + Send>>
where
    R: ApiResource,
    F: Fn(&Arc<AppState>, R::Id) -> Fut + Clone + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<R>> + Send + 'static,
{
    move |State(state): State<Arc<AppState>>, Path(id_str): Path<String>| {
        let fetch_fn = fetch_fn.clone();
        let options = options.clone();

        Box::pin(async move {
            info!("🔍 Getting {} with ID: {}", R::resource_type(), id_str);

            // Parse the ID - this assumes a numeric ID, modify as needed
            let id = id_str.parse().map_err(|_| {
                AppError::BadRequest(format!(
                    "Invalid {} ID format: {}",
                    R::resource_type(),
                    id_str
                ))
            })?;

            // Check cache first if enabled
            if options.use_cache {
                if let Some(cache) = &state.pet_cache {
                    let resource_result = check_cache::<R>(&id, cache, &id_str).await;
                    if let Some(resource) = resource_result {
                        return Ok(Json(resource));
                    }
                }
            }

            // If not in cache or cache is disabled, fetch the resource
            let resource = if options.use_retries {
                fetch_with_retry(&state, id, &fetch_fn).await?
            } else {
                fetch_fn(&state, id).await?
            };

            // Store in cache if enabled
            if options.use_cache {
                if let Some(cache) = &state.pet_cache {
                    store_in_cache::<R>(&id, &resource, cache, &id_str).await;
                }
            }

            Ok(Json(resource))
        })
    }
}

/// Check if a resource is in the cache
async fn check_cache<R: ApiResource>(id: &R::Id, cache: &PetCache, id_str: &str) -> Option<R> {
    debug!(
        "🔍 Checking cache for {} ID: {}",
        R::resource_type(),
        id_str
    );

    // Note: This assumes the cache key is an i64, which might need to be adapted
    // for different resource types
    if let Some(cache_id) = to_cache_key::<R>(id) {
        let resource_result = cache.get(&cache_id).await;
        if let Some(resource) = resource_result {
            // Type conversion - this will need to be handled appropriately for different resources
            if let Ok(typed_resource) = convert_cached_resource::<R>(resource) {
                counter!("pet_cache_hits_total").increment(1);
                api_logger::log_cache_hit(R::resource_type(), id_str);
                return Some(typed_resource);
            }
        }
    }

    counter!("pet_cache_misses_total").increment(1);
    api_logger::log_cache_miss(R::resource_type(), id_str);
    None
}

/// Store a resource in the cache
async fn store_in_cache<R: ApiResource>(id: &R::Id, resource: &R, cache: &PetCache, id_str: &str) {
    debug!("💾 Storing {} ID: {} in cache", R::resource_type(), id_str);

    // Note: This assumes the cache stores Upet objects, which might need to be adapted
    // for different resource types
    if let Some(cache_id) = to_cache_key::<R>(id) {
        if let Ok(cache_value) = convert_to_cache_value::<R>(resource) {
            cache.insert(cache_id, cache_value).await;
            counter!("cache_entries_created").increment(1);
            api_logger::log_cache_store(R::resource_type(), id_str);
        }
    }
}

/// Fetch a resource with retry logic
async fn fetch_with_retry<R: ApiResource, F, Fut>(
    state: &Arc<AppState>,
    id: R::Id,
    fetch_fn: &F,
) -> Result<R>
where
    F: Fn(&Arc<AppState>, R::Id) -> Fut,
    Fut: std::future::Future<Output = Result<R>>,
{
    let max_retries = state.config.server.max_retries;
    let mut last_error = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            info!(
                "Retry attempt {} for {} ID: {}",
                attempt,
                R::resource_type(),
                id.to_string()
            );
        }

        match fetch_fn(state, id.clone()).await {
            Ok(resource) => return Ok(resource),
            Err(err) => {
                // Don't log attempt number or retry on 404 Not Found errors
                if err.to_string().contains("not found (HTTP 404)") {
                    warn!("❓ {} not found: {}", R::resource_type(), err);
                    return Err(AppError::NotFound(format!(
                        "{} with ID {} not found",
                        R::resource_type(),
                        id.to_string()
                    )));
                }

                warn!("Attempt {} failed: {}", attempt + 1, err);
                last_error = Some(err);

                // Don't sleep on the last attempt
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_millis(
                        100 * 2u64.pow(attempt as u32),
                    ))
                    .await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        AppError::InternalError(format!(
            "Unknown error fetching {} with ID {}",
            R::resource_type(),
            id.to_string()
        ))
    }))
}

// These helper functions would need to be implemented according to your specific cache implementation
// They are placeholders that you would need to adapt to your actual types

/// Convert a resource ID to a cache key
fn to_cache_key<R: ApiResource>(id: &R::Id) -> Option<i64> {
    // This is a placeholder - implement based on your actual ID type
    // For string IDs, this might parse the string to an i64
    // For numeric IDs, this might be a direct conversion
    // This implementation assumes R::Id can be converted to i64 somehow
    None
}

/// Convert a cached value to the specific resource type
fn convert_cached_resource<R: ApiResource>(cached: impl Any) -> Result<R, ()> {
    // This is a placeholder - implement based on your cache value type
    // For example, if your cache stores JsonValue, this would deserialize it
    // If your cache stores the same type as R, this might be a direct cast
    Err(())
}

/// Convert a resource to a cache value
fn convert_to_cache_value<R: ApiResource>(resource: &R) -> Result<impl Any, ()> {
    // This is a placeholder - implement based on your cache value type
    // For example, if your cache stores JsonValue, this would serialize the resource
    // If your cache stores the same type as R, this might be a direct clone
    Err(())
}

use std::any::Any;

// You'll need to implement these placeholder functions based on your specific cache implementation
// For example:

fn to_cache_key_impl<T: ToString>(id: &T) -> Option<i64> {
    id.to_string().parse::<i64>().ok()
}

fn convert_cached_resource_impl<T: Clone>(cached: T) -> Result<T, ()> {
    Ok(cached.clone())
}

fn convert_to_cache_value_impl<T: Clone>(resource: &T) -> Result<T, ()> {
    Ok(resource.clone())
}
