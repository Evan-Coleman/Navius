use std::sync::Arc;

use navius_http::server::HttpServerBuilder;

use crate::infrastructure::ServiceRegistry;

/// Configure all routes for the application
pub fn configure_routes(
    server: HttpServerBuilder,
    registry: Arc<ServiceRegistry>,
) -> HttpServerBuilder {
    // Add route configurations here
    // For now, return the server as-is
    server
}
