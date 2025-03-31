pub mod controllers;
pub mod middleware;
pub mod openapi;
pub mod routes;

use navius_http::server::HttpServerBuilder;
use std::sync::Arc;

use crate::infrastructure::ServiceRegistry;
use openapi::configure_openapi_routes;
use routes::{auth, categories, health, notifications, tasks, users};

/// Configure all routes for the application
pub fn configure_routes(
    server: HttpServerBuilder,
    registry: Arc<ServiceRegistry>,
) -> HttpServerBuilder {
    server
        .with_state(registry.clone())
        .pipe(|s| health::configure_routes(s))
        .pipe(|s| auth::configure_routes(s, registry.clone()))
        .pipe(|s| tasks::configure_routes(s, registry.clone()))
        .pipe(|s| users::configure_routes(s, registry.clone()))
        .pipe(|s| categories::configure_routes(s, registry.clone()))
        .pipe(|s| notifications::configure_routes(s, registry.clone()))
        .pipe(|s| {
            // Add OpenAPI/Swagger UI routes
            let router = s.into_router();
            let router = configure_openapi_routes(router);
            HttpServerBuilder::from_router(router)
        })
}
