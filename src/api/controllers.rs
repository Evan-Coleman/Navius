use crate::infrastructure::ServiceRegistry;
use navius_http::server::HttpServer;
use std::sync::Arc;

mod health;

// Add other controller modules as needed: users, auth, etc.

/// Configure all API controllers
pub fn configure_controllers(
    server: HttpServer,
    service_registry: Arc<ServiceRegistry>,
) -> HttpServer {
    let server = health::configure_routes(server);

    // Add other controller route configurations here

    server
}
