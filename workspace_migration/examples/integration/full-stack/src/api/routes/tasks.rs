use axum::routing::{delete, get, post, put};
use navius_http::server::HttpServerBuilder;

use crate::api::controllers::tasks::{
    add_comment, assign_task, create_task, delete_comment, delete_task, get_task,
    get_task_comments, get_tasks, unassign_task, update_comment, update_task,
};
use crate::api::middleware::auth::requires_auth;

/// Configure task routes
pub fn configure_routes(server: HttpServerBuilder) -> HttpServerBuilder {
    server
        // Task management
        .route(
            "/api/tasks",
            get(get_tasks).post(create_task).layer(requires_auth()),
        )
        .route(
            "/api/tasks/:id",
            get(get_task)
                .put(update_task)
                .delete(delete_task)
                .layer(requires_auth()),
        )
        .route(
            "/api/tasks/:id/assign/:user_id",
            post(assign_task).layer(requires_auth()),
        )
        .route(
            "/api/tasks/:id/unassign",
            post(unassign_task).layer(requires_auth()),
        )
        // Comments
        .route(
            "/api/tasks/:id/comments",
            get(get_task_comments)
                .post(add_comment)
                .layer(requires_auth()),
        )
        .route(
            "/api/tasks/:task_id/comments/:comment_id",
            put(update_comment)
                .delete(delete_comment)
                .layer(requires_auth()),
        )
}
