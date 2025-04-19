use crate::plugins::web_plugin::AppState;
use axum::extract::State;
use axum::{Json, response::IntoResponse};
use navius_core::di::registry::ComponentRegistry;
use serde_json::json;
use tracing::info;

/// Hello world handler
pub async fn hello_world() -> impl IntoResponse {
    Json(json!({ "message": "Hello, world!" }))
}
