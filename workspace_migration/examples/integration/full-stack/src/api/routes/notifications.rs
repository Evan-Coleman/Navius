use axum::routing::{delete, get, post};
use navius_http::server::HttpServerBuilder;

use crate::api::controllers::notifications::{
    delete_notification, get_notifications, mark_all_as_read, mark_as_read, send_notification,
};
use crate::api::middleware::auth::{requires_admin, requires_auth};

/// Configure notification routes
pub fn configure_routes(server: HttpServerBuilder) -> HttpServerBuilder {
    server
        .route(
            "/api/notifications",
            get(get_notifications)
                .delete(mark_all_as_read)
                .layer(requires_auth()),
        )
        .route(
            "/api/notifications/:id",
            post(mark_as_read)
                .delete(delete_notification)
                .layer(requires_auth()),
        )
        .route(
            "/api/notifications/send",
            post(send_notification).layer(requires_admin()),
        )
}
