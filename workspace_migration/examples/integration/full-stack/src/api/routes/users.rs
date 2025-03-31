use axum::routing::{delete, get, post, put};
use navius_http::server::HttpServerBuilder;

use crate::api::controllers::users;

// Configure user routes
pub fn configure_routes(server: HttpServerBuilder) -> HttpServerBuilder {
    server
        .route("/api/users", get(users::get_users))
        .route("/api/users", post(users::create_user))
        .route("/api/users/:id", get(users::get_user))
        .route("/api/users/:id", put(users::update_user))
        .route("/api/users/:id", delete(users::delete_user))
        .route("/api/users/:id/profile", put(users::update_profile))
}
