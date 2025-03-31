use crate::infrastructure::ServiceRegistry;
use navius_http::server::HttpServer;
use std::sync::Arc;

mod controllers;
mod middleware;
mod models;

#[cfg(feature = "openapi")]
mod openapi;

/// Configure the application routes
pub fn configure_routes(server: HttpServer, service_registry: Arc<ServiceRegistry>) -> HttpServer {
    let mut server = server;

    // Set up middleware
    server = middleware::configure_middleware(server);

    // Configure API routes
    server = controllers::configure_controllers(server, service_registry.clone());

    // Configure OpenAPI routes if feature is enabled
    #[cfg(feature = "openapi")]
    {
        server = openapi::configure_openapi_routes(server);
    }

    server
}

/// Create a test version of the application for integration tests
#[cfg(test)]
pub fn create_test_app() -> HttpServer {
    use crate::infrastructure::ServiceRegistry;
    use navius_http::server::HttpServerBuilder;

    let server = HttpServerBuilder::new().with_test_config().build();

    let service_registry = Arc::new(ServiceRegistry::new());

    configure_routes(server, service_registry)
}
