use axum::routing::{get, post};
use navius_http::server::HttpServerBuilder;

use crate::api::controllers::auth;

// Configure auth routes
pub fn configure_routes(server: HttpServerBuilder) -> HttpServerBuilder {
    server
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/refresh", post(auth::refresh_token))
        .route("/api/auth/logout", get(auth::logout))
}
