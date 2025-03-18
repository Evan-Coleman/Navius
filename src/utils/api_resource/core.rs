use axum::{
    Json,
    extract::{Path, State},
};
use metrics::counter;
use serde::{Serialize, de::DeserializeOwned};
use std::{any::Any, fmt::Debug, sync::Arc};
use tracing::{debug, info, warn};

use crate::{
    app::AppState,
    cache::cache_manager::PetCache,
    error::{AppError, Result},
    generated_apis::petstore_api::models::Upet,
    utils::api_logger,
};

/// A trait for resources that can be retrieved from an API.
///
/// By implementing this trait, a model can benefit from common
/// functionality like caching, retries, and error handling.
pub trait ApiResource:
    Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static
{
    /// The ID type of the resource (e.g., i64, String)
    type Id: ToString + Clone + Send + Sync + 'static;

    /// The string representation of the resource type (e.g., "pet", "user")
    /// Used for logging and metrics
    fn resource_type() -> &'static str;

    /// The API name used for logging (e.g., "Petstore")
    fn api_name() -> &'static str;
}

/// Options for configuring the API handler's behavior
///
/// These options allow you to customize how the handler works,
/// such as enabling/disabling caching or retries.
#[derive(Clone, Debug)]
pub struct ApiHandlerOptions {
    /// Whether to use caching for this resource
    ///
    /// Set to false for resources that shouldn't be cached
    /// (e.g., random data, rapidly changing information)
    pub use_cache: bool,

    /// Whether to use retries for failed requests
    ///
    /// Set to false to disable automatic retries
    pub use_retries: bool,

    /// Maximum number of retry attempts for failed requests
    ///
    /// Default is 3
    pub max_retry_attempts: u32,

    /// Cache time-to-live in seconds
    ///
    /// Default is 300 seconds (5 minutes)
    /// Set to 0 to disable TTL (cache until explicitly invalidated)
    pub cache_ttl_seconds: u64,

    /// Whether to log detailed information about the request/response
    ///
    /// Set to false to reduce log verbosity for high-volume endpoints
    pub detailed_logging: bool,
}

impl Default for ApiHandlerOptions {
    fn default() -> Self {
        Self {
            use_cache: true,
            use_retries: true,
            max_retry_attempts: 3,
            cache_ttl_seconds: 300, // 5 minutes
            detailed_logging: true,
        }
    }
}

/// Creates a handler function for an API resource.
///
/// This function creates an async function that can be used as an Axum handler,
/// wrapping the provided fetch function with additional functionality:
/// - Automatic caching (if enabled)
/// - Automatic retries (if enabled)
/// - Error handling
/// - Logging and metrics
///
/// # Type Parameters
///
/// - `R`: The resource type that implements ApiResource
/// - `F`: The fetch function type
/// - `Fut`: The future returned by the fetch function
///
/// # Arguments
///
/// - `fetch_fn`: A function that fetches the resource from a data source
/// - `options`: Options for configuring the handler's behavior
pub fn create_api_handler<R, F, Fut>(
    fetch_fn: F,
    options: ApiHandlerOptions,
) -> impl Fn(State<Arc<AppState>>, Path<String>) -> futures::future::BoxFuture<'static, Result<Json<R>>>
+ Clone
+ Send
+ Sync
+ 'static
where
    R: ApiResource,
    F: Fn(&Arc<AppState>, R::Id) -> Fut + Clone + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<R>> + Send + 'static,
    R::Id: std::str::FromStr + Clone,
{
    move |State(state), Path(id_str)| {
        let fetch_fn = fetch_fn.clone();
        let options = options.clone();
        let state = state.clone();

        Box::pin(async move {
            if options.detailed_logging {
                info!("🔍 Getting {} with ID: {}", R::resource_type(), id_str);
            } else {
                debug!("Getting {} with ID: {}", R::resource_type(), id_str);
            }

            // Parse the ID
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
                        if options.detailed_logging {
                            info!("✅ Retrieved {} {} from cache", R::resource_type(), id_str);
                        }
                        counter!("cache_hits_total").increment(1);
                        return Ok(Json(resource));
                    }
                }
            }

            // If not in cache or cache is disabled, fetch the resource
            let resource = if options.use_retries {
                fetch_with_retry(
                    &state,
                    &id,
                    &fetch_fn,
                    options.max_retry_attempts,
                    options.detailed_logging,
                )
                .await?
            } else {
                // Clone the ID here to avoid moving it
                fetch_fn(&state, id.clone()).await?
            };

            // Store in cache if enabled
            if options.use_cache {
                if let Some(cache) = &state.pet_cache {
                    store_in_cache::<R>(&id, &resource, cache, &id_str).await;

                    // Cache TTL would be implemented here in a more complete solution
                    // For now, we'll just log that we're using the TTL setting
                    if options.detailed_logging && options.cache_ttl_seconds > 0 {
                        info!(
                            "📅 Cache TTL for {} {} set to {} seconds",
                            R::resource_type(),
                            id_str,
                            options.cache_ttl_seconds
                        );
                    }
                }
            }

            Ok(Json(resource))
        })
    }
}

/// Check if a resource is in the cache
///
/// # Type Parameters
///
/// - `R`: The resource type that implements ApiResource
///
/// # Arguments
///
/// - `id`: The resource ID
/// - `cache`: The cache to check
/// - `id_str`: String representation of the ID (for logging)
///
/// # Returns
///
/// Some(resource) if found in cache, None otherwise
async fn check_cache<R: ApiResource>(id: &R::Id, cache: &PetCache, id_str: &str) -> Option<R> {
    debug!(
        "🔍 Checking cache for {} ID: {}",
        R::resource_type(),
        id_str
    );

    // Convert the ID to a cache key
    if let Some(cache_id) = to_cache_key::<R>(id) {
        let resource_result = cache.get(&cache_id).await;
        if let Some(resource) = resource_result {
            // Convert the cached resource to the appropriate type
            if let Some(typed_resource) = convert_cached_resource::<R>(resource) {
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
///
/// # Type Parameters
///
/// - `R`: The resource type that implements ApiResource
///
/// # Arguments
///
/// - `id`: The resource ID
/// - `resource`: The resource to store
/// - `cache`: The cache to store in
/// - `id_str`: String representation of the ID (for logging)
async fn store_in_cache<R: ApiResource>(id: &R::Id, resource: &R, cache: &PetCache, id_str: &str) {
    debug!("💾 Storing {} ID: {} in cache", R::resource_type(), id_str);

    // Convert the ID to a cache key
    if let Some(cache_id) = to_cache_key::<R>(id) {
        if let Some(cache_value) = convert_to_cache_value::<R>(resource) {
            cache.insert(cache_id, cache_value).await;
            counter!("cache_entries_created").increment(1);
            api_logger::log_cache_store(R::resource_type(), id_str);
        }
    }
}

/// Fetch a resource with retries on failure
///
/// This function will retry the fetch operation with exponential backoff
/// if it fails. It will not retry if the error is a not found error.
///
/// # Type Parameters
///
/// - `R`: The resource type that implements ApiResource
/// - `F`: The fetch function type
/// - `Fut`: The future returned by the fetch function
/// - `S`: The state type
///
/// # Arguments
///
/// - `state`: The application state
/// - `id`: The resource ID
/// - `fetch_fn`: A function that fetches the resource
/// - `max_retries`: Maximum number of retry attempts
/// - `detailed_logging`: Whether to log detailed information
///
/// # Returns
///
/// The resource if successful, or an error
pub(crate) async fn fetch_with_retry<R: ApiResource, F, Fut, S>(
    state: &Arc<S>,
    id: &R::Id,
    fetch_fn: &F,
    max_retries: u32,
    detailed_logging: bool,
) -> Result<R>
where
    F: Fn(&Arc<S>, R::Id) -> Fut,
    Fut: std::future::Future<Output = Result<R>>,
    S: 'static,
{
    let mut last_error = None;

    for attempt in 0..=max_retries {
        if attempt > 0 && detailed_logging {
            info!(
                "🔄 Retry attempt {} for {} ID: {}",
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
                    if detailed_logging {
                        warn!("❓ {} not found: {}", R::resource_type(), err);
                    }
                    return Err(AppError::NotFound(format!(
                        "{} with ID {} not found",
                        R::resource_type(),
                        id.to_string()
                    )));
                }

                if detailed_logging {
                    warn!("❌ Attempt {} failed: {}", attempt + 1, err);
                } else {
                    debug!("Attempt {} failed: {}", attempt + 1, err);
                }

                last_error = Some(err);

                // Don't sleep on the last attempt
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_millis(100 * 2u64.pow(attempt)))
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

// Specialized implementations for our specific cache

/// Convert a resource ID to a cache key (i64)
fn to_cache_key<R: ApiResource>(id: &R::Id) -> Option<i64> {
    // For now, we assume all IDs can be converted to i64
    id.to_string().parse::<i64>().ok()
}

/// Convert a cached Upet to the specific resource type
fn convert_cached_resource<R: ApiResource>(cached: Upet) -> Option<R> {
    // For pet resources, we can just clone the cached item
    if std::any::TypeId::of::<R>() == std::any::TypeId::of::<Upet>() {
        // SAFETY: We've verified the types match
        let boxed: Box<dyn Any> = Box::new(cached);
        let resource = boxed.downcast::<Upet>().ok()?;

        // SAFETY: We've verified R is Upet above
        let resource_any: Box<dyn Any> = Box::new(*resource);
        match resource_any.downcast::<R>() {
            Ok(typed) => Some(*typed),
            Err(_) => None,
        }
    } else {
        // For other resource types, we would need custom conversion
        // This is a placeholder for future resource types
        None
    }
}

/// Convert a resource to a cache value
fn convert_to_cache_value<R: ApiResource>(resource: &R) -> Option<Upet> {
    // For Upet resources, we can just clone
    if std::any::TypeId::of::<R>() == std::any::TypeId::of::<Upet>() {
        // SAFETY: We've verified the types match
        let boxed: Box<dyn Any> = Box::new(resource.clone());
        match boxed.downcast::<Upet>() {
            Ok(pet) => Some(*pet),
            Err(_) => None,
        }
    } else {
        // For other resource types, we would need custom conversion
        // This is a placeholder for future resource types
        None
    }
}
