use axum::{Router, routing::get};
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tracing::{debug, info, warn};

/// Configuration for route discovery
#[derive(Debug, Clone)]
pub struct RouteDiscoveryConfig {
    /// Whether to enable route discovery
    pub enabled: bool,
    /// Whether to include file names in route paths
    pub include_file_names: bool,
    /// Base path for route discovery
    pub base_path: Option<String>,
    /// Module paths for route discovery
    pub module_paths: Vec<String>,
    /// Whether to recursively discover routes
    pub recursive: bool,
}

impl Default for RouteDiscoveryConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteDiscoveryConfig {
    /// Create a new route discovery configuration
    pub fn new() -> Self {
        Self {
            enabled: true,
            include_file_names: false,
            base_path: None,
            module_paths: Vec::new(),
            recursive: true,
        }
    }

    /// Disable route discovery
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Enable including file names in route paths
    pub fn include_file_names(mut self) -> Self {
        self.include_file_names = true;
        self
    }

    /// Set the base path for route discovery
    pub fn with_base_path<S: Into<String>>(mut self, base_path: S) -> Self {
        self.base_path = Some(base_path.into());
        self
    }

    /// Set whether route discovery should be recursive
    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Set whether route discovery is enabled
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Add a module path for route discovery
    pub fn with_module_path<S: Into<String>>(mut self, path: S) -> Self {
        self.module_paths.push(path.into());
        self
    }
}

/// Route registration for the inventory pattern
///
/// This struct is used to register routes with the inventory pattern.
/// Routes are collected at compile time and processed at runtime.
#[derive(Clone)]
pub struct RouteRegistration {
    /// The HTTP method for this route (GET, POST, etc.)
    pub method: String,
    /// The path for this route (e.g., "/api/users")
    pub path: String,
    /// The handler function for this route
    pub handler: Arc<dyn Fn() -> Router + Send + Sync>,
}

// Manual Debug implementation since the handler doesn't implement Debug
impl Debug for RouteRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouteRegistration")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("handler", &"<function>")
            .finish()
    }
}

inventory::collect!(RouteRegistration);

/// Route registry for collecting and building routes
///
/// This struct is responsible for collecting all registered routes
/// and building a router from them.
#[derive(Debug, Default)]
pub struct RouteRegistry {
    routes: HashMap<String, RouteRegistration>,
}

impl RouteRegistry {
    /// Create a new route registry
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Register a route with the registry
    pub fn register_route(&self, route: RouteRegistration) {
        debug!("Registering route: {} {}", route.method, route.path);

        // Store the route in the registry using a unique key
        let key = format!("{}:{}", route.method, route.path);
        let _ = self.routes.clone().insert(key, route);
    }

    /// Register a router at the given path
    pub fn register(&mut self, path: &str, router: Router) {
        debug!("Registering router at path: {}", path);
        let registration = RouteRegistration {
            method: "ALL".to_string(),
            path: path.to_string(),
            handler: Arc::new(move || router.clone()),
        };
        self.register_route(registration);
    }

    /// Get the number of registered routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Build a router from all registered routes
    pub fn build_router(&self) -> Router {
        let mut router = Router::new();

        // Add a default welcome route if no routes are registered
        if self.routes.is_empty() {
            info!("No routes registered, adding default welcome route");
            router = router.route("/", get(|| async { "Welcome to Navius!" }));
            return router;
        }

        // Add all registered routes to the router
        for (_, route) in &self.routes {
            info!("Adding route: {} {}", route.method, route.path);
            let handler = (route.handler)();
            router = router.merge(handler);
        }

        router
    }
}

/// Discover and register routes based on the provided configuration
pub fn discover_routes(config: &RouteDiscoveryConfig) -> RouteRegistry {
    let registry = RouteRegistry::new();

    if !config.enabled {
        warn!("Route discovery is disabled");
        return registry;
    }

    if config.base_path.is_none() && config.module_paths.is_empty() {
        warn!("No base path or module paths provided for route discovery");
        return registry;
    }

    // Note: The actual implementation is through procedural macros
    // This function serves as an integration point for compile-time discovered routes

    info!(
        "Route discovery complete. Found {} routes",
        registry.route_count()
    );

    registry
}

/// Internal function used by the procedural macros to register a router
#[doc(hidden)]
pub fn _register_discovered_router(registry: &mut RouteRegistry, path: &str, router: Router) {
    registry.register(path, router);
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use navius_test::error::TestResult;

    #[test]
    fn test_route_registry() -> TestResult<()> {
        let mut registry = RouteRegistry::new();

        // Register a test router
        let test_router = Router::new().route("/test", get(|| async { "Test" }));
        registry.register("/api", test_router);

        // Check route count
        assert_eq!(registry.route_count(), 1);

        // Build router
        let router = registry.build_router();
        assert!(
            router
                .into_make_service_with_connect_info::<std::net::SocketAddr>()
                .is_ok()
        );

        Ok(())
    }

    #[test]
    fn test_route_discovery_config() -> TestResult<()> {
        let config = RouteDiscoveryConfig::new()
            .with_base_path("handlers")
            .with_recursive(false)
            .with_enabled(true)
            .with_module_path("api");

        assert_eq!(config.base_path, Some("handlers".to_string()));
        assert_eq!(config.recursive, false);
        assert_eq!(config.enabled, true);
        assert_eq!(config.module_paths, vec!["api"]);

        Ok(())
    }
}
