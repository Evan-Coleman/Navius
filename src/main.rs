use crate::handlers::admin_handler;
use crate::handlers::api_handler;
use axum::Json;
use navius_macros::{navius_app, nest, route};
use serde_json::json;

// API route module with routes
#[nest(prefix = "/api/v1")]
mod api {
    use super::*;

    #[route(path = "/hello", method = "GET")]
    async fn hello_world() -> Json<serde_json::Value> {
        api_handler::hello_world().await
    }

    #[route(path = "/echo/:text", method = "GET")]
    async fn echo(axum::extract::Path(text): axum::extract::Path<String>) -> String {
        api_handler::echo(axum::extract::Path(text)).await
    }

    #[route(path = "/users", method = ["GET", "POST"])]
    async fn users(
        // Optional payload for POST requests
        payload: Option<Json<api_handler::UserRequest>>,
    ) -> impl axum::response::IntoResponse {
        api_handler::users(payload).await
    }
}

// Admin routes
#[nest(prefix = "/admin")]
mod admin {
    use super::*;

    #[route(path = "/status", method = "GET")]
    async fn status() -> Json<serde_json::Value> {
        admin_handler::status().await
    }
}

// Main application with zero boilerplate
#[navius_app(
    name = "zero-boilerplate-app",
    routes = [self::api, self::admin]
)]
async fn main() {
    // The macro handles all the boilerplate for us!
    tracing::info!("Zero-boilerplate application started!");
}
