use axum::{Json, extract::State, response::IntoResponse};
use chrono::Utc;
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

    let now = Utc::now();
    let uptime_seconds = (now - state.start_time).num_seconds() as u64;

    let health_status = json!({
        "status": "UP",
        "database": db_status,
        "uptime": uptime_seconds,
    });

    Json(health_status)
}
