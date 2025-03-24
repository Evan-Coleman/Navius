use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

use crate::core::router::AppState;

pub async fn metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    state.metrics_handle.render()
}
