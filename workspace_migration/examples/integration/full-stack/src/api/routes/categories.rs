use axum::routing::{delete, get, post, put};
use navius_http::server::HttpServerBuilder;

use crate::api::controllers::categories::{
    create_category, delete_category, get_categories, get_category, get_tasks_by_category,
    update_category,
};
use crate::api::middleware::auth::{requires_auth, requires_manager};

/// Configure category routes
pub fn configure_routes(server: HttpServerBuilder) -> HttpServerBuilder {
    server
        .route(
            "/api/categories",
            get(get_categories)
                .post(create_category)
                .layer(requires_auth()),
        )
        .route(
            "/api/categories/:id",
            get(get_category)
                .put(update_category)
                .delete(delete_category)
                .layer(requires_auth()),
        )
        .route(
            "/api/categories/:id/tasks",
            get(get_tasks_by_category).layer(requires_auth()),
        )
}
