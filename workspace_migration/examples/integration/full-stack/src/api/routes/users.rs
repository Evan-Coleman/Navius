use axum::routing::{delete, get, post, put};
use navius_http::server::HttpServerBuilder;
use std::sync::Arc;

use crate::api::controllers::users;
use crate::api::middleware::auth::{requires_admin, requires_auth};
use crate::infrastructure::ServiceRegistry;

/// Configure user routes
pub fn configure_routes(
    server: HttpServerBuilder,
    registry: Arc<ServiceRegistry>,
) -> HttpServerBuilder {
    server
        // Collection routes
        .route(
            "/api/users",
            get(users::get_users).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/users",
            post(users::create_user)
                .layer(requires_admin())
                .layer(requires_auth(registry.clone())),
        )
        // Individual resource routes
        .route(
            "/api/users/:id",
            get(users::get_user).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/users/:id",
            put(users::update_user).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/users/:id",
            delete(users::delete_user)
                .layer(requires_admin())
                .layer(requires_auth(registry.clone())),
        )
        // Profile sub-resource route
        .route(
            "/api/users/:id/profile",
            put(users::update_profile).layer(requires_auth(registry.clone())),
        )
}
