use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use navius_http::WebPlugin;
use navius_macros::{navius_app, nest, route};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// API module with annotated route handlers
#[nest(prefix = "/api/v1")]
mod api {
    use super::*;

    #[route(path = "/hello", method = "GET")]
    async fn hello() -> Json<serde_json::Value> {
        Json(json!({ "message": "Hello from zero-boilerplate API!" }))
    }

    #[route(path = "/users/:id", method = "GET")]
    async fn get_user(Path(id): Path<String>) -> Json<serde_json::Value> {
        Json(json!({
            "id": id,
            "name": "Sample User",
            "email": "user@example.com"
        }))
    }

    #[route(path = "/users", method = ["GET", "POST"])]
    async fn users(
        // Only used for POST requests
        // For GET requests, this will be ignored
        payload: Option<Json<CreateUser>>,
    ) -> impl IntoResponse {
        if let Some(Json(user_data)) = payload {
            // Create a new user (POST)
            let user = User {
                id: 1337,
                username: user_data.username,
            };
            return (StatusCode::CREATED, Json(user)).into_response();
        }

        // Return user list (GET)
        let users = vec![
            User {
                id: 1,
                username: "alice".to_string(),
            },
            User {
                id: 2,
                username: "bob".to_string(),
            },
        ];
        Json(users).into_response()
    }

    #[route(path = "/context", method = "GET")]
    async fn context_info(State(state): State<navius_http::AppState>) -> Json<serde_json::Value> {
        Json(json!({
            "components": state.app.registry().component_types().len(),
            "app_id": "zero-boilerplate-example"
        }))
    }

    // Example of wildcard path handling
    #[route(path = "/assets/*path", method = "GET")]
    async fn serve_asset(Path(path): Path<String>) -> impl IntoResponse {
        format!("Would serve asset: {}", path)
    }
}

/// Admin module with annotated route handlers
#[nest(prefix = "/admin")]
mod admin {
    use super::*;

    #[route(path = "/status", method = "GET")]
    async fn status() -> Json<serde_json::Value> {
        Json(json!({
            "status": "OK",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime": "10m"
        }))
    }

    // Capture path parameters in nested routes
    #[route(path = "/metrics/:type", method = "GET")]
    async fn metrics(Path(params): Path<HashMap<String, String>>) -> impl IntoResponse {
        let metric_type = params.get("type").unwrap_or(&"default".to_string());
        format!("Admin metrics for type: {}", metric_type)
    }
}

// Data structures for request and response
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

/// Zero-boilerplate application main function
#[navius_app(
    name = "zero-boilerplate-example",
    routes = [self::api, self::admin],
    plugins = [WebPlugin]
)]
async fn main() {
    // This is the only code you need to write - everything else is handled by the macro
    tracing::info!("Zero-boilerplate application started!");
    tracing::info!("Application ready to serve requests!");

    // The following routes are automatically available:
    // - GET /api/v1/hello
    // - GET /api/v1/users/:id
    // - GET /api/v1/users
    // - POST /api/v1/users
    // - GET /api/v1/context
    // - GET /api/v1/assets/*path
    // - GET /admin/status
    // - GET /admin/metrics/:type
}
