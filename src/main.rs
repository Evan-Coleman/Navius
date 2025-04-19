mod handlers;

use navius_http::WebPlugin;
use navius_macros::{navius_app, nest, route};

// Main application with zero boilerplate
#[navius_app(
    name = "simmr",
    routes = "self::api, self::admin",
    plugins = "WebPlugin"
)]
async fn main() {
    tracing::info!("Simmr application started!");
}

// API route module with routes
#[nest(prefix = "/api/v1")]
mod api {
    use super::*;
    use crate::handlers::api_handlers;
    use axum::Json;

    #[route(path = "/hello", method = "GET")]
    async fn hello_world() -> Json<serde_json::Value> {
        api_handlers::hello_world().await
    }

    #[route(path = "/echo/:text", method = "GET")]
    async fn echo(axum::extract::Path(text): axum::extract::Path<String>) -> String {
        api_handlers::echo(axum::extract::Path(text)).await
    }

    #[route(path = "/users", method = "GET")]
    async fn users() -> impl axum::response::IntoResponse {
        api_handlers::users(None).await
    }
}

// Admin routes
#[nest(prefix = "/admin")]
mod admin {
    use super::*;
    use crate::handlers::admin_handlers;
    use axum::Json;

    #[route(path = "/status", method = "GET")]
    async fn status() -> Json<serde_json::Value> {
        admin_handlers::status().await
    }
}
