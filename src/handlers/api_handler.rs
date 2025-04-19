use axum::extract::State;
use axum::{Json, response::IntoResponse};
use axum::{extract::Path, http::StatusCode};
use navius_core::app::Registry;
use navius_core::di::registry::ComponentRegistry;
use navius_http::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::info;

/// Simple hello world handler
pub async fn hello_world() -> Json<serde_json::Value> {
    Json(json!({ "message": "Hello, world!" }))
}

/// Echo the text passed in the URL
pub async fn echo(Path(text): Path<String>) -> String {
    format!("Echo: {}", text)
}

/// Handle user requests - GET to list users, POST to create a user
pub async fn users(
    // Optional payload for POST requests
    payload: Option<Json<UserRequest>>,
) -> impl IntoResponse {
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

/// Get a single user by ID
pub async fn get_user(Path(id): Path<String>) -> impl IntoResponse {
    let user = User {
        id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };
    Json(user)
}

/// List all users
pub async fn list_users() -> impl IntoResponse {
    let users = vec![
        User {
            id: "1".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        },
        User {
            id: "2".to_string(),
            name: "Jane Smith".to_string(),
            email: "jane@example.com".to_string(),
        },
    ];
    Json(users)
}

/// Create a new user
pub async fn create_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    let user = User {
        id: "new-id".to_string(),
        name: payload.name,
        email: payload.email,
    };
    (StatusCode::CREATED, Json(user))
}

/// Information about the application context
pub async fn context_info(State(state): State<Arc<Registry>>) -> impl IntoResponse {
    let component_types = state
        .component_types()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let response = format!("Application has components of types: {}", component_types);
    response
}

#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
}

/// Request model for the users endpoint
#[derive(Deserialize)]
pub struct UserRequest {
    pub name: String,
}
