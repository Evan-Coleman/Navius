use axum::Router;
use metrics_exporter_prometheus::PrometheusHandle;
use reqwest::Client;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;

use crate::core::{
    auth::TokenClient,
    cache::cache_manager::CacheRegistry,
    config::app_config::AppConfig,
    services::{Lifecycle, Service, ServiceRegistry},
    utils::api_resource::ApiResourceRegistry,
};

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
    pub token_client: Option<Arc<dyn TokenClient>>,

    /// Metrics handler for prometheus metrics
    pub metrics_handle: Option<PrometheusHandle>,

    /// API resource registry
    pub resource_registry: Option<Arc<ApiResourceRegistry>>,

    /// Service registry for dependency injection
    pub service_registry: Arc<ServiceRegistry>,

    /// Lifecycle hooks for application startup and shutdown
    lifecycle_hooks:
        HashMap<String, Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>,
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
            lifecycle_hooks: HashMap::new(),
        }
    }
}

impl AppState {
    /// Create a new AppState builder
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::new()
    }

    /// Get a service from the service registry by type
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.service_registry.get::<T>()
    }

    /// Add a startup hook that will be executed when the application starts
    pub fn add_startup_hook<F, Fut>(&mut self, name: &str, f: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let boxed_fn = Box::new(move || Box::pin(f()) as Pin<Box<dyn Future<Output = ()> + Send>>);
        self.lifecycle_hooks
            .insert(format!("startup:{}", name), boxed_fn);
    }

    /// Add a shutdown hook that will be executed when the application shuts down
    pub fn add_shutdown_hook<F, Fut>(&mut self, name: &str, f: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let boxed_fn = Box::new(move || Box::pin(f()) as Pin<Box<dyn Future<Output = ()> + Send>>);
        self.lifecycle_hooks
            .insert(format!("shutdown:{}", name), boxed_fn);
    }

    /// Run all startup hooks
    pub async fn run_startup_hooks(&self) {
        for (key, hook) in &self.lifecycle_hooks {
            if key.starts_with("startup:") {
                let future = hook();
                future.await;
            }
        }
    }

    /// Run all shutdown hooks
    pub async fn run_shutdown_hooks(&self) {
        for (key, hook) in &self.lifecycle_hooks {
            if key.starts_with("shutdown:") {
                let future = hook();
                future.await;
            }
        }
    }
}

/// Builder for AppState with fluent API
pub struct AppStateBuilder {
    config: Option<AppConfig>,
    client: Option<Client>,
    cache_registry: Option<Arc<CacheRegistry>>,
    token_client: Option<Arc<dyn TokenClient>>,
    metrics_handle: Option<PrometheusHandle>,
    resource_registry: Option<Arc<ApiResourceRegistry>>,
    service_registry: ServiceRegistry,
    lifecycle_hooks:
        HashMap<String, Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>,
}

impl AppStateBuilder {
    /// Create a new AppStateBuilder with default values
    pub fn new() -> Self {
        Self {
            config: None,
            client: None,
            cache_registry: None,
            token_client: None,
            metrics_handle: None,
            resource_registry: None,
            service_registry: ServiceRegistry::new(),
            lifecycle_hooks: HashMap::new(),
        }
    }

    /// Set the application configuration
    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Add an HTTP client for external API calls
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Add a cache registry for resource caching
    pub fn with_cache(mut self, cache: Arc<CacheRegistry>) -> Self {
        self.cache_registry = Some(cache);
        self
    }

    /// Add a token client for authentication
    pub fn with_token_client(mut self, token_client: Arc<dyn TokenClient>) -> Self {
        self.token_client = Some(token_client);
        self
    }

    /// Add a metrics handle for prometheus metrics
    pub fn with_metrics(mut self, metrics_handle: PrometheusHandle) -> Self {
        self.metrics_handle = Some(metrics_handle);
        self
    }

    /// Add a resource registry for API resources
    pub fn with_resource_registry(mut self, registry: Arc<ApiResourceRegistry>) -> Self {
        self.resource_registry = Some(registry);
        self
    }

    /// Register a service in the service registry
    pub fn register_service<T: Service>(mut self, service: T) -> Self {
        self.service_registry.register(service);
        self
    }

    /// Register a service using a trait as the key
    pub fn register_service_as<T: Service, K: 'static>(mut self, service: T) -> Self {
        self.service_registry.register_as::<T, K>(service);
        self
    }

    /// Add a startup hook that will be executed when the application starts
    pub fn with_startup_hook<F, Fut>(mut self, name: &str, f: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let boxed_fn = Box::new(move || Box::pin(f()) as Pin<Box<dyn Future<Output = ()> + Send>>);
        self.lifecycle_hooks
            .insert(format!("startup:{}", name), boxed_fn);
        self
    }

    /// Add a shutdown hook that will be executed when the application shuts down
    pub fn with_shutdown_hook<F, Fut>(mut self, name: &str, f: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let boxed_fn = Box::new(move || Box::pin(f()) as Pin<Box<dyn Future<Output = ()> + Send>>);
        self.lifecycle_hooks
            .insert(format!("shutdown:{}", name), boxed_fn);
        self
    }

    /// Build the AppState
    pub fn build(self) -> AppState {
        AppState {
            config: self.config.unwrap_or_default(),
            start_time: SystemTime::now(),
            client: self.client,
            cache_registry: self.cache_registry,
            token_client: self.token_client,
            metrics_handle: self.metrics_handle,
            resource_registry: self.resource_registry,
            service_registry: Arc::new(self.service_registry),
            lifecycle_hooks: self.lifecycle_hooks,
        }
    }

    /// Build the AppState and create a router
    pub fn build_router(self) -> Router {
        let app_state = Arc::new(self.build());

        // Delegate route creation to CoreRouter
        crate::core::router::core_router::CoreRouter::create_core_routes(app_state)
    }
}

/// Initialize the application state with default configuration
pub fn init_app_state() -> AppState {
    AppState::default()
}

/// Helper function to create a Spring Boot-like application
pub fn create_application() -> AppStateBuilder {
    AppStateBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestService {
        name: String,
    }

    impl Service for TestService {}

    #[test]
    fn test_app_state_builder() {
        // Create a basic AppState with the builder
        let state = AppState::builder()
            .with_config(AppConfig::default())
            .register_service(TestService {
                name: "test".to_string(),
            })
            .build();

        // Check that the service was registered
        let service = state.get::<TestService>();
        assert!(service.is_some());
        assert_eq!(service.unwrap().name, "test");
    }

    #[tokio::test]
    async fn test_lifecycle_hooks() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        // Create atomic flags to track hook execution
        let startup_called = Arc::new(AtomicBool::new(false));
        let shutdown_called = Arc::new(AtomicBool::new(false));

        // Clones for the closures
        let startup_flag = startup_called.clone();
        let shutdown_flag = shutdown_called.clone();

        // Create AppState with lifecycle hooks
        let state = AppState::builder()
            .with_startup_hook("test", move || {
                let flag = startup_flag.clone();
                async move {
                    flag.store(true, Ordering::SeqCst);
                }
            })
            .with_shutdown_hook("test", move || {
                let flag = shutdown_flag.clone();
                async move {
                    flag.store(true, Ordering::SeqCst);
                }
            })
            .build();

        // Initially, hooks should not have been called
        assert!(!startup_called.load(Ordering::SeqCst));
        assert!(!shutdown_called.load(Ordering::SeqCst));

        // Run startup hooks
        state.run_startup_hooks().await;
        assert!(startup_called.load(Ordering::SeqCst));
        assert!(!shutdown_called.load(Ordering::SeqCst));

        // Run shutdown hooks
        state.run_shutdown_hooks().await;
        assert!(startup_called.load(Ordering::SeqCst));
        assert!(shutdown_called.load(Ordering::SeqCst));
    }
}
