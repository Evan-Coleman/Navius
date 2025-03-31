use axum::routing::{delete, get, post};
use navius_http::server::HttpServerBuilder;
use std::sync::Arc;

use crate::api::controllers::notifications::{
    delete_notification, get_notifications, mark_all_as_read, mark_as_read, send_notification,
};
use crate::api::middleware::auth::{requires_admin, requires_auth};
use crate::infrastructure::ServiceRegistry;

/// Configure notification routes
pub fn configure_routes(
    server: HttpServerBuilder,
    registry: Arc<ServiceRegistry>,
) -> HttpServerBuilder {
    server
        // Collection routes
        .route(
            "/api/notifications",
            get(get_notifications).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/notifications",
            delete(mark_all_as_read).layer(requires_auth(registry.clone())),
        )
        // Individual resource routes
        .route(
            "/api/notifications/:id",
            post(mark_as_read).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/notifications/:id",
            delete(delete_notification).layer(requires_auth(registry.clone())),
        )
        // Administrative actions
        .route(
            "/api/notifications/send",
            post(send_notification)
                .layer(requires_admin())
                .layer(requires_auth(registry.clone())),
        )
}
