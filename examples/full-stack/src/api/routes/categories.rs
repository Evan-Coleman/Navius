use axum::routing::{delete, get, post, put};
use navius_http::server::HttpServerBuilder;
use std::sync::Arc;

use crate::api::controllers::categories::{
    create_category, delete_category, get_categories, get_category, get_tasks_by_category,
    update_category,
};
use crate::api::middleware::auth::{requires_auth, requires_manager};
use crate::infrastructure::ServiceRegistry;

/// Configure category routes
pub fn configure_routes(
    server: HttpServerBuilder,
    registry: Arc<ServiceRegistry>,
) -> HttpServerBuilder {
    server
        // Collection routes
        .route(
            "/api/categories",
            get(get_categories).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/categories",
            post(create_category)
                .layer(requires_manager())
                .layer(requires_auth(registry.clone())),
        )
        // Individual resource routes
        .route(
            "/api/categories/:id",
            get(get_category).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/categories/:id",
            put(update_category)
                .layer(requires_manager())
                .layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/categories/:id",
            delete(delete_category)
                .layer(requires_manager())
                .layer(requires_auth(registry.clone())),
        )
        // Sub-resource routes
        .route(
            "/api/categories/:id/tasks",
            get(get_tasks_by_category).layer(requires_auth(registry.clone())),
        )
}
