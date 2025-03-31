use axum::routing::{get, post};
use navius_http::server::HttpServerBuilder;
use std::sync::Arc;

use crate::api::controllers::auth;
use crate::api::middleware::auth::requires_auth;
use crate::infrastructure::ServiceRegistry;

// Configure auth routes
pub fn configure_routes(
    server: HttpServerBuilder,
    registry: Arc<ServiceRegistry>,
) -> HttpServerBuilder {
    server
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/refresh", post(auth::refresh_token))
        .route(
            "/api/auth/logout",
            get(auth::logout).layer(requires_auth(registry.clone())),
        )
}
