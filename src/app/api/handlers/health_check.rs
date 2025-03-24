use axum::{Json, extract::State, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;

use crate::core::router::AppState;

pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_status = match state.get_db_pool() {
        Ok(pool) => match pool.acquire().await {
            Ok(_) => "UP",
            Err(_) => "DOWN",
        },
        Err(_) => "DOWN",
    };

    let health_status = json!({
        "status": "UP",
        "database": db_status,
        "uptime": state.start_time.elapsed().as_secs(),
    });

    Json(health_status)
}
