use axum::routing::get;
use navius_http::server::HttpServerBuilder;

use crate::api::controllers::health::{health_check, liveness_check, readiness_check};

/// Configure health check routes
pub fn configure_routes(server: HttpServerBuilder) -> HttpServerBuilder {
    server
        .route("/health", get(health_check))
        .route("/health/readiness", get(readiness_check))
        .route("/health/liveness", get(liveness_check))
}
