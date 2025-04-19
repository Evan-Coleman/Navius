use axum::extract::State;
use axum::{Json, response::IntoResponse};
use axum::{extract::Path, http::StatusCode};
use navius_core::di::registry::ComponentRegistry;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

// Define a simple Registry type for the example - could be any struct
pub struct Registry {}

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
        (
            StatusCode::OK,
            Json(json!([
                { "id": 1, "name": "Alice" },
                { "id": 2, "name": "Bob" },
            ])),
        )
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
pub async fn context_info(State(_state): State<Arc<Registry>>) -> impl IntoResponse {
    // Simplified implementation that doesn't depend on actual Registry methods
    "Application context information"
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
