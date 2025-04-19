// WebPlugin provides web server capabilities to Navius applications
// This file implements the web plugin abstraction for the Zero Boilerplate Initiative (NC-11)

use axum::{Router, routing::get};
use navius_core::{
    di::Application,
    error::{Error, Result},
};
use std::net::SocketAddr;
use tokio::runtime::Handle;
use tracing::{debug, info};

use crate::server::{
    self,
    route_discovery::{RouteDiscoveryConfig, RouteRegistration, RouteRegistry},
};

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
/// // Use with a Navius Application
/// let app = navius_core::di::Application::builder().build();
/// // TODO: Add direct plugin support
/// ```
#[derive(Debug)]
pub struct WebPlugin {
    /// Base path for all routes (e.g., "/api")
    base: String,
    /// Host address to bind to (e.g., "127.0.0.1" or "0.0.0.0")
    host: String,
    /// Port to listen on (e.g., 3000)
    port: u16,
    /// Router with all configured routes
    router: Option<Router>,
    /// Route discovery configuration
    route_discovery: RouteDiscoveryConfig,
}

/// Define a simple application state that contains the application context
#[derive(Clone, Debug)]
pub struct AppState<T = ()> {
    /// Application state that can be accessed in route handlers
    pub state: T,
}

impl Default for WebPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WebPlugin {
    /// Create a new WebPlugin with default settings
    pub fn new() -> Self {
        Self {
            base: String::new(),
            host: "127.0.0.1".to_string(),
            port: 3000,
            router: None,
            route_discovery: RouteDiscoveryConfig::new(),
        }
    }

    /// Set the base path for all routes
    pub fn with_base<T: Into<String>>(mut self, base: T) -> Self {
        self.base = base.into();
        self
    }

    /// Set the host address
    pub fn with_host<T: Into<String>>(mut self, host: T) -> Self {
        self.host = host.into();
        self
    }

    /// Set the port number
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Configure route discovery behavior
    pub fn with_route_discovery(mut self, config: RouteDiscoveryConfig) -> Self {
        self.route_discovery = config;
        self
    }

    /// Set the router for the web server
    pub fn with_router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    /// Build a router from registered routes if none is provided.
    fn build_default_router(&self) -> Router {
        let registry = RouteRegistry::new();

        info!("Discovering routes using inventory collect");

        // Collect routes from inventory
        for route in inventory::iter::<RouteRegistration>() {
            debug!("Registering route: {}", route.path);
            let registration = route.clone();
            registry.register_route(registration);
        }

        if registry.route_count() == 0 {
            // Default welcome route
            Router::new().route("/", get(|| async { "Welcome to Navius WebPlugin!" }))
        } else {
            registry.build_router()
        }
    }

    /// Start the web server - can be used directly or through a runtime
    pub async fn start_server(&self) -> Result<()> {
        // Get the address to bind to
        let addr = SocketAddr::new(
            self.host
                .parse()
                .map_err(|_| Error::internal(format!("Invalid IP address: {}", self.host)))?,
            self.port,
        );

        // Create a router, either from the provided one or by discovering routes
        let router = if let Some(router) = &self.router {
            router.clone()
        } else {
            self.build_default_router()
        };

        // Create a shutdown channel
        let (_shutdown_sender, shutdown_future) = server::create_shutdown_channel();

        // Bind to the address
        let listener = server::bind_listener(&addr).await?;

        // Start the server
        info!("Starting HTTP server on http://{}:{}", self.host, self.port);

        // Use axum to serve the router
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                server::shutdown_future(shutdown_future).await;
            })
            .await
            .map_err(|e| Error::internal(format!("HTTP server error: {}", e)))?;

        Ok(())
    }

    /// Run the web server in a separate task on the specified runtime
    pub fn run_in_background(&self, handle: Handle) -> Result<()> {
        // Clone self for the async block
        let plugin = self.clone();

        // Start the server in the background
        handle.spawn(async move {
            if let Err(e) = plugin.start_server().await {
                tracing::error!("Failed to start web server: {}", e);
            }
        });

        Ok(())
    }
}

// Need to implement Clone for WebPlugin
impl Clone for WebPlugin {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            host: self.host.clone(),
            port: self.port,
            router: self.router.clone(),
            route_discovery: self.route_discovery.clone(),
        }
    }
}

/// Extension trait to add the web plugin to an Application
pub trait WebPluginExt {
    fn with_web_plugin(self, plugin: WebPlugin) -> Self;
}

impl WebPluginExt for Application {
    fn with_web_plugin(self, _plugin: WebPlugin) -> Self {
        // TODO: Implement proper plugin registration
        self
    }
}
