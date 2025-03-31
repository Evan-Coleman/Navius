use axum::routing::{delete, get, post, put};
use navius_http::server::HttpServerBuilder;
use std::sync::Arc;

use crate::api::controllers::tasks::{
    add_comment, assign_task, create_task, delete_comment, delete_task, get_task,
    get_task_comments, get_tasks, unassign_task, update_comment, update_task,
};
use crate::api::middleware::auth::{requires_auth, requires_manager};
use crate::infrastructure::ServiceRegistry;

/// Configure task routes
pub fn configure_routes(
    server: HttpServerBuilder,
    registry: Arc<ServiceRegistry>,
) -> HttpServerBuilder {
    server
        // Collection routes
        .route(
            "/api/tasks",
            get(get_tasks).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/tasks",
            post(create_task).layer(requires_auth(registry.clone())),
        )
        // Individual resource routes
        .route(
            "/api/tasks/:id",
            get(get_task).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/tasks/:id",
            put(update_task).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/tasks/:id",
            delete(delete_task).layer(requires_auth(registry.clone())),
        )
        // Assignment sub-resource routes
        .route(
            "/api/tasks/:id/assign/:user_id",
            post(assign_task)
                .layer(requires_manager())
                .layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/tasks/:id/unassign",
            post(unassign_task)
                .layer(requires_manager())
                .layer(requires_auth(registry.clone())),
        )
        // Comments sub-resource routes
        .route(
            "/api/tasks/:id/comments",
            get(get_task_comments).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/tasks/:id/comments",
            post(add_comment).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/tasks/:task_id/comments/:comment_id",
            put(update_comment).layer(requires_auth(registry.clone())),
        )
        .route(
            "/api/tasks/:task_id/comments/:comment_id",
            delete(delete_comment).layer(requires_auth(registry.clone())),
        )
}
