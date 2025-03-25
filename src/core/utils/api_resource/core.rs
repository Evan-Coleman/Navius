use axum::{
    Json,
    extract::{Path, State},
};
use metrics::{counter, gauge};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{
    any::Any,
    fmt::{Debug, Display},
    sync::Arc,
};
use tracing::{debug, info, warn};

use crate::{
    core::{router::AppState, utils::api_logger},
    error::{AppError, Result},
    utils::api_resource::ApiResourceRegistry,
};

use crate::core::models::DependencyStatus;

/// Trait for API resources that can be cached and managed
pub trait ApiResource: Clone + Send + Sync + 'static {
    /// The type used for resource identification
    type Id: Display + Clone + Send + Sync;

    /// The string representation of the resource type (e.g., "user", "account")
    fn resource_type() -> &'static str;

    /// The API name used for logging (e.g., "UserService", "AccountAPI")
    fn api_name() -> &'static str;
}

/// Type alias for boxed future results
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// Register a resource type with the registry
pub fn register_resource<R: ApiResource + 'static>(
    registry: &Arc<ApiResourceRegistry>,
) -> std::result::Result<(), String> {
    // Create a simple health check function that just returns UP status
    let health_check = |_state: &Arc<AppState>| {
        Box::pin(async {
            DependencyStatus {
                name: format!("{} ({})", R::api_name(), R::resource_type()),
                status: "UP".to_string(),
                details: None,
            }
        }) as futures::future::BoxFuture<'static, DependencyStatus>
    };

    registry.register::<R, _>(health_check)
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

            // Try to parse the ID from the string
            let id: R::Id = id_str.parse().map_err(|_| {
                AppError::BadRequest(format!(
                    "Invalid ID format for {}: {}",
                    R::resource_type(),
                    id_str
                ))
            })?;

            // Check cache first if enabled
            if options.use_cache {
                // Use the resource type from the ApiResource trait
                let resource_type = R::resource_type();

                // Convert ID to string for cache key
                let cache_key = id.to_string();

                // Try to fetch from cache using the generic get_or_fetch function
                let fetch_closure = || async {
                    // Call the original fetch function and convert AppError to String
                    fetch_fn(&state, id.clone())
                        .await
                        .map_err(|e| e.to_string())
                };

                match crate::core::cache::get_or_fetch::<R, _, _>(
                    &state.cache_registry,
                    resource_type,
                    &cache_key,
                    fetch_closure,
                )
                .await
                {
                    Ok(resource) => return Ok(Json(resource)),
                    Err(err) => {
                        // If cache fails, fall back to direct fetch
                        tracing::warn!(
                            "Cache fetch failed for {}: {}, falling back to direct fetch",
                            resource_type,
                            err
                        );
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

            // Store in new cache registry if enabled
            if options.use_cache {
                // Get the resource type and create a cache key
                let resource_type = R::resource_type();
                let cache_key = id.to_string();

                // Try to store the resource in the registry
                let store_result = match crate::core::cache::get_resource_cache::<R>(
                    &state.cache_registry,
                    resource_type,
                ) {
                    Some(cache) => {
                        // Resource type is registered, store in cache
                        if options.detailed_logging {
                            info!("➕ Storing {} {} in registry cache", resource_type, id_str);
                        }

                        // Create a closure to store the resource
                        let store_fn = || async {
                            // Store directly in the cache
                            cache
                                .cache
                                .insert(cache_key.clone(), resource.clone())
                                .await;

                            // Increment counters
                            counter!("cache_entries_created", "resource_type" => resource_type.to_string()).increment(1);
                            let new_count = cache
                                .active_entries
                                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                                + 1;

                            // Update metrics
                            let current_size = cache.cache.entry_count();
                            gauge!("cache_current_size", "resource_type" => resource_type.to_string()).set(current_size as f64);
                            gauge!("cache_active_entries", "resource_type" => resource_type.to_string()).set(new_count as f64);

                            if options.detailed_logging {
                                debug!(
                                    "➕ Added {} ID: {} to registry cache (size: {}, active: {})",
                                    resource_type, id_str, current_size, new_count
                                );
                            }

                            Ok(())
                        };

                        // Execute the store function
                        store_fn().await
                    }
                    None => {
                        // Resource type is not registered, try to register it
                        if options.detailed_logging {
                            info!(
                                "🔍 No cache found for {} in registry, attempting to register",
                                resource_type
                            );
                        }

                        // Try to register the resource type
                        match crate::utils::api_resource::register_resource::<R>(&state, None) {
                            Ok(_) => {
                                // Successfully registered, now try to get the cache again
                                match crate::core::cache::get_resource_cache::<R>(
                                    &state.cache_registry,
                                    resource_type,
                                ) {
                                    Some(new_cache) => {
                                        // Store in the newly registered cache
                                        if options.detailed_logging {
                                            info!(
                                                "➕ Storing {} {} in newly registered cache",
                                                resource_type, id_str
                                            );
                                        }

                                        // Store directly in the cache
                                        new_cache
                                            .cache
                                            .insert(cache_key.clone(), resource.clone())
                                            .await;

                                        // Increment counters
                                        counter!("cache_entries_created", "resource_type" => resource_type.to_string()).increment(1);
                                        let new_count = new_cache
                                            .active_entries
                                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                                            + 1;

                                        // Update metrics
                                        let current_size = new_cache.cache.entry_count();
                                        gauge!("cache_current_size", "resource_type" => resource_type.to_string()).set(current_size as f64);
                                        gauge!("cache_active_entries", "resource_type" => resource_type.to_string()).set(new_count as f64);

                                        if options.detailed_logging {
                                            debug!(
                                                "➕ Added {} ID: {} to newly registered cache (size: {}, active: {})",
                                                resource_type, id_str, current_size, new_count
                                            );
                                        }

                                        Ok(())
                                    }
                                    None => {
                                        // Still can't get the cache, log the error
                                        if options.detailed_logging {
                                            warn!(
                                                "❌ Failed to get cache for {} after registration",
                                                resource_type
                                            );
                                        }
                                        Err("Failed to get cache after registration".to_string())
                                    }
                                }
                            }
                            Err(e) => {
                                // Failed to register the resource type
                                if options.detailed_logging {
                                    warn!(
                                        "❌ Failed to register {} in cache registry: {}",
                                        resource_type, e
                                    );
                                }
                                Err(e)
                            }
                        }
                    }
                };

                // Log any errors but continue - we don't want to fail the request if caching fails
                if let Err(e) = store_result {
                    debug!(
                        "❌ Failed to store {} {} in registry cache: {}",
                        resource_type, id_str, e
                    );
                }

                if options.detailed_logging {
                    info!(
                        "📅 Cache TTL for {} {} set to {} seconds (in registry)",
                        resource_type, id_str, options.cache_ttl_seconds
                    );
                }
            }

            Ok(Json(resource))
        })
    }
}

// Static counters for cache hits and misses
static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);

/// Calculate cache hit ratio as a percentage
fn cache_hit_ratio() -> f64 {
    let hits = CACHE_HITS.load(Ordering::Relaxed);
    let misses = CACHE_MISSES.load(Ordering::Relaxed);

    let total = hits + misses;
    if total > 0 {
        (hits as f64 / total as f64) * 100.0
    } else {
        0.0
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
pub async fn fetch_with_retry<R: ApiResource, F, Fut, S>(
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
        AppError::InternalServerError(format!(
            "Could not generate URL for resource {}: {}",
            R::resource_type(),
            id.to_string()
        ))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, Response, StatusCode};
    use axum::routing::get;
    use futures::future::BoxFuture;
    use metrics_exporter_prometheus::PrometheusBuilder;
    use serde::{Deserialize, Serialize};
    use std::sync::atomic::AtomicUsize;
    use tower::{BoxError, Service, ServiceExt};

    // Mock API resource for testing
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct MockResource {
        id: i64,
        name: String,
        status: String,
    }

    impl ApiResource for MockResource {
        type Id = i64;

        fn resource_type() -> &'static str {
            "mock_resource"
        }

        fn api_name() -> &'static str {
            "MockService"
        }
    }

    #[test]
    fn test_api_handler_options_default() {
        let options = ApiHandlerOptions::default();

        assert!(options.use_cache);
        assert!(options.use_retries);
        assert_eq!(options.max_retry_attempts, 3);
        assert_eq!(options.cache_ttl_seconds, 300);
        assert!(options.detailed_logging);
    }

    #[test]
    fn test_api_handler_options_custom() {
        let options = ApiHandlerOptions {
            use_cache: false,
            use_retries: false,
            max_retry_attempts: 5,
            cache_ttl_seconds: 600,
            detailed_logging: false,
        };

        assert!(!options.use_cache);
        assert!(!options.use_retries);
        assert_eq!(options.max_retry_attempts, 5);
        assert_eq!(options.cache_ttl_seconds, 600);
        assert!(!options.detailed_logging);
    }

    #[tokio::test]
    async fn test_create_api_handler() {
        // Create sample app state
        let metrics_recorder = PrometheusBuilder::new().build_recorder();
        let metrics_handle = metrics_recorder.handle();

        let app_state = Arc::new(AppState {
            config: crate::core::config::app_config::AppConfig::default(),
            start_time: std::time::SystemTime::now(),
            cache_registry: Some(Arc::new(CacheRegistry::new())),
            client: Some(reqwest::Client::new()),
            db_pool: None,
            token_client: Some(Arc::new(MockTokenClient::default())),
            metrics_handle: Some(metrics_handle),
            resource_registry: Some(Arc::new(ApiResourceRegistry::new())),
            service_registry: Arc::new(ServiceRegistry::new()),
        });

        // Create a counter to track how many times the fetch function is called
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        // Create a fetch function that returns a mock resource
        let fetch_fn = move |_state: &Arc<AppState>, id: i64| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            let resource = MockResource {
                id,
                name: format!("Resource {}", id),
                status: "available".to_string(),
            };
            async move { Ok(resource) }
        };

        // Create the handler with default options
        let handler = create_api_handler(fetch_fn, ApiHandlerOptions::default());

        // Create a router with the handler
        let app = axum::Router::new()
            .route(
                "/resources/{id}",
                get(
                    |state: State<Arc<AppState>>, path: Path<String>| async move {
                        handler(state, path).await
                    },
                ),
            )
            .with_state(app_state);

        // Create a test request
        let request = Request::builder()
            .uri("/resources/123")
            .body(Body::empty())
            .unwrap();

        // Process the request
        let response = app.oneshot(request).await.unwrap();

        // Check the response
        assert_eq!(response.status(), StatusCode::OK);

        // Get the response body
        let body = response.into_body();
        let body_bytes = hyper::body::to_bytes(body).await.unwrap();
        let resource: MockResource = serde_json::from_slice(&body_bytes).unwrap();

        // Verify the resource data
        assert_eq!(resource.id, 123);
        assert_eq!(resource.name, "Resource 123");
        assert_eq!(resource.status, "available");

        // Verify the fetch function was called exactly once
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_fetch_with_retry_success_first_try() {
        // Create sample app state
        let app_state = Arc::new(AppState {
            config: crate::core::config::app_config::AppConfig::default(),
            start_time: std::time::SystemTime::now(),
            cache_registry: Some(Arc::new(CacheRegistry::new())),
            client: Some(reqwest::Client::new()),
            db_pool: None,
            token_client: Some(Arc::new(MockTokenClient::default())),
            metrics_handle: Some(PrometheusBuilder::new().build_recorder().handle()),
            resource_registry: Some(Arc::new(ApiResourceRegistry::new())),
            service_registry: Arc::new(ServiceRegistry::new()),
        });

        // Create a counter to track how many times the fetch function is called
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        // Create a fetch function that succeeds on first try
        let fetch_fn = move |_: &Arc<AppState>, id: i64| {
            let _count = call_count_clone.fetch_add(1, Ordering::SeqCst);
            let resource = MockResource {
                id,
                name: format!("Resource {}", id),
                status: "available".to_string(),
            };
            async move { Ok(resource) }
        };

        // Call fetch_with_retry
        let result = fetch_with_retry(&app_state, &123, &fetch_fn, 3, true).await;

        // Verify success
        assert!(result.is_ok());
        let resource = result.unwrap();
        assert_eq!(resource.id, 123);

        // Verify the fetch function was called exactly once
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_fetch_with_retry_success_after_retries() {
        // Create sample app state
        let app_state = Arc::new(AppState {
            config: crate::core::config::app_config::AppConfig::default(),
            start_time: std::time::SystemTime::now(),
            cache_registry: Some(Arc::new(CacheRegistry::new())),
            client: Some(reqwest::Client::new()),
            db_pool: None,
            token_client: Some(Arc::new(MockTokenClient::default())),
            metrics_handle: Some(PrometheusBuilder::new().build_recorder().handle()),
            resource_registry: Some(Arc::new(ApiResourceRegistry::new())),
            service_registry: Arc::new(ServiceRegistry::new()),
        });

        // Create a counter to track how many times the fetch function is called
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        // Create a fetch function that fails on first two tries, then succeeds
        let fetch_fn = move |_: &Arc<AppState>, id: i64| {
            let count = call_count_clone.fetch_add(1, Ordering::SeqCst);

            let future: BoxFuture<'static, Result<MockResource>> = if count < 2 {
                // First two calls fail
                Box::pin(async move {
                    Err(AppError::InternalServerError(
                        "Simulated failure for testing".to_string(),
                    ))
                })
            } else {
                // Third call succeeds
                let resource = MockResource {
                    id,
                    name: format!("Resource {}", id),
                    status: "available".to_string(),
                };
                Box::pin(async move { Ok(resource) })
            };

            future
        };

        // Call fetch_with_retry
        let result = fetch_with_retry(&app_state, &123, &fetch_fn, 3, true).await;

        // Verify success
        assert!(result.is_ok());
        let resource = result.unwrap();
        assert_eq!(resource.id, 123);

        // Verify the fetch function was called exactly three times
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_fetch_with_retry_all_failures() {
        // Create sample app state
        let app_state = Arc::new(AppState {
            config: crate::core::config::app_config::AppConfig::default(),
            start_time: std::time::SystemTime::now(),
            cache_registry: Some(Arc::new(CacheRegistry::new())),
            client: Some(reqwest::Client::new()),
            db_pool: None,
            token_client: Some(Arc::new(MockTokenClient::default())),
            metrics_handle: Some(PrometheusBuilder::new().build_recorder().handle()),
            resource_registry: Some(Arc::new(ApiResourceRegistry::new())),
            service_registry: Arc::new(ServiceRegistry::new()),
        });

        // Create a counter to track how many times the fetch function is called
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        // Create a fetch function that always fails
        let fetch_fn = move |_: &Arc<AppState>, _id: i64| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                Err(AppError::InternalServerError(
                    "Simulated failure for testing".to_string(),
                ))
            }
        };

        // Call fetch_with_retry with 2 max retries
        let result: Result<MockResource> =
            fetch_with_retry(&app_state, &123, &fetch_fn, 2, true).await;

        // Verify failure
        assert!(result.is_err());

        // Verify the fetch function was called exactly three times (initial + 2 retries)
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }
}
