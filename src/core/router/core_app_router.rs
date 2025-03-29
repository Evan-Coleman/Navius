use axum::{
    Router,
    extract::State,
    routing::{get, post},
};
#[cfg(feature = "metrics")]
use metrics_exporter_prometheus::PrometheusHandle;
use reqwest::Client;
use std::sync::Arc;
use std::time::SystemTime;

#[cfg(feature = "auth")]
use crate::core::auth::TokenClient;
use crate::core::{
    cache::cache_manager::CacheRegistry, config::app_config::AppConfig,
    utils::api_resource::ApiResourceRegistry,
};

/// ServiceRegistry for dependency injection
#[derive(Debug, Default)]
pub struct ServiceRegistry {
    // Internal storage for service instances
    services: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

impl ServiceRegistry {
    /// Create a new empty service registry
    pub fn new() -> Self {
        Self {
            services: std::collections::HashMap::new(),
        }
    }

    /// Register a service in the registry
    pub fn register<T: 'static + Send + Sync>(&mut self, service: T) {
        let type_id = std::any::TypeId::of::<T>();
        self.services.insert(type_id, Box::new(service));
    }

    /// Get a service from the registry
    pub fn get<T: 'static>(&self) -> Option<&T> {
        let type_id = std::any::TypeId::of::<T>();
        self.services
            .get(&type_id)
            .and_then(|s| s.downcast_ref::<T>())
    }

    /// Get the count of registered services
    pub fn service_count(&self) -> usize {
        self.services.len()
    }
}

/// Application state shared across all routes
#[derive(Debug)]
pub struct AppState {
    /// Application configuration
    pub config: AppConfig,

    /// Application start time for uptime tracking
    pub start_time: SystemTime,

    /// HTTP client for external API calls
    pub client: Option<Client>,

    /// Cache registry for resource caching
    pub cache_registry: Option<Arc<CacheRegistry>>,

    /// Token client for authentication
    #[cfg(feature = "auth")]
    pub token_client: Option<Arc<dyn TokenClient>>,

    #[cfg(not(feature = "auth"))]
    pub token_client: Option<()>,

    /// Metrics handler for prometheus metrics
    #[cfg(feature = "metrics")]
    pub metrics_handle: Option<PrometheusHandle>,

    #[cfg(not(feature = "metrics"))]
    pub metrics_handle: Option<()>,

    /// API resource registry
    pub resource_registry: Option<Arc<ApiResourceRegistry>>,

    /// Service registry for dependency injection
    pub service_registry: Arc<ServiceRegistry>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: AppConfig::default(),
            start_time: SystemTime::now(),
            client: None,
            cache_registry: None,
            token_client: None,
            metrics_handle: None,
            resource_registry: None,
            service_registry: Arc::new(ServiceRegistry::new()),
        }
    }
}

/// Builder for creating and configuring the application router
pub struct RouterBuilder {
    /// Application state
    app_state: AppState,

    /// Whether CORS is enabled
    cors_enabled: bool,

    /// Whether metrics are enabled
    metrics_enabled: bool,

    /// Whether authentication is enabled
    auth_enabled: bool,
}

impl RouterBuilder {
    /// Create a new router builder with default settings
    pub fn new() -> Self {
        Self {
            app_state: AppState::default(),
            cors_enabled: true,
            metrics_enabled: true,
            auth_enabled: false,
        }
    }

    /// Set the application configuration
    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.app_state.config = config.clone();
        self.auth_enabled = config.auth.enabled;
        self
    }

    /// Add a cache registry
    pub fn with_cache(mut self, cache: Option<Arc<CacheRegistry>>) -> Self {
        self.app_state.cache_registry = cache;
        self
    }

    /// Add an HTTP client
    pub fn with_client(mut self, client: Option<Client>) -> Self {
        self.app_state.client = client;
        self
    }

    /// Add a token client for authentication
    #[cfg(feature = "auth")]
    pub fn with_token_client(mut self, token_client: Option<Arc<dyn TokenClient>>) -> Self {
        self.app_state.token_client = token_client;
        self
    }

    /// Add a metrics handle
    #[cfg(feature = "metrics")]
    pub fn with_metrics(mut self, metrics_handle: Option<PrometheusHandle>) -> Self {
        self.app_state.metrics_handle = metrics_handle;
        self
    }

    /// Add a resource registry
    pub fn with_resource_registry(mut self, registry: Option<Arc<ApiResourceRegistry>>) -> Self {
        self.app_state.resource_registry = registry;
        self
    }

    /// Register a service with the service registry
    pub fn register_service<T: 'static + Send + Sync>(mut self, service: T) -> Self {
        // Clone the Arc to get mutable access
        let service_registry = Arc::get_mut(&mut self.app_state.service_registry)
            .expect("Failed to get mutable reference to service registry");

        service_registry.register(service);
        self
    }

    /// Enable or disable CORS
    pub fn with_cors(mut self, enabled: bool) -> Self {
        self.cors_enabled = enabled;
        self
    }

    /// Enable or disable metrics
    pub fn with_metrics_enabled(mut self, enabled: bool) -> Self {
        self.metrics_enabled = enabled;
        self
    }

    /// Enable or disable authentication
    pub fn with_auth(mut self, enabled: bool) -> Self {
        self.auth_enabled = enabled;
        self
    }

    /// Build the router with all configured components
    pub fn build(self) -> Router {
        let state = Arc::new(self.app_state);

        // Delegate route creation to CoreRouter
        crate::core::router::core_router::CoreRouter::create_core_routes(state)
    }
}

/// Initialize the application state with default configuration
pub fn init_app_state() -> AppState {
    AppState::default()
}

/// Helper function to create a Spring Boot-like application
pub fn create_application() -> RouterBuilder {
    RouterBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_router_builder_basic() {
        // Create a router with default settings
        let app = RouterBuilder::new().build();

        // Create a request to the health endpoint
        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();

        // Verify the response status
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_service_registry() {
        // Create a new service registry
        let mut registry = ServiceRegistry::new();

        // Add a service
        registry.register::<String>("test service".to_string());

        // Retrieve the service
        let service = registry.get::<String>();

        // Verify the service was retrieved correctly
        assert_eq!(service, Some(&"test service".to_string()));

        // Try to retrieve a service that doesn't exist
        let nonexistent = registry.get::<i32>();

        // Verify the service doesn't exist
        assert_eq!(nonexistent, None);
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();

        // Verify that the default state has the expected values
        assert!(state.client.is_none());
        assert!(state.cache_registry.is_none());
        assert!(state.token_client.is_none());
        assert!(state.metrics_handle.is_none());
        assert!(state.resource_registry.is_none());
    }
}
