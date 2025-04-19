use axum::Json;
use navius_macros::{navius_app, nest, route};
use serde_json::json;

// API route module with routes
#[nest(prefix = "/api/v1")]
mod api {
    use super::*;

    #[route(path = "/hello", method = "GET")]
    async fn hello_world() -> Json<serde_json::Value> {
        Json(json!({ "message": "Hello, world!" }))
    }

    #[route(path = "/echo/:text", method = "GET")]
    async fn echo(axum::extract::Path(text): axum::extract::Path<String>) -> String {
        format!("Echo: {}", text)
    }

    #[route(path = "/users", method = ["GET", "POST"])]
    async fn users(
        // Optional payload for POST requests
        payload: Option<Json<UserRequest>>,
    ) -> impl axum::response::IntoResponse {
        use axum::http::StatusCode;

        if let Some(Json(user)) = payload {
            // Create user (POST)
            (
                StatusCode::CREATED,
                Json(json!({
                    "id": 1234,
                    "name": user.name,
                    "created": true
                })),
            )
        } else {
            // Get all users (GET)
            Json(json!([
                { "id": 1, "name": "Alice" },
                { "id": 2, "name": "Bob" },
            ]))
        }
    }
}

// Admin routes
#[nest(prefix = "/admin")]
mod admin {
    use super::*;

    #[route(path = "/status", method = "GET")]
    async fn status() -> Json<serde_json::Value> {
        Json(json!({
            "status": "OK",
            "uptime": "10m",
            "version": env!("CARGO_PKG_VERSION")
        }))
    }
}

// Request/response models
#[derive(serde::Deserialize)]
struct UserRequest {
    name: String,
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
