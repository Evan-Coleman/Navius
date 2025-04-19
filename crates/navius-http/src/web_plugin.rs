// WebPlugin provides web server capabilities to Navius applications
// This file implements the web plugin abstraction for the Zero Boilerplate Initiative (NC-11)

use async_trait::async_trait;
use axum::{Router, routing::get};
use inventory::collect;
use navius_core::{di::Application, error::Result};
use navius_plugin::plugin::{MessageHandler, PluginLifecycle, PluginLifecycleStage};
use navius_plugin::{BasePlugin, Plugin, PluginConfig, PluginHealth, PluginMetadata, PluginResult};
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::{debug, info};

use crate::server::{
    HttpServerConfig,
    route_discovery::{RouteDiscoveryConfig, RouteRegistry, discover_routes},
};

/// Route registration for automatic route discovery.
#[derive(Debug)]
pub struct RouteRegistration {
    /// Path of the route.
    pub path: &'static str,

    /// HTTP methods supported by the route.
    pub methods: Vec<String>,

    /// Name of the handler function.
    pub handler_name: &'static str,

    /// Route handler function.
    pub handler: fn(Router) -> Router,
}

/// Global registry for routes discovered via attribute macros.
#[derive(Debug)]
pub struct RouteRegistry {
    routes: Vec<RouteRegistration>,
}

inventory::collect!(RouteRegistration);

impl RouteRegistry {
    /// Create a new route registry.
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Register a route.
    pub fn register(&mut self, route: RouteRegistration) {
        self.routes.push(route);
    }

    /// Build a router from the registered routes.
    pub fn build_router(&self) -> Router {
        let mut router = Router::new();

        for route in &self.routes {
            debug!("Registering route: {} ({})", route.path, route.handler_name);
            router = (route.handler)(router);
        }

        router
    }

    /// Gets an iterator over all registered routes.
    pub fn iter(&self) -> impl Iterator<Item = &RouteRegistration> {
        self.routes.iter()
    }
}

/// Global route registry for collecting route registrations.
#[inventory::collect]
pub static ROUTE_REGISTRY: inventory::Registry<RouteRegistration> = inventory::Registry::new();

/// WebPlugin provides web server capabilities to Navius applications
///
/// This plugin allows you to easily add web server functionality to your Navius application
/// with minimal configuration. The plugin handles starting the Axum server, managing the lifecycle,
/// and providing default routes.
///
/// # Examples
///
/// ```rust,no_run
/// use navius_http::WebPlugin;
/// use axum::{Router, routing::get};
///
/// // Create the plugin with default settings
/// let web_plugin = WebPlugin::new();
///
/// // Or customize it with a fluent interface
/// let web_plugin = WebPlugin::new()
///     .with_host("127.0.0.1")
///     .with_port(8080)
///     .with_router(Router::new().route("/hello", get(|| async { "Hello, world!" })));
///
/// // Add it to your Navius application
/// let app = navius_core::app::App::new()
///     .with_plugin(web_plugin)
///     .run();
/// ```
#[derive(Debug)]
pub struct WebPlugin {
    base: BasePlugin,
    host: String,
    port: u16,
    router: Option<Router>,
    lifecycle_stage: PluginLifecycleStage,
}

/// Define a simple application state that contains the application context
#[derive(Clone, Debug)]
pub struct AppState {
    pub app: Arc<Application>,
}

impl WebPlugin {
    /// Create a new WebPlugin with default settings
    pub fn new() -> Self {
        Self {
            base: BasePlugin::new(PluginMetadata::new(
                "web",
                "0.1.0",
                "Web Server Plugin",
                "Provides web server capabilities to Navius applications",
                "Navius Team",
            )),
            host: "127.0.0.1".to_string(),
            port: 3000,
            router: None,
            lifecycle_stage: PluginLifecycleStage::Created,
        }
    }

    /// Set the host address
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Set the port number
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Get the host address
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Get the port number
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Set the router for the web server
    pub fn with_router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    /// Build a router from registered routes if none is provided.
    fn build_default_router(&self) -> Router {
        let mut registry = RouteRegistry::new();

        // Collect all registered routes
        for route in ROUTE_REGISTRY.iter() {
            registry.register(route.clone());
        }

        if registry.routes.is_empty() {
            // Default welcome route
            Router::new().route("/", get(|| async { "Welcome to Navius WebPlugin!" }))
        } else {
            registry.build_router()
        }
    }

    /// Start the web server with the configured settings
    pub async fn start_server<T: Clone + Send + Sync + 'static>(&self, app_state: T) -> Result<()> {
        // Create router with explicit state
        let router = if let Some(router) = &self.router {
            // Convert the router to a service
            router
                .clone()
                .with_state(app_state)
                .into_make_service_with_connect_info::<std::net::SocketAddr>()
        } else {
            // Default router with registered routes
            let default_router = self.build_default_router();
            default_router
                .with_state(app_state)
                .into_make_service_with_connect_info::<std::net::SocketAddr>()
        };

        let addr = format!("{}:{}", self.host, self.port)
            .parse::<SocketAddr>()
            .map_err(|e| navius_core::error::Error::internal(format!("Invalid address: {}", e)))?;

        info!("Starting web server on {}", addr);

        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            navius_core::error::Error::internal(format!("Failed to bind to address: {}", e))
        })?;

        info!("Server listening on http://{}", addr);

        axum::serve(listener, router)
            .await
            .map_err(|e| navius_core::error::Error::internal(format!("Server error: {}", e)))?;

        Ok(())
    }
}

impl Default for WebPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PluginLifecycle for WebPlugin {
    async fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        // Extract configuration values if present
        if let Some(host) = config.get_string("host") {
            self.host = host;
        }

        if let Some(port) = config.get_number("port") {
            self.port = port as u16;
        }

        self.lifecycle_stage = PluginLifecycleStage::Initialized;
        info!(
            "Initialized WebPlugin with host: {}, port: {}",
            self.host, self.port
        );
        Ok(())
    }

    async fn start(&mut self) -> PluginResult<()> {
        self.lifecycle_stage = PluginLifecycleStage::Started;
        info!("Started WebPlugin - server will be started when needed");
        Ok(())
    }

    async fn stop(&mut self) -> PluginResult<()> {
        self.lifecycle_stage = PluginLifecycleStage::Stopped;
        info!("Stopped WebPlugin");
        Ok(())
    }

    async fn health_check(&self) -> PluginHealth {
        PluginHealth::Healthy
    }
}

#[async_trait]
impl MessageHandler for WebPlugin {
    async fn handle_message(&self, _message: Value) -> PluginResult<Option<Value>> {
        // No message handling for now
        Ok(None)
    }
}

impl Plugin for WebPlugin {
    fn id(&self) -> &str {
        self.base.metadata().id.as_str()
    }

    fn version(&self) -> &str {
        self.base.metadata().version.as_str()
    }

    fn metadata(&self) -> &PluginMetadata {
        self.base.metadata()
    }

    fn lifecycle_stage(&self) -> PluginLifecycleStage {
        self.lifecycle_stage
    }

    fn capabilities(&self) -> HashMap<String, Arc<dyn std::any::Any + Send + Sync>> {
        self.base.capabilities()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use navius_test::error::TestResult;

    #[tokio::test]
    async fn test_web_plugin_creation() -> TestResult<()> {
        let plugin = WebPlugin::new();
        assert_eq!(plugin.host(), "127.0.0.1");
        assert_eq!(plugin.port(), 3000);
        assert!(plugin.router.is_none());
        assert_eq!(plugin.lifecycle_stage(), PluginLifecycleStage::Created);
        Ok(())
    }

    #[tokio::test]
    async fn test_web_plugin_with_host() -> TestResult<()> {
        let plugin = WebPlugin::new().with_host("0.0.0.0");
        assert_eq!(plugin.host(), "0.0.0.0");
        Ok(())
    }

    #[tokio::test]
    async fn test_web_plugin_with_port() -> TestResult<()> {
        let plugin = WebPlugin::new().with_port(8080);
        assert_eq!(plugin.port(), 8080);
        Ok(())
    }

    #[tokio::test]
    async fn test_web_plugin_initialization() -> TestResult<()> {
        let mut plugin = WebPlugin::new();
        let config = PluginConfig::new();
        plugin.initialize(config).await?;
        assert_eq!(plugin.lifecycle_stage(), PluginLifecycleStage::Initialized);
        Ok(())
    }

    #[tokio::test]
    async fn test_web_plugin_lifecycle() -> TestResult<()> {
        let mut plugin = WebPlugin::new();

        // Initialize
        plugin.initialize(PluginConfig::new()).await?;
        assert_eq!(plugin.lifecycle_stage(), PluginLifecycleStage::Initialized);

        // Start
        plugin.start().await?;
        assert_eq!(plugin.lifecycle_stage(), PluginLifecycleStage::Started);

        // Health check
        let health = plugin.health_check().await;
        assert_eq!(health, PluginHealth::Healthy);

        // Stop
        plugin.stop().await?;
        assert_eq!(plugin.lifecycle_stage(), PluginLifecycleStage::Stopped);

        Ok(())
    }
}
