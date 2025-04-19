use axum::Router;
use std::collections::HashMap;
use tracing::{info, warn};

/// Configuration for route discovery
#[derive(Debug, Clone)]
pub struct RouteDiscoveryConfig {
    /// Base path to scan for route handlers
    pub base_path: Option<String>,
    /// Whether to scan recursively
    pub recursive: bool,
    /// Whether route discovery is enabled
    pub enabled: bool,
    /// Additional module paths to scan (e.g., "api", "handlers")
    pub module_paths: Vec<String>,
}

impl Default for RouteDiscoveryConfig {
    fn default() -> Self {
        Self {
            base_path: None,
            recursive: true,
            enabled: true,
            module_paths: Vec::new(),
        }
    }
}

impl RouteDiscoveryConfig {
    /// Create a new route discovery configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base path to scan for route handlers
    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = Some(path.into());
        self
    }

    /// Set whether to scan recursively
    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Set whether route discovery is enabled
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Add a module path to scan for routes
    pub fn with_module_path(mut self, path: impl Into<String>) -> Self {
        self.module_paths.push(path.into());
        self
    }

    /// Add multiple module paths to scan for routes
    pub fn with_module_paths(mut self, paths: Vec<impl Into<String>>) -> Self {
        for path in paths {
            self.module_paths.push(path.into());
        }
        self
    }
}

/// A registry of discovered routes
#[derive(Debug, Default)]
pub struct RouteRegistry {
    routes: HashMap<String, Router>,
}

impl RouteRegistry {
    /// Create a new route registry
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Register a router with a path prefix
    pub fn register(&mut self, path: impl Into<String>, router: Router) {
        let path = path.into();
        info!("Registering router at path: {}", path);
        self.routes.insert(path, router);
    }

    /// Get all registered routes as a single router
    pub fn build_router(&self) -> Router {
        let mut main_router = Router::new();

        for (path, router) in &self.routes {
            info!("Adding router at path: {}", path);
            main_router = main_router.nest(path, router.clone());
        }

        main_router
    }

    /// Get the number of registered routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
}

/// Discover and register routes based on the provided configuration
pub fn discover_routes(config: &RouteDiscoveryConfig) -> RouteRegistry {
    let mut registry = RouteRegistry::new();

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
